//! Parser für WireGuard-Konfigurationsdateien (`.conf`), wie Anbieter sie zum
//! Herunterladen anbieten.
//!
//! Unterstützt `[Interface]` und beliebig viele `[Peer]`-Abschnitte. Unbekannte
//! Schlüssel, die nur `wg-quick` betreffen (`PostUp`, `Table`, …), werden ignoriert,
//! weil Shinoho den Tunnel selbst im Userspace betreibt.

use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use thiserror::Error;

/// Ein 32-Byte-WireGuard-Schlüssel (Curve25519 bzw. Preshared Key).
///
/// `Debug` gibt den Inhalt absichtlich nicht aus, damit private Schlüssel nie in
/// Logs landen.
#[derive(Clone, PartialEq, Eq)]
pub struct Key([u8; 32]);

impl Key {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        BASE64.encode(self.0)
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key(<verborgen>)")
    }
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = BASE64.decode(s.trim()).map_err(|_| ())?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| ())?;
        Ok(Key(bytes))
    }
}

/// Ziel eines Peers: Hostname oder IP plus Port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub host: String,
    pub port: u16,
}

impl FromStr for Endpoint {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (host, port) = if let Some(rest) = s.strip_prefix('[') {
            // IPv6: [2001:db8::1]:51820
            let (host, port) = rest.split_once("]:").ok_or(())?;
            host.parse::<std::net::Ipv6Addr>().map_err(|_| ())?;
            (host, port)
        } else {
            let (host, port) = s.rsplit_once(':').ok_or(())?;
            if host.contains(':') {
                return Err(()); // IPv6 ohne eckige Klammern ist mehrdeutig
            }
            (host, port)
        };
        if host.is_empty() {
            return Err(());
        }
        let port = port.parse::<u16>().map_err(|_| ())?;
        if port == 0 {
            return Err(());
        }
        Ok(Endpoint {
            host: host.to_string(),
            port,
        })
    }
}

/// Netz in CIDR-Schreibweise, z. B. `10.2.0.2/32` oder `::/0`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpNet {
    pub addr: IpAddr,
    pub prefix: u8,
}

impl FromStr for IpNet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (addr, prefix) = match s.split_once('/') {
            Some((a, p)) => (a, Some(p)),
            None => (s, None),
        };
        let addr: IpAddr = addr.parse().map_err(|_| ())?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(p) => p.parse::<u8>().map_err(|_| ())?,
            None => max,
        };
        if prefix > max {
            return Err(());
        }
        Ok(IpNet { addr, prefix })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgInterface {
    pub private_key: Key,
    pub addresses: Vec<IpNet>,
    /// DNS-Server; Shinoho schickt DNS-Anfragen über den Tunnel an diese Server.
    pub dns: Vec<IpAddr>,
    pub mtu: Option<u16>,
    pub listen_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgPeer {
    pub public_key: Key,
    pub preshared_key: Option<Key>,
    pub endpoint: Option<Endpoint>,
    pub allowed_ips: Vec<IpNet>,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WgConfig {
    pub interface: WgInterface,
    pub peers: Vec<WgPeer>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WgConfigError {
    #[error("Zeile {line}: unbekannter Abschnitt [{name}]")]
    UnknownSection { line: usize, name: String },
    #[error("Zeile {line}: Eintrag steht außerhalb von [Interface] oder [Peer]")]
    OutsideSection { line: usize },
    #[error("Zeile {line}: erwartet `Schlüssel = Wert`")]
    Malformed { line: usize },
    #[error("Zeile {line}: ungültiger Wert für {key}")]
    InvalidValue { line: usize, key: String },
    #[error("Zeile {line}: {key} ist doppelt angegeben")]
    Duplicate { line: usize, key: String },
    #[error("[Interface] fehlt")]
    MissingInterface,
    #[error("[Interface] ist doppelt vorhanden")]
    DuplicateInterface,
    #[error("[Interface] braucht PrivateKey")]
    MissingPrivateKey,
    #[error("[Interface] braucht mindestens eine Address")]
    MissingAddress,
    #[error("Kein [Peer] vorhanden")]
    MissingPeer,
    #[error("[Peer] Nr. {index} braucht PublicKey")]
    MissingPublicKey { index: usize },
    #[error("[Peer] Nr. {index} braucht Endpoint")]
    MissingEndpoint { index: usize },
}

#[derive(Default)]
struct RawInterface {
    private_key: Option<Key>,
    addresses: Vec<IpNet>,
    dns: Vec<IpAddr>,
    mtu: Option<u16>,
    listen_port: Option<u16>,
}

#[derive(Default)]
struct RawPeer {
    public_key: Option<Key>,
    preshared_key: Option<Key>,
    endpoint: Option<Endpoint>,
    allowed_ips: Vec<IpNet>,
    persistent_keepalive: Option<u16>,
}

enum Section {
    None,
    Interface,
    Peer,
}

fn parse_list<T: FromStr>(value: &str, line: usize, key: &str) -> Result<Vec<T>, WgConfigError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| {
            v.parse().map_err(|_| WgConfigError::InvalidValue {
                line,
                key: key.to_string(),
            })
        })
        .collect()
}

fn parse_one<T: FromStr>(value: &str, line: usize, key: &str) -> Result<T, WgConfigError> {
    value
        .trim()
        .parse()
        .map_err(|_| WgConfigError::InvalidValue {
            line,
            key: key.to_string(),
        })
}

fn set_once<T>(slot: &mut Option<T>, v: T, line: usize, key: &str) -> Result<(), WgConfigError> {
    if slot.is_some() {
        return Err(WgConfigError::Duplicate {
            line,
            key: key.to_string(),
        });
    }
    *slot = Some(v);
    Ok(())
}

impl FromStr for WgConfig {
    type Err = WgConfigError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut section = Section::None;
        let mut interface: Option<RawInterface> = None;
        let mut peers: Vec<RawPeer> = Vec::new();

        for (idx, raw_line) in text.lines().enumerate() {
            let line = idx + 1;
            let content = raw_line.split('#').next().unwrap_or("").trim();
            if content.is_empty() {
                continue;
            }

            if let Some(name) = content.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
                let name = name.trim();
                section = if name.eq_ignore_ascii_case("Interface") {
                    if interface.is_some() {
                        return Err(WgConfigError::DuplicateInterface);
                    }
                    interface = Some(RawInterface::default());
                    Section::Interface
                } else if name.eq_ignore_ascii_case("Peer") {
                    peers.push(RawPeer::default());
                    Section::Peer
                } else {
                    return Err(WgConfigError::UnknownSection {
                        line,
                        name: name.to_string(),
                    });
                };
                continue;
            }

            let (key, value) = content
                .split_once('=')
                .map(|(k, v)| (k.trim(), v.trim()))
                .ok_or(WgConfigError::Malformed { line })?;
            let lkey = key.to_ascii_lowercase();

            match section {
                Section::None => return Err(WgConfigError::OutsideSection { line }),
                Section::Interface => {
                    let i = interface.as_mut().expect("Abschnitt ist gesetzt");
                    match lkey.as_str() {
                        "privatekey" => {
                            set_once(&mut i.private_key, parse_one(value, line, key)?, line, key)?
                        }
                        "address" => i.addresses.extend(parse_list(value, line, key)?),
                        // DNS darf neben IPs auch Suchdomänen enthalten; die ignorieren wir.
                        "dns" => i.dns.extend(
                            value
                                .split(',')
                                .filter_map(|v| v.trim().parse::<IpAddr>().ok()),
                        ),
                        "mtu" => set_once(&mut i.mtu, parse_one(value, line, key)?, line, key)?,
                        "listenport" => {
                            set_once(&mut i.listen_port, parse_one(value, line, key)?, line, key)?
                        }
                        _ => {} // wg-quick-spezifisch (PostUp, Table, SaveConfig, …)
                    }
                }
                Section::Peer => {
                    let p = peers.last_mut().expect("Abschnitt ist gesetzt");
                    match lkey.as_str() {
                        "publickey" => {
                            set_once(&mut p.public_key, parse_one(value, line, key)?, line, key)?
                        }
                        "presharedkey" => set_once(
                            &mut p.preshared_key,
                            parse_one(value, line, key)?,
                            line,
                            key,
                        )?,
                        "endpoint" => {
                            set_once(&mut p.endpoint, parse_one(value, line, key)?, line, key)?
                        }
                        "allowedips" => p.allowed_ips.extend(parse_list(value, line, key)?),
                        "persistentkeepalive" => {
                            let v = if value.eq_ignore_ascii_case("off") {
                                0
                            } else {
                                parse_one(value, line, key)?
                            };
                            set_once(&mut p.persistent_keepalive, v, line, key)?
                        }
                        _ => {}
                    }
                }
            }
        }

        let i = interface.ok_or(WgConfigError::MissingInterface)?;
        let interface = WgInterface {
            private_key: i.private_key.ok_or(WgConfigError::MissingPrivateKey)?,
            addresses: if i.addresses.is_empty() {
                return Err(WgConfigError::MissingAddress);
            } else {
                i.addresses
            },
            dns: i.dns,
            mtu: i.mtu,
            listen_port: i.listen_port,
        };

        if peers.is_empty() {
            return Err(WgConfigError::MissingPeer);
        }
        let peers = peers
            .into_iter()
            .enumerate()
            .map(|(n, p)| {
                let index = n + 1;
                Ok(WgPeer {
                    public_key: p
                        .public_key
                        .ok_or(WgConfigError::MissingPublicKey { index })?,
                    preshared_key: p.preshared_key,
                    // Shinoho baut den Tunnel aktiv auf, ohne Endpoint geht das nicht.
                    endpoint: Some(p.endpoint.ok_or(WgConfigError::MissingEndpoint { index })?),
                    allowed_ips: p.allowed_ips,
                    persistent_keepalive: p.persistent_keepalive.filter(|&k| k > 0),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WgConfig { interface, peers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Zufällig erzeugte Testschlüssel, nirgends in Benutzung.
    const PRIV: &str = "h/HJZHlumRydb9LYBabMv/JchSH7aNzuR81/Fcx6/Dw=";
    const PUB: &str = "FhSeS1jzvpLjzmA/edRCuRjr4vKvbgO1r2phvFPGx7M=";
    const PSK: &str = "/wxWgpQrf/TKjyp60kK7ePPh//P7z0YuW3oDmWMQDSc=";

    fn sample() -> String {
        format!(
            "# Beispiel wie von einem Anbieter heruntergeladen\n\
             [Interface]\n\
             PrivateKey = {PRIV}\n\
             Address = 10.2.0.2/32, fd00::2/128\n\
             DNS = 10.2.0.1, example.internal\n\
             PostUp = iptables -A FORWARD  # wird ignoriert\n\
             \n\
             [Peer]\n\
             PublicKey = {PUB}\n\
             PresharedKey = {PSK}\n\
             AllowedIPs = 0.0.0.0/0, ::/0\n\
             Endpoint = vpn.example.com:51820\n\
             PersistentKeepalive = 25\n"
        )
    }

    #[test]
    fn parses_typical_provider_config() {
        let cfg: WgConfig = sample().parse().unwrap();
        assert_eq!(cfg.interface.private_key.to_base64(), PRIV);
        assert_eq!(cfg.interface.addresses.len(), 2);
        assert_eq!(cfg.interface.addresses[0].prefix, 32);
        assert_eq!(
            cfg.interface.dns,
            vec!["10.2.0.1".parse::<IpAddr>().unwrap()]
        );
        assert_eq!(cfg.peers.len(), 1);
        let peer = &cfg.peers[0];
        assert_eq!(peer.public_key.to_base64(), PUB);
        assert!(peer.preshared_key.is_some());
        assert_eq!(
            peer.endpoint,
            Some(Endpoint {
                host: "vpn.example.com".into(),
                port: 51820
            })
        );
        assert_eq!(peer.allowed_ips.len(), 2);
        assert_eq!(peer.persistent_keepalive, Some(25));
    }

    #[test]
    fn keys_and_sections_are_case_insensitive() {
        let text = format!(
            "[interface]\nprivatekey={PRIV}\naddress=10.0.0.2\n[PEER]\npublickey={PUB}\nendpoint=1.2.3.4:51820\n"
        );
        let cfg: WgConfig = text.parse().unwrap();
        assert_eq!(cfg.interface.addresses[0].prefix, 32);
    }

    #[test]
    fn debug_never_shows_private_key() {
        let cfg: WgConfig = sample().parse().unwrap();
        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains(PRIV));
        assert!(!dbg.contains(PSK));
    }

    #[test]
    fn ipv6_endpoint() {
        let e: Endpoint = "[2001:db8::1]:51820".parse().unwrap();
        assert_eq!(e.host, "2001:db8::1");
        assert!("2001:db8::1:51820".parse::<Endpoint>().is_err());
        assert!("host:0".parse::<Endpoint>().is_err());
        assert!(":51820".parse::<Endpoint>().is_err());
    }

    #[test]
    fn rejects_bad_key() {
        let text = sample().replace(PUB, "nicht-base64");
        assert_eq!(
            text.parse::<WgConfig>(),
            Err(WgConfigError::InvalidValue {
                line: 9,
                key: "PublicKey".into()
            })
        );
        // Gültiges Base64, aber nur 16 Byte
        let text = sample().replace(PUB, "AAAAAAAAAAAAAAAAAAAAAA==");
        assert!(text.parse::<WgConfig>().is_err());
    }

    #[test]
    fn rejects_missing_parts() {
        assert_eq!("".parse::<WgConfig>(), Err(WgConfigError::MissingInterface));
        let no_peer = format!("[Interface]\nPrivateKey = {PRIV}\nAddress = 10.0.0.2/32\n");
        assert_eq!(no_peer.parse::<WgConfig>(), Err(WgConfigError::MissingPeer));
        let no_endpoint = format!("{no_peer}[Peer]\nPublicKey = {PUB}\n");
        assert_eq!(
            no_endpoint.parse::<WgConfig>(),
            Err(WgConfigError::MissingEndpoint { index: 1 })
        );
        let no_addr = sample().replace("Address = 10.2.0.2/32, fd00::2/128\n", "");
        assert_eq!(
            no_addr.parse::<WgConfig>(),
            Err(WgConfigError::MissingAddress)
        );
    }

    #[test]
    fn rejects_duplicates_and_garbage() {
        let dup = sample().replace(
            "PersistentKeepalive = 25\n",
            "PersistentKeepalive = 25\nPersistentKeepalive = 30\n",
        );
        assert!(matches!(
            dup.parse::<WgConfig>(),
            Err(WgConfigError::Duplicate { .. })
        ));
        assert!(matches!(
            "Address = 1.2.3.4".parse::<WgConfig>(),
            Err(WgConfigError::OutsideSection { line: 1 })
        ));
        assert!(matches!(
            "[Foo]".parse::<WgConfig>(),
            Err(WgConfigError::UnknownSection { .. })
        ));
        let bad_cidr = sample().replace("10.2.0.2/32", "10.2.0.2/33");
        assert!(bad_cidr.parse::<WgConfig>().is_err());
    }

    #[test]
    fn keepalive_off_means_none() {
        let text = sample().replace("PersistentKeepalive = 25", "PersistentKeepalive = off");
        let cfg: WgConfig = text.parse().unwrap();
        assert_eq!(cfg.peers[0].persistent_keepalive, None);
    }
}
