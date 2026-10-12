//! Shinoho Core Library
//!
//! Enthaelt Kernkomponenten fuer den Shinoho-Browser:
//! - Tab-Verwaltung und Lebenszyklus
//! - Fehlerbehandlung und Kern-Typen

pub mod error;
pub mod tabs;

pub use error::{CoreError, CoreResult};
pub use tabs::TabInfo;
