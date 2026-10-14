//! Shinoho VPN (T-401): WireGuard-Konfigurationen einlesen und die Liste der
//! empfohlenen Anbieter laden.
//!
//! Dazu der WireGuard-Tunnel mit lokalem SOCKS5-Proxy (T-402).

pub mod providers;
pub mod switchboard;
#[cfg(feature = "tauri")]
pub mod tauri_api;
pub mod tunnel;
pub mod wgconfig;

pub use providers::{load_bundled_providers, parse_providers, Provider, ProviderList};
pub use switchboard::{Route, Switchboard};
pub use tunnel::{Tunnel, TunnelError};
pub use wgconfig::{Endpoint, Key, WgConfig, WgInterface, WgPeer};
