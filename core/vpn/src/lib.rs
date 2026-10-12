//! Shinoho VPN (T-401): WireGuard-Konfigurationen einlesen und die Liste der
//! empfohlenen Anbieter laden.
//!
//! Der Tunnel selbst (boringtun + lokaler Proxy) folgt in T-402.

pub mod providers;
pub mod wgconfig;

pub use providers::{load_bundled_providers, parse_providers, Provider, ProviderList};
pub use wgconfig::{Endpoint, Key, WgConfig, WgInterface, WgPeer};
