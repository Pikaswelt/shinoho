//! Ende-zu-Ende-Test: SOCKS5 → Shinoho-Tunnel → WireGuard → Test-Server mit
//! Echo-Dienst, alles lokal auf 127.0.0.1.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use shinoho_vpn::{Tunnel, WgConfig};
use smoltcp::iface::{Config, Interface, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::tcp;
use smoltcp::time::Instant as SmolInstant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr};
use std::collections::VecDeque;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};

const ECHO_PORT: u16 = 7;

/// Kleine virtuelle Netzwerkkarte für die Server-Seite des Tests.
#[derive(Default)]
struct Dev {
    rx: VecDeque<Vec<u8>>,
    tx: VecDeque<Vec<u8>>,
}
struct Rx(Vec<u8>);
struct Tx<'a>(&'a mut VecDeque<Vec<u8>>);
impl RxToken for Rx {
    fn consume<R, F: FnOnce(&[u8]) -> R>(self, f: F) -> R {
        f(&self.0)
    }
}
impl TxToken for Tx<'_> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        let mut b = vec![0; len];
        let r = f(&mut b);
        self.0.push_back(b);
        r
    }
}
impl Device for Dev {
    type RxToken<'a> = Rx;
    type TxToken<'a> = Tx<'a>;
    fn receive(&mut self, _: SmolInstant) -> Option<(Rx, Tx<'_>)> {
        let p = self.rx.pop_front()?;
        Some((Rx(p), Tx(&mut self.tx)))
    }
    fn transmit(&mut self, _: SmolInstant) -> Option<Tx<'_>> {
        Some(Tx(&mut self.tx))
    }
    fn capabilities(&self) -> DeviceCapabilities {
        let mut c = DeviceCapabilities::default();
        c.medium = Medium::Ip;
        c.max_transmission_unit = 1420;
        c
    }
}

/// Test-VPN-Server: WireGuard-Peer mit Adresse 10.8.0.1 und TCP-Echo auf Port 7.
async fn run_server(udp: UdpSocket, server_secret: StaticSecret, client_public: PublicKey) {
    let mut tunn = Tunn::new(server_secret, client_public, None, None, 7, None);
    let mut dev = Dev::default();
    let mut iface = Interface::new(
        Config::new(HardwareAddress::Ip),
        &mut dev,
        SmolInstant::from_millis(0),
    );
    iface.update_ip_addrs(|a| {
        a.push(IpCidr::new(IpAddress::v4(10, 8, 0, 1), 24)).unwrap();
    });
    let mut sockets = SocketSet::new(Vec::new());
    let listen = || {
        let mut s = tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0; 65_536]),
            tcp::SocketBuffer::new(vec![0; 65_536]),
        );
        s.listen(ECHO_PORT).unwrap();
        s
    };
    let handle = sockets.add(listen());
    let start = Instant::now();
    let now = || SmolInstant::from_millis(start.elapsed().as_millis() as i64);
    let mut peer: Option<SocketAddr> = None;
    let mut inc = vec![0u8; 65_536];
    let mut out = vec![0u8; 65_536];

    loop {
        iface.poll(now(), &mut dev, &mut sockets);
        let s = sockets.get_mut::<tcp::Socket>(handle);
        if s.can_recv() && s.can_send() {
            let mut buf = vec![0u8; 4096];
            let n = s.recv_slice(&mut buf).unwrap();
            s.send_slice(&buf[..n]).unwrap();
        }
        if s.state() == tcp::State::CloseWait {
            s.close();
        }
        iface.poll(now(), &mut dev, &mut sockets);
        if let Some(addr) = peer {
            for p in dev.tx.drain(..).collect::<Vec<_>>() {
                if let TunnResult::WriteToNetwork(d) = tunn.encapsulate(&p, &mut out) {
                    udp.send_to(d, addr).await.unwrap();
                }
            }
        }

        tokio::select! {
            r = udp.recv_from(&mut inc) => {
                let (n, from) = r.unwrap();
                peer = Some(from);
                match tunn.decapsulate(Some(from.ip()), &inc[..n], &mut out) {
                    TunnResult::WriteToNetwork(d) => {
                        udp.send_to(d, from).await.unwrap();
                        while let TunnResult::WriteToNetwork(d) = tunn.decapsulate(None, &[], &mut out) {
                            udp.send_to(d, from).await.unwrap();
                        }
                    }
                    TunnResult::WriteToTunnelV4(p, _) | TunnResult::WriteToTunnelV6(p, _) => {
                        dev.rx.push_back(p.to_vec());
                    }
                    _ => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(5)) => {}
        }
    }
}

fn client_config(client: &StaticSecret, server: &PublicKey, endpoint: SocketAddr) -> WgConfig {
    format!(
        "[Interface]\nPrivateKey = {}\nAddress = 10.8.0.2/32\n\n\
         [Peer]\nPublicKey = {}\nAllowedIPs = 0.0.0.0/0\nEndpoint = {}\n",
        BASE64.encode(client.to_bytes()),
        BASE64.encode(server.as_bytes()),
        endpoint
    )
    .parse()
    .unwrap()
}

async fn socks_connect(proxy: SocketAddr, target: [u8; 4], port: u16) -> (TcpStream, u8) {
    let mut s = TcpStream::connect(proxy).await.unwrap();
    s.write_all(&[5, 1, 0]).await.unwrap();
    let mut greet = [0u8; 2];
    s.read_exact(&mut greet).await.unwrap();
    assert_eq!(greet, [5, 0]);
    let mut req = vec![5, 1, 0, 1];
    req.extend_from_slice(&target);
    req.extend_from_slice(&port.to_be_bytes());
    s.write_all(&req).await.unwrap();
    let mut rep = [0u8; 10];
    s.read_exact(&mut rep).await.unwrap();
    (s, rep[1])
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn echo_through_wireguard_tunnel() {
    let server_secret = StaticSecret::from([7u8; 32]);
    let server_public = PublicKey::from(&server_secret);
    let client_secret = StaticSecret::from([9u8; 32]);
    let client_public = PublicKey::from(&client_secret);

    let udp = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let endpoint = udp.local_addr().unwrap();
    tokio::spawn(run_server(udp, server_secret, client_public));

    let tunnel = Tunnel::start(&client_config(&client_secret, &server_public, endpoint))
        .await
        .unwrap();
    assert!(tunnel.socks_addr().ip().is_loopback());

    tokio::time::timeout(Duration::from_secs(15), async {
        let (mut s, code) = socks_connect(tunnel.socks_addr(), [10, 8, 0, 1], ECHO_PORT).await;
        assert_eq!(code, 0, "SOCKS5-Verbindung durch den Tunnel fehlgeschlagen");

        // Mehr als ein Paket, damit auch Segmentierung durch den Tunnel geprüft wird.
        let message: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        s.write_all(&message).await.unwrap();
        let mut echo = vec![0u8; message.len()];
        s.read_exact(&mut echo).await.unwrap();
        assert_eq!(echo, message);
    })
    .await
    .expect("Zeitüberschreitung im Tunnel-Test");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn domain_targets_are_refused_until_dns_is_tunneled() {
    let server_secret = StaticSecret::from([3u8; 32]);
    let client_secret = StaticSecret::from([4u8; 32]);
    let endpoint: SocketAddr = "127.0.0.1:9".parse().unwrap();
    let tunnel = Tunnel::start(&client_config(
        &client_secret,
        &PublicKey::from(&server_secret),
        endpoint,
    ))
    .await
    .unwrap();

    let mut s = TcpStream::connect(tunnel.socks_addr()).await.unwrap();
    s.write_all(&[5, 1, 0]).await.unwrap();
    let mut greet = [0u8; 2];
    s.read_exact(&mut greet).await.unwrap();
    let mut req = vec![5, 1, 0, 3, 11];
    req.extend_from_slice(b"example.com");
    req.extend_from_slice(&80u16.to_be_bytes());
    s.write_all(&req).await.unwrap();
    let mut rep = [0u8; 10];
    s.read_exact(&mut rep).await.unwrap();
    assert_eq!(rep[1], 8, "Domainnamen müssen bis T-404 abgelehnt werden");
}
