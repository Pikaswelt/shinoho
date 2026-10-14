//! Shinoho Core Library
//!
//! Enthaelt Kernkomponenten fuer den Shinoho-Browser:
//! - Tab-Verwaltung und Lebenszyklus
//! - Tauri-Commands und Events (core/api v0)
//! - Fehlerbehandlung und Kern-Typen

pub mod api;
pub mod error;
pub mod tabs;
pub mod userstyles;

pub use error::{CoreError, CoreResult};
pub use tabs::{TabId, TabInfo, TabManager};
pub use userstyles::{UserStyleRule, UserStylesManager, UserStylesState};
