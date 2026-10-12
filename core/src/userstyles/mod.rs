//! Modul fuer Seiten-CSS (Userstyles) im Kern (T-303).

pub mod manager;
pub mod matcher;
pub mod models;

pub use manager::UserStylesManager;
pub use matcher::matches_pattern;
pub use models::{
    AddUserStyleRequest, GetStylesForUrlRequest, InjectStylesRequest, InjectionResult,
    RemoveUserStyleRequest, SiteStylesResponse, StyleRunAt, ToggleUserStyleRequest,
    UpdateUserStyleRequest, UserStyleRule,
};

use std::sync::{Arc, Mutex};

/// Thread-sicherer Tauri-State fuer den UserStylesManager.
#[derive(Debug, Clone, Default)]
pub struct UserStylesState(pub Arc<Mutex<UserStylesManager>>);

impl UserStylesState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(UserStylesManager::new())))
    }
}
