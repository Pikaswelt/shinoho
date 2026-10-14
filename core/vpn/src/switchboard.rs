//! Die „Weiche“ (T-403): ein fester lokaler SOCKS5-Proxy, über den **alle** WebViews laufen.
//!
//! ```text
//! WebView ──socks5──► Weiche (127.0.0.1:fest) ──┬─ Direkt   → Internet (VPN aus)
//!                                               ├─ Tunnel   → Tunnel-SOCKS → WireGuard (VPN an)
//!                                               └─ Gesperrt → Verbindung abgelehnt (Kill-Switch)
//! ```
//!
//! Weil die WebViews immer dieselbe Proxy-Adresse haben, wirkt ein Umschalten sofort und ohne
//! dass Tabs neu erzeugt werden müssen. Beim Umschalten werden alle offenen Verbindungen
//! getrennt, damit nichts aus dem alten Modus weiterläuft (z. B. nach einem VPN-Abbruch).

use std::io;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::limiter::RateLimiter;
use crate::tunnel::socks::{self, Reply, Request};

/// Wohin die Weiche Verbindungen leitet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Ohne VPN direkt ins Internet.
    Direct,
    /// Durch den VPN-Tunnel (Adresse des Tunnel-SOCKS-Proxys).
    Tunnel(SocketAddr),
    /// Alles ablehnen (Kill-Switch: VPN soll an sein, ist aber nicht verbunden).
    Blocked,
}

/// Laufende Weiche. Wird sie fallen gelassen, stoppt sie.
pub struct Switchboard {
    addr: SocketAddr,
    route: watch::Sender<Route>,
    limiter: RateLimiter,
    task: JoinHandle<()>,
}

impl Switchboard {
    /// Startet die Weiche auf `127.0.0.1` mit zufälligem freiem Port.
    pub async fn start(initial: Route) -> io::Result<Switchboard> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let (route, rx) = watch::channel(initial);
        let limiter = RateLimiter::unlimited();
        let task = tokio::spawn(accept_loop(listener, rx, limiter.clone()));
        Ok(Switchboard {
            addr,
            route,
            limiter,
            task,
        })
    }

    /// Adresse für die WebViews, z. B. `socks5://127.0.0.1:PORT`.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Proxy-URL für `WebviewBuilder::proxy_url`.
    pub fn proxy_url(&self) -> String {
        format!("socks5://{}", self.addr)
    }

    pub fn route(&self) -> Route {
        *self.route.borrow()
    }

    /// Bandbreiten-Begrenzer für alle Tabs zusammen (T-204).
    pub fn limiter(&self) -> &RateLimiter {
        &self.limiter
    }

    /// Schaltet um. Offene Verbindungen werden dabei getrennt.
    pub fn set_route(&self, route: Route) {
        self.route.send_replace(route);
    }
}

impl Drop for Switchboard {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn accept_loop(listener: TcpListener, route: watch::Receiver<Route>, limiter: RateLimiter) {
    while let Ok((stream, _)) = listener.accept().await {
        let mut route = route.clone();
        route.mark_unchanged();
        let limiter = limiter.clone();
        tokio::spawn(async move {
            let _ = serve(stream, &mut route, &limiter).await;
        });
    }
}

async fn serve(
    mut client: TcpStream,
    route: &mut watch::Receiver<Route>,
    limiter: &RateLimiter,
) -> io::Result<()> {
    let _ = client.set_nodelay(true);
    let Some(request) = socks::handshake(&mut client).await? else {
        return Ok(());
    };
    let current = *route.borrow_and_update();

    let upstream = match current {
        Route::Blocked => {
            socks::reply(&mut client, Reply::ConnectionNotAllowed).await?;
            return Ok(());
        }
        Route::Direct => match connect_direct(&request).await {
            Ok(s) => s,
            Err(_) => {
                socks::reply(&mut client, Reply::HostUnreachable).await?;
                return Ok(());
            }
        },
        Route::Tunnel(tunnel) => match connect_via_socks(tunnel, &request).await {
            Ok(s) => s,
            Err(code) => {
                socks::reply(&mut client, code).await?;
                return Ok(());
            }
        },
    };
    socks::reply(&mut client, Reply::Succeeded).await?;

    let mut upstream = upstream;
    let (mut cr, mut cw) = client.split();
    let (mut ur, mut uw) = upstream.split();
    let relay = async {
        tokio::join!(
            limiter.copy(&mut cr, &mut uw),
            limiter.copy(&mut ur, &mut cw)
        )
    };
    tokio::select! {
        _ = relay => {}
        // Modus gewechselt: Verbindung trennen, damit nichts am neuen Modus vorbei läuft.
        _ = route.changed() => {}
    }
    Ok(())
}

async fn connect_direct(request: &Request) -> io::Result<TcpStream> {
    let stream = match request {
        Request::Connect(addr) => TcpStream::connect(addr).await?,
        Request::ConnectDomain(host, port) => TcpStream::connect((host.as_str(), *port)).await?,
    };
    let _ = stream.set_nodelay(true);
    Ok(stream)
}

/// Baut über einen anderen SOCKS5-Proxy (den Tunnel) eine Verbindung zum Ziel auf.
async fn connect_via_socks(proxy: SocketAddr, request: &Request) -> Result<TcpStream, Reply> {
    let fail = |_| Reply::GeneralFailure;
    let mut s = TcpStream::connect(proxy).await.map_err(fail)?;
    let _ = s.set_nodelay(true);
    s.write_all(&[socks::VERSION, 1, 0]).await.map_err(fail)?;
    let mut greet = [0u8; 2];
    s.read_exact(&mut greet).await.map_err(fail)?;
    if greet != [socks::VERSION, 0] {
        return Err(Reply::GeneralFailure);
    }

    let mut req = vec![socks::VERSION, 1, 0];
    match request {
        Request::Connect(SocketAddr::V4(a)) => {
            req.push(1);
            req.extend_from_slice(&a.ip().octets());
            req.extend_from_slice(&a.port().to_be_bytes());
        }
        Request::Connect(SocketAddr::V6(a)) => {
            req.push(4);
            req.extend_from_slice(&a.ip().octets());
            req.extend_from_slice(&a.port().to_be_bytes());
        }
        Request::ConnectDomain(host, port) => {
            let name = host.as_bytes();
            if name.len() > 255 {
                return Err(Reply::GeneralFailure);
            }
            req.push(3);
            req.push(name.len() as u8);
            req.extend_from_slice(name);
            req.extend_from_slice(&port.to_be_bytes());
        }
    }
    s.write_all(&req).await.map_err(fail)?;

    // Antwort: VER REP RSV ATYP BND.ADDR BND.PORT
    let mut head = [0u8; 4];
    s.read_exact(&mut head).await.map_err(fail)?;
    let skip = match head[3] {
        1 => 4 + 2,
        4 => 16 + 2,
        3 => s.read_u8().await.map_err(fail)? as usize + 2,
        _ => return Err(Reply::GeneralFailure),
    };
    let mut rest = vec![0u8; skip];
    s.read_exact(&mut rest).await.map_err(fail)?;
    if head[1] != 0 {
        return Err(Reply::from_code(head[1]));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    async fn echo_server() -> SocketAddr {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut s, _)) = l.accept().await {
                tokio::spawn(async move {
                    let (mut r, mut w) = s.split();
                    let _ = tokio::io::copy(&mut r, &mut w).await;
                });
            }
        });
        addr
    }

    /// Minimaler Upstream-SOCKS5 (Ersatz für den Tunnel), leitet zu `real` weiter und zählt.
    async fn fake_tunnel(real: SocketAddr) -> (SocketAddr, tokio::sync::mpsc::Receiver<Request>) {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        tokio::spawn(async move {
            while let Ok((mut s, _)) = l.accept().await {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(Some(req)) = socks::handshake(&mut s).await {
                        let _ = tx.send(req).await;
                        let mut up = TcpStream::connect(real).await.unwrap();
                        socks::reply(&mut s, Reply::Succeeded).await.unwrap();
                        let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await;
                    }
                });
            }
        });
        (addr, rx)
    }

    async fn socks_connect(proxy: SocketAddr, target: SocketAddr) -> (TcpStream, u8) {
        let mut s = TcpStream::connect(proxy).await.unwrap();
        s.write_all(&[5, 1, 0]).await.unwrap();
        let mut g = [0u8; 2];
        s.read_exact(&mut g).await.unwrap();
        let SocketAddr::V4(t) = target else {
            unreachable!()
        };
        let mut req = vec![5, 1, 0, 1];
        req.extend_from_slice(&t.ip().octets());
        req.extend_from_slice(&t.port().to_be_bytes());
        s.write_all(&req).await.unwrap();
        let mut rep = [0u8; 10];
        s.read_exact(&mut rep).await.unwrap();
        (s, rep[1])
    }

    async fn roundtrip(s: &mut TcpStream) -> bool {
        if s.write_all(b"shinoho").await.is_err() {
            return false;
        }
        let mut buf = [0u8; 7];
        matches!(
            tokio::time::timeout(Duration::from_secs(2), s.read_exact(&mut buf)).await,
            Ok(Ok(_))
        ) && &buf == b"shinoho"
    }

    #[tokio::test]
    async fn direct_mode_connects_to_target() {
        let echo = echo_server().await;
        let sb = Switchboard::start(Route::Direct).await.unwrap();
        assert!(sb.proxy_url().starts_with("socks5://127.0.0.1:"));
        let (mut s, code) = socks_connect(sb.addr(), echo).await;
        assert_eq!(code, 0);
        assert!(roundtrip(&mut s).await);
    }

    #[tokio::test]
    async fn blocked_mode_refuses_everything() {
        let echo = echo_server().await;
        let sb = Switchboard::start(Route::Blocked).await.unwrap();
        let (_, code) = socks_connect(sb.addr(), echo).await;
        assert_eq!(code, Reply::ConnectionNotAllowed as u8);
    }

    #[tokio::test]
    async fn tunnel_mode_goes_through_upstream() {
        let echo = echo_server().await;
        let (tunnel, mut seen) = fake_tunnel(echo).await;
        let sb = Switchboard::start(Route::Tunnel(tunnel)).await.unwrap();
        let (mut s, code) = socks_connect(sb.addr(), echo).await;
        assert_eq!(code, 0);
        assert!(roundtrip(&mut s).await);
        assert_eq!(seen.recv().await, Some(Request::Connect(echo)));
    }

    #[tokio::test]
    async fn bandwidth_limit_applies_to_proxied_traffic() {
        let echo = echo_server().await;
        let sb = Switchboard::start(Route::Direct).await.unwrap();
        sb.limiter().set_limit_kbps(Some(800)); // 100 kB/s je Richtung insgesamt
        let (mut s, _) = socks_connect(sb.addr(), echo).await;
        let data = vec![1u8; 40_000];
        let start = std::time::Instant::now();
        let (mut r, mut w) = s.split();
        let send = async { w.write_all(&data).await.unwrap() };
        let recv = async {
            let mut got = vec![0u8; data.len()];
            r.read_exact(&mut got).await.unwrap();
        };
        tokio::join!(send, recv);
        // 40 kB hin + 40 kB zurück teilen sich 100 kB/s ≈ 0,8 s
        assert!(start.elapsed() > Duration::from_millis(400));
    }

    #[tokio::test]
    async fn switching_route_cuts_open_connections() {
        let echo = echo_server().await;
        let sb = Switchboard::start(Route::Direct).await.unwrap();
        let (mut s, _) = socks_connect(sb.addr(), echo).await;
        assert!(roundtrip(&mut s).await);

        // VPN fällt aus → Kill-Switch: alte Verbindung muss getrennt werden …
        sb.set_route(Route::Blocked);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !roundtrip(&mut s).await,
            "Verbindung lief am Kill-Switch vorbei"
        );
        // … und neue werden abgelehnt.
        let (_, code) = socks_connect(sb.addr(), echo).await;
        assert_eq!(code, Reply::ConnectionNotAllowed as u8);
        assert_eq!(sb.route(), Route::Blocked);
    }
}
