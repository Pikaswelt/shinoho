//! Shinoho VPN (T-401): WireGuard-Konfigurationen einlesen und die Liste der
//! empfohlenen Anbieter laden.
//!
//! Dazu der WireGuard-Tunnel mit lokalem SOCKS5-Proxy (T-402).

pub mod providers;
pub mod tunnel;
pub mod wgconfig;

pub use providers::{load_bundled_providers, parse_providers, Provider, ProviderList};
pub use tunnel::{Tunnel, TunnelError};
pub use wgconfig::{Endpoint, Key, WgConfig, WgInterface, WgPeer};
