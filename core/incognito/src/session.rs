use std::collections::HashSet;
use std::path::{Path, PathBuf};

use shinoho_storage::Storage;
use tempfile::TempDir;

use crate::config::IncognitoConfig;
use crate::error::IncognitoError;

pub struct IncognitoSession {
    id: String,
    config: IncognitoConfig,
    storage: Storage,
    temp_dir: TempDir,
    active_tabs: HashSet<String>,
    active_windows: HashSet<String>,
    created_at: i64,
}

impl IncognitoSession {
    pub fn new(id: String, config: IncognitoConfig) -> Result<Self, IncognitoError> {
        let storage = Storage::open_in_memory()?;
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("shinoho_incognito_{}_", id))
            .tempdir()?;

        Ok(Self {
            id,
            config,
            storage,
            temp_dir,
            active_tabs: HashSet::new(),
            active_windows: HashSet::new(),
            created_at: chrono::Utc::now().timestamp_millis(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn config(&self) -> &IncognitoConfig {
        &self.config
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn data_directory(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn data_directory_path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    pub fn add_tab(&mut self, tab_id: &str) {
        self.active_tabs.insert(tab_id.to_string());
    }

    pub fn remove_tab(&mut self, tab_id: &str) -> bool {
        self.active_tabs.remove(tab_id);
        self.is_empty()
    }

    pub fn add_window(&mut self, window_id: &str) {
        self.active_windows.insert(window_id.to_string());
    }

    pub fn remove_window(&mut self, window_id: &str) -> bool {
        self.active_windows.remove(window_id);
        self.is_empty()
    }

    pub fn active_tab_count(&self) -> usize {
        self.active_tabs.len()
    }

    pub fn active_window_count(&self) -> usize {
        self.active_windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.active_tabs.is_empty() && self.active_windows.is_empty()
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Explicitly destroy the session, wiping in-memory storage and deleting the temporary directory.
    pub fn wipe(self) -> Result<(), IncognitoError> {
        // Clearing in-memory storage
        let _ = self.storage.clear_history();
        let _ = self.storage.clear_downloads();

        // TempDir automatically removes itself on drop, but close() returns any io error
        self.temp_dir.close()?;
        Ok(())
    }
}
