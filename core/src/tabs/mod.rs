//! Tab-Modell, Tab-Manager und WebView-Koordinator fuer Shinoho (T-101).

pub mod manager;
pub mod models;
pub mod webview;

pub use manager::{TabHistory, TabManager};
pub use models::*;
pub use webview::{WebViewCoordinator, WebViewTabState};
