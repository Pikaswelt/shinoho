use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use shinoho_storage::Storage;
use uuid::Uuid;

use crate::config::IncognitoConfig;
use crate::error::IncognitoError;
use crate::session::IncognitoSession;

#[derive(Clone, Default)]
pub struct IncognitoManager {
    sessions: Arc<Mutex<HashMap<String, IncognitoSession>>>,
    default_config: IncognitoConfig,
}

impl IncognitoManager {
    pub fn new(default_config: IncognitoConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            default_config,
        }
    }

    pub fn create_session(
        &self,
        custom_id: Option<&str>,
        config: Option<IncognitoConfig>,
    ) -> Result<String, IncognitoError> {
        let id = custom_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let cfg = config.unwrap_or_else(|| self.default_config.clone());
        let session = IncognitoSession::new(id.clone(), cfg)?;

        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        if lock.contains_key(&id) {
            return Err(IncognitoError::SessionAlreadyExists(id));
        }

        lock.insert(id.clone(), session);
        Ok(id)
    }

    pub fn has_session(&self, session_id: &str) -> Result<bool, IncognitoError> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        Ok(lock.contains_key(session_id))
    }

    pub fn close_session(&self, session_id: &str) -> Result<(), IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        if let Some(session) = lock.remove(session_id) {
            session.wipe()?;
            Ok(())
        } else {
            Err(IncognitoError::SessionNotFound(session_id.to_string()))
        }
    }

    pub fn close_all_sessions(&self) -> Result<(), IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        for (_, session) in lock.drain() {
            session.wipe()?;
        }
        Ok(())
    }

    pub fn register_tab(&self, session_id: &str, tab_id: &str) -> Result<(), IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        let session = lock
            .get_mut(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        session.add_tab(tab_id);
        Ok(())
    }

    pub fn unregister_tab(
        &self,
        session_id: &str,
        tab_id: &str,
        auto_close_if_empty: bool,
    ) -> Result<bool, IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        let is_empty = {
            let session = lock
                .get_mut(session_id)
                .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
            session.remove_tab(tab_id)
        };

        if is_empty && auto_close_if_empty {
            if let Some(session) = lock.remove(session_id) {
                session.wipe()?;
            }
            Ok(true)
        } else {
            Ok(is_empty)
        }
    }

    pub fn register_window(&self, session_id: &str, window_id: &str) -> Result<(), IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        let session = lock
            .get_mut(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        session.add_window(window_id);
        Ok(())
    }

    pub fn unregister_window(
        &self,
        session_id: &str,
        window_id: &str,
        auto_close_if_empty: bool,
    ) -> Result<bool, IncognitoError> {
        let mut lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        let is_empty = {
            let session = lock
                .get_mut(session_id)
                .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
            session.remove_window(window_id)
        };

        if is_empty && auto_close_if_empty {
            if let Some(session) = lock.remove(session_id) {
                session.wipe()?;
            }
            Ok(true)
        } else {
            Ok(is_empty)
        }
    }

    pub fn with_session_storage<F, R>(&self, session_id: &str, f: F) -> Result<R, IncognitoError>
    where
        F: FnOnce(&Storage) -> R,
    {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        let session = lock
            .get(session_id)
            .ok_or_else(|| IncognitoError::SessionNotFound(session_id.to_string()))?;
        Ok(f(session.storage()))
    }

    pub fn active_session_ids(&self) -> Result<Vec<String>, IncognitoError> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        Ok(lock.keys().cloned().collect())
    }

    pub fn active_session_count(&self) -> Result<usize, IncognitoError> {
        let lock = self
            .sessions
            .lock()
            .map_err(|_| IncognitoError::LockPoisoned)?;
        Ok(lock.len())
    }
}
