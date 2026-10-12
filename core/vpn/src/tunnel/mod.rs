//! WireGuard-Tunnel im Userspace mit lokalem SOCKS5-Proxy (T-402).
//!
//! ```text
//! WebView ──SOCKS5──► 127.0.0.1:<port> ──► smoltcp (TCP/IP) ──► boringtun ──UDP──► VPN-Server
//! ```
//!
//! Es werden keine Admin-Rechte und kein TUN-Gerät gebraucht: smoltcp baut die
//! TCP-Verbindungen selbst, boringtun verschlüsselt die IP-Pakete. Nur der Verkehr,
//! der über den Proxy läuft, geht durch das VPN (Browser-VPN, siehe docs/VPN.md).
//!
//! Noch nicht enthalten: Domainnamen/DNS über den Tunnel (T-404) und das Anbinden an
//! die WebViews samt Kill-Switch (T-403).

mod device;
pub mod socks;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use boringtun::noise::{Tunn, TunnResult};
use boringtun::x25519::{PublicKey, StaticSecret};
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::socket::tcp;
use smoltcp::time::Instant as SmolInstant;
use smoltcp::wire::{HardwareAddress, IpAddress, IpCidr};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::wgconfig::WgConfig;
use device::VirtualDevice;
use socks::{Reply, Request};

/// WireGuard erwartet, dass die Timer etwa alle 250 ms geprüft werden.
const TIMER_TICK: Duration = Duration::from_millis(250);
/// Standard-MTU von WireGuard.
const DEFAULT_MTU: usize = 1420;
/// Puffer pro TCP-Verbindung (je Richtung).
const TCP_BUFFER: usize = 256 * 1024;
/// So lange darf ein Verbindungsaufbau durch den Tunnel dauern.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
/// Größter UDP-Datagramm plus WireGuard-Overhead.
const PACKET_BUFFER: usize = 65_536;

#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("Die Konfiguration hat keinen [Peer] mit Endpoint")]
    NoPeer,
    #[error("VPN-Server {0} konnte nicht aufgelöst werden")]
    Resolve(String),
    #[error("Netzwerkfehler: {0}")]
    Io(#[from] std::io::Error),
}

/// Ein laufender Tunnel. Wird er fallen gelassen, stoppt er.
pub struct Tunnel {
    socks_addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    tasks: Vec<JoinHandle<()>>,
}

impl Tunnel {
    /// Baut den Tunnel nach `config` auf und startet den SOCKS5-Proxy auf
    /// `127.0.0.1` mit einem zufälligen freien Port.
    ///
    /// Bei mehreren `[Peer]`-Abschnitten wird der erste verwendet.
    pub async fn start(config: &WgConfig) -> Result<Tunnel, TunnelError> {
        let peer = config.peers.first().ok_or(TunnelError::NoPeer)?;
        let endpoint = peer.endpoint.as_ref().ok_or(TunnelError::NoPeer)?;
        let server = tokio::net::lookup_host((endpoint.host.as_str(), endpoint.port))
            .await?
            .next()
            .ok_or_else(|| TunnelError::Resolve(endpoint.host.clone()))?;

        let udp = UdpSocket::bind(if server.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        })
        .await?;
        udp.connect(server).await?;

        let tunn = Tunn::new(
            StaticSecret::from(*config.interface.private_key.as_bytes()),
            PublicKey::from(*peer.public_key.as_bytes()),
            peer.preshared_key.as_ref().map(|k| *k.as_bytes()),
            peer.persistent_keepalive,
            session_index(),
            None,
        );

        let mtu = config.interface.mtu.map(usize::from).unwrap_or(DEFAULT_MTU);
        let mut device = VirtualDevice::new(mtu);
        let mut iface_config = Config::new(HardwareAddress::Ip);
        iface_config.random_seed = u64::from(session_index());
        let mut iface = Interface::new(iface_config, &mut device, SmolInstant::from_millis(0));
        iface.update_ip_addrs(|addrs| {
            for net in &config.interface.addresses {
                let _ = addrs.push(IpCidr::new(IpAddress::from(net.addr), net.prefix));
            }
        });
        // Alles, was nicht im eigenen Netz liegt, geht durch den Tunnel.
        for net in &config.interface.addresses {
            match net.addr {
                IpAddr::V4(a) => {
                    let _ = iface.routes_mut().add_default_ipv4_route(a);
                }
                IpAddr::V6(a) => {
                    let _ = iface.routes_mut().add_default_ipv6_route(a);
                }
            }
        }

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let socks_addr = listener.local_addr()?;
        let (msg_tx, msg_rx) = mpsc::channel(256);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let engine = Engine {
            tunn,
            udp,
            device,
            iface,
            sockets: SocketSet::new(Vec::new()),
            conns: HashMap::new(),
            next_id: 1,
            next_port: 49_152,
            started: std::time::Instant::now(),
        };
        let engine_task = tokio::spawn(engine.run(msg_rx, shutdown_rx));
        let accept_task = tokio::spawn(accept_loop(listener, msg_tx));

        Ok(Tunnel {
            socks_addr,
            shutdown: Some(shutdown_tx),
            tasks: vec![engine_task, accept_task],
        })
    }

    /// Adresse des SOCKS5-Proxys, z. B. für `socks5://127.0.0.1:PORT`.
    pub fn socks_addr(&self) -> SocketAddr {
        self.socks_addr
    }

    /// Stoppt den Tunnel und alle Verbindungen.
    pub fn stop(mut self) {
        self.shutdown_now();
    }

    fn shutdown_now(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        for task in &self.tasks {
            task.abort();
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.shutdown_now();
    }
}

/// Zufälliger Sitzungsindex für WireGuard (boringtun nutzt die unteren 24 Bit).
fn session_index() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() ^ (d.as_secs() as u32))
        .unwrap_or(1);
    nanos & 0x00FF_FFFF
}

enum Msg {
    Connect {
        target: SocketAddr,
        to_client: mpsc::Sender<Vec<u8>>,
        ready: oneshot::Sender<Result<u64, Reply>>,
    },
    Data(u64, Vec<u8>),
    ClientClosed(u64),
}

async fn accept_loop(listener: TcpListener, msgs: mpsc::Sender<Msg>) {
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(serve_client(stream, msgs.clone()));
    }
}

async fn serve_client(mut stream: TcpStream, msgs: mpsc::Sender<Msg>) {
    let _ = stream.set_nodelay(true);
    let target = match socks::handshake(&mut stream).await {
        Ok(Some(Request::Connect(addr))) => addr,
        Ok(Some(Request::ConnectDomain(..))) => {
            let _ = socks::reply(&mut stream, Reply::AddressTypeNotSupported).await;
            return;
        }
        _ => return,
    };

    let (to_client, mut from_tunnel) = mpsc::channel::<Vec<u8>>(64);
    let (ready_tx, ready_rx) = oneshot::channel();
    let msg = Msg::Connect {
        target,
        to_client,
        ready: ready_tx,
    };
    if msgs.send(msg).await.is_err() {
        let _ = socks::reply(&mut stream, Reply::GeneralFailure).await;
        return;
    }
    let id = match ready_rx.await {
        Ok(Ok(id)) => id,
        Ok(Err(code)) => {
            let _ = socks::reply(&mut stream, code).await;
            return;
        }
        Err(_) => {
            let _ = socks::reply(&mut stream, Reply::GeneralFailure).await;
            return;
        }
    };
    if socks::reply(&mut stream, Reply::Succeeded).await.is_err() {
        let _ = msgs.send(Msg::ClientClosed(id)).await;
        return;
    }

    let (mut rd, mut wr) = stream.into_split();
    let upload = async {
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            match rd.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if msgs.send(Msg::Data(id, buf[..n].to_vec())).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = msgs.send(Msg::ClientClosed(id)).await;
    };
    let download = async {
        while let Some(data) = from_tunnel.recv().await {
            if wr.write_all(&data).await.is_err() {
                break;
            }
        }
        let _ = wr.shutdown().await;
    };
    tokio::join!(upload, download);
}

struct Conn {
    handle: SocketHandle,
    /// `None`, sobald die Gegenseite fertig gesendet hat.
    to_client: Option<mpsc::Sender<Vec<u8>>>,
    /// Daten vom Client, die noch nicht in den TCP-Puffer passten.
    pending: Vec<u8>,
    ready: Option<oneshot::Sender<Result<u64, Reply>>>,
    client_closed: bool,
}

struct Engine {
    tunn: Tunn,
    udp: UdpSocket,
    device: VirtualDevice,
    iface: Interface,
    sockets: SocketSet<'static>,
    conns: HashMap<u64, Conn>,
    next_id: u64,
    next_port: u16,
    started: std::time::Instant,
}

impl Engine {
    fn now(&self) -> SmolInstant {
        SmolInstant::from_millis(self.started.elapsed().as_millis() as i64)
    }

    async fn run(mut self, mut msgs: mpsc::Receiver<Msg>, mut shutdown: oneshot::Receiver<()>) {
        let mut out = vec![0u8; PACKET_BUFFER];
        let mut inc = vec![0u8; PACKET_BUFFER];
        let mut last_timers = std::time::Instant::now();

        // Handshake sofort starten, damit die erste Verbindung nicht warten muss.
        if let TunnResult::WriteToNetwork(p) =
            self.tunn.format_handshake_initiation(&mut out, false)
        {
            let _ = self.udp.send(p).await;
        }

        loop {
            self.pump();
            self.flush_outbound(&mut out).await;

            let delay = self
                .iface
                .poll_delay(self.now(), &self.sockets)
                .map(|d| Duration::from_micros(d.total_micros()))
                .unwrap_or(TIMER_TICK)
                .min(TIMER_TICK);

            tokio::select! {
                _ = &mut shutdown => break,
                received = self.udp.recv(&mut inc) => {
                    if let Ok(n) = received {
                        self.handle_datagram(&inc[..n], &mut out).await;
                    }
                }
                msg = msgs.recv() => match msg {
                    Some(msg) => self.handle_msg(msg),
                    None => break,
                },
                _ = tokio::time::sleep(delay) => {}
            }

            if last_timers.elapsed() >= TIMER_TICK {
                last_timers = std::time::Instant::now();
                if let TunnResult::WriteToNetwork(p) = self.tunn.update_timers(&mut out) {
                    let _ = self.udp.send(p).await;
                }
            }
        }
    }

    async fn handle_datagram(&mut self, datagram: &[u8], out: &mut [u8]) {
        match self.tunn.decapsulate(None, datagram, out) {
            TunnResult::WriteToNetwork(p) => {
                let _ = self.udp.send(p).await;
                // Nach dem Handshake wartende Pakete hinterherschicken.
                while let TunnResult::WriteToNetwork(p) = self.tunn.decapsulate(None, &[], out) {
                    let _ = self.udp.send(p).await;
                }
            }
            TunnResult::WriteToTunnelV4(p, _) | TunnResult::WriteToTunnelV6(p, _) => {
                self.device.inbound.push_back(p.to_vec());
            }
            TunnResult::Done | TunnResult::Err(_) => {}
        }
    }

    async fn flush_outbound(&mut self, out: &mut [u8]) {
        let packets: Vec<Vec<u8>> = self.device.outbound.drain(..).collect();
        for packet in packets {
            if let TunnResult::WriteToNetwork(p) = self.tunn.encapsulate(&packet, out) {
                let _ = self.udp.send(p).await;
            }
        }
    }

    fn handle_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Connect {
                target,
                to_client,
                ready,
            } => {
                let mut socket = tcp::Socket::new(
                    tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
                    tcp::SocketBuffer::new(vec![0; TCP_BUFFER]),
                );
                socket.set_timeout(Some(CONNECT_TIMEOUT.into()));
                socket.set_nagle_enabled(false);
                let port = self.next_port;
                self.next_port = if port == u16::MAX { 49_152 } else { port + 1 };
                if socket.connect(self.iface.context(), target, port).is_err() {
                    let _ = ready.send(Err(Reply::NetworkUnreachable));
                    return;
                }
                let id = self.next_id;
                self.next_id += 1;
                let handle = self.sockets.add(socket);
                self.conns.insert(
                    id,
                    Conn {
                        handle,
                        to_client: Some(to_client),
                        pending: Vec::new(),
                        ready: Some(ready),
                        client_closed: false,
                    },
                );
            }
            Msg::Data(id, data) => {
                if let Some(c) = self.conns.get_mut(&id) {
                    c.pending.extend_from_slice(&data);
                }
            }
            Msg::ClientClosed(id) => {
                if let Some(c) = self.conns.get_mut(&id) {
                    c.client_closed = true;
                }
            }
        }
    }

    /// Lässt smoltcp arbeiten und schiebt Daten zwischen TCP-Sockets und Clients hin und her.
    fn pump(&mut self) {
        let now = self.now();
        self.iface.poll(now, &mut self.device, &mut self.sockets);

        let mut finished = Vec::new();
        for (&id, c) in self.conns.iter_mut() {
            let s = self.sockets.get_mut::<tcp::Socket>(c.handle);

            if c.ready.is_some() {
                match s.state() {
                    tcp::State::Established => {
                        let _ = c.ready.take().map(|r| r.send(Ok(id)));
                    }
                    tcp::State::Closed | tcp::State::TimeWait => {
                        let _ = c.ready.take().map(|r| r.send(Err(Reply::HostUnreachable)));
                        finished.push(id);
                        continue;
                    }
                    _ => continue,
                }
            }

            // Tunnel → Client
            if let Some(tx) = &c.to_client {
                if tx.is_closed() {
                    s.abort();
                    c.to_client = None;
                } else {
                    while s.can_recv() && tx.capacity() > 0 {
                        let mut buf = vec![0u8; 16 * 1024];
                        match s.recv_slice(&mut buf) {
                            Ok(n) if n > 0 => {
                                buf.truncate(n);
                                let _ = tx.try_send(buf);
                            }
                            _ => break,
                        }
                    }
                    if !s.may_recv() && !s.can_recv() {
                        c.to_client = None; // Gegenseite fertig → Client bekommt EOF
                    }
                }
            }

            // Client → Tunnel
            if !c.pending.is_empty() && s.can_send() {
                if let Ok(n) = s.send_slice(&c.pending) {
                    c.pending.drain(..n);
                }
            }
            if c.client_closed && c.pending.is_empty() && s.may_send() {
                s.close();
            }

            if !s.is_open() {
                finished.push(id);
            }
        }

        for id in finished {
            if let Some(c) = self.conns.remove(&id) {
                self.sockets.remove(c.handle);
            }
        }

        self.iface.poll(now, &mut self.device, &mut self.sockets);
    }
}
