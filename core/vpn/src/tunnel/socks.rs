//! Minimaler SOCKS5-Server (RFC 1928): nur `CONNECT`, nur ohne Anmeldung.
//!
//! Domainnamen werden an den Tunnel weitergereicht und dort über den DNS-Server
//! der VPN-Konfiguration aufgelöst, damit Namensauflösung nie am VPN vorbei läuft.

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const VERSION: u8 = 5;

/// Antwortcodes aus RFC 1928 §6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reply {
    Succeeded = 0,
    GeneralFailure = 1,
    ConnectionNotAllowed = 2,
    NetworkUnreachable = 3,
    HostUnreachable = 4,
    ConnectionRefused = 5,
    CommandNotSupported = 7,
    AddressTypeNotSupported = 8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Connect(SocketAddr),
    /// Domainname, wird im Tunnel aufgelöst.
    ConnectDomain(String, u16),
}

impl Reply {
    /// Antwortcode eines anderen SOCKS5-Servers übernehmen (unbekannte → allgemeiner Fehler).
    pub fn from_code(code: u8) -> Reply {
        match code {
            0 => Reply::Succeeded,
            2 => Reply::ConnectionNotAllowed,
            3 => Reply::NetworkUnreachable,
            4 => Reply::HostUnreachable,
            5 => Reply::ConnectionRefused,
            7 => Reply::CommandNotSupported,
            8 => Reply::AddressTypeNotSupported,
            _ => Reply::GeneralFailure,
        }
    }
}

/// Liest Begrüßung und Anfrage. Bei ungültigen Anfragen wird die passende
/// Fehlerantwort schon geschickt und `Ok(None)` zurückgegeben.
pub async fn handshake<S>(stream: &mut S) -> io::Result<Option<Request>>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // Begrüßung: VER, NMETHODS, METHODS…
    let mut head = [0u8; 2];
    stream.read_exact(&mut head).await?;
    if head[0] != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "kein SOCKS5-Client",
        ));
    }
    let mut methods = vec![0u8; head[1] as usize];
    stream.read_exact(&mut methods).await?;
    if !methods.contains(&0x00) {
        stream.write_all(&[VERSION, 0xFF]).await?;
        return Ok(None);
    }
    stream.write_all(&[VERSION, 0x00]).await?;

    // Anfrage: VER, CMD, RSV, ATYP, DST.ADDR, DST.PORT
    let mut req = [0u8; 4];
    stream.read_exact(&mut req).await?;
    if req[0] != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "falsche Version",
        ));
    }
    let target = match req[3] {
        0x01 => {
            let mut a = [0u8; 4];
            stream.read_exact(&mut a).await?;
            Request::Connect(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from(a)),
                stream.read_u16().await?,
            ))
        }
        0x04 => {
            let mut a = [0u8; 16];
            stream.read_exact(&mut a).await?;
            Request::Connect(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::from(a)),
                stream.read_u16().await?,
            ))
        }
        0x03 => {
            let len = stream.read_u8().await? as usize;
            let mut name = vec![0u8; len];
            stream.read_exact(&mut name).await?;
            let port = stream.read_u16().await?;
            Request::ConnectDomain(String::from_utf8_lossy(&name).into_owned(), port)
        }
        _ => {
            reply(stream, Reply::AddressTypeNotSupported).await?;
            return Ok(None);
        }
    };
    if req[1] != 0x01 {
        reply(stream, Reply::CommandNotSupported).await?;
        return Ok(None);
    }
    Ok(Some(target))
}

/// Schickt die Antwort auf eine Anfrage (gebundene Adresse immer 0.0.0.0:0).
pub async fn reply<S>(stream: &mut S, code: Reply) -> io::Result<()>
where
    S: AsyncWriteExt + Unpin,
{
    stream
        .write_all(&[VERSION, code as u8, 0, 0x01, 0, 0, 0, 0, 0, 0])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    async fn run(client_bytes: &[u8]) -> (io::Result<Option<Request>>, Vec<u8>) {
        let (mut client, mut server) = duplex(1024);
        client.write_all(client_bytes).await.unwrap();
        let result = handshake(&mut server).await;
        drop(server);
        let mut answer = Vec::new();
        client.read_to_end(&mut answer).await.unwrap();
        (result, answer)
    }

    #[tokio::test]
    async fn connect_ipv4() {
        let (r, answer) = run(&[5, 1, 0, 5, 1, 0, 1, 10, 0, 0, 1, 0x1F, 0x90]).await;
        assert_eq!(
            r.unwrap(),
            Some(Request::Connect("10.0.0.1:8080".parse().unwrap()))
        );
        assert_eq!(answer, vec![5, 0]);
    }

    #[tokio::test]
    async fn connect_ipv6_and_domain() {
        let mut msg = vec![5, 1, 0, 5, 1, 0, 4];
        msg.extend_from_slice(&"2001:db8::1".parse::<Ipv6Addr>().unwrap().octets());
        msg.extend_from_slice(&443u16.to_be_bytes());
        let (r, _) = run(&msg).await;
        assert_eq!(
            r.unwrap(),
            Some(Request::Connect("[2001:db8::1]:443".parse().unwrap()))
        );

        let mut msg = vec![5, 1, 0, 5, 1, 0, 3, 11];
        msg.extend_from_slice(b"example.com");
        msg.extend_from_slice(&80u16.to_be_bytes());
        let (r, _) = run(&msg).await;
        assert_eq!(
            r.unwrap(),
            Some(Request::ConnectDomain("example.com".into(), 80))
        );
    }

    #[tokio::test]
    async fn rejects_auth_only_clients() {
        let (r, answer) = run(&[5, 1, 2]).await;
        assert_eq!(r.unwrap(), None);
        assert_eq!(answer, vec![5, 0xFF]);
    }

    #[tokio::test]
    async fn rejects_bind_and_udp() {
        let (r, answer) = run(&[5, 1, 0, 5, 2, 0, 1, 1, 2, 3, 4, 0, 80]).await;
        assert_eq!(r.unwrap(), None);
        assert_eq!(answer[2..4], [5, Reply::CommandNotSupported as u8]);
    }

    #[tokio::test]
    async fn rejects_socks4() {
        let (r, _) = run(&[4, 1, 0, 80]).await;
        assert!(r.is_err());
    }
}
