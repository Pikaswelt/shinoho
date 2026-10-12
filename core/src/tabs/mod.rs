//! Tab-Modell und Tab-Manager fuer Shinoho.

use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Eindeutige Kennung fuer einen Tab.
pub type TabId = String;

/// Detaillierte Information ueber einen Tab (kompatibel mit core/api types.ts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabInfo {
    pub id: TabId,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_pinned: bool,
    pub is_muted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    pub created_at: i64,
}

impl TabInfo {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let url_str = url.into();
        let title = if url_str == "shinoho://hub" || url_str.is_empty() {
            "Neuer Tab".to_string()
        } else {
            url_str.clone()
        };

        Self {
            id: id.into(),
            title,
            url: url_str,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            is_pinned: false,
            is_muted: false,
            favicon: None,
            created_at: now,
        }
    }
}

/// Request zum Erstellen eines neuen Tabs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTabRequest {
    pub url: Option<String>,
    pub activate: Option<bool>,
    pub is_pinned: Option<bool>,
}

/// Request zum Navigieren eines Tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigateTabRequest {
    pub tab_id: TabId,
    pub url: String,
}

/// Request fuer Standard-Aktionen auf einem Tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabActionRequest {
    pub tab_id: TabId,
}

/// Request zum Neuladen eines Tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadTabRequest {
    pub tab_id: TabId,
    pub ignore_cache: Option<bool>,
}

/// Tab-Events gemaess core/api types.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TabEvent {
    #[serde(rename_all = "camelCase")]
    TabCreated { tab: TabInfo },
    #[serde(rename_all = "camelCase")]
    TabUpdated { tab: TabInfo },
    #[serde(rename_all = "camelCase")]
    TabClosed { tab_id: TabId },
    #[serde(rename_all = "camelCase")]
    TabActivated { tab_id: TabId },
}

#[derive(Debug, Clone)]
struct TabHistory {
    stack: Vec<String>,
    index: usize,
}

/// In-Memory Tab-Manager fuer Lebenszyklus und Navigation.
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<TabInfo>,
    active_tab_id: Option<TabId>,
    history_map: HashMap<TabId, TabHistory>,
    id_counter: u64,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab_id: None,
            history_map: HashMap::new(),
            id_counter: 0,
        };
        // Standard-Tab anlegen
        manager.create_tab(Some(CreateTabRequest {
            url: Some("shinoho://hub".to_string()),
            activate: Some(true),
            is_pinned: Some(false),
        }));
        manager
    }

    pub fn get_tabs(&self) -> Vec<TabInfo> {
        self.tabs.clone()
    }

    pub fn get_active_tab(&self) -> Option<TabInfo> {
        if let Some(ref active_id) = self.active_tab_id {
            self.tabs.iter().find(|t| &t.id == active_id).cloned()
        } else {
            self.tabs.first().cloned()
        }
    }

    pub fn create_tab(&mut self, request: Option<CreateTabRequest>) -> TabInfo {
        self.id_counter += 1;
        let tab_id = format!("tab-{}", self.id_counter);
        let req = request.unwrap_or_default();
        let url = req.url.unwrap_or_else(|| "shinoho://hub".to_string());
        let mut tab = TabInfo::new(tab_id.clone(), url.clone());
        tab.is_pinned = req.is_pinned.unwrap_or(false);

        self.history_map.insert(
            tab_id.clone(),
            TabHistory {
                stack: vec![url],
                index: 0,
            },
        );

        let should_activate = req.activate.unwrap_or(true);
        if should_activate {
            self.active_tab_id = Some(tab_id);
        }

        self.tabs.push(tab.clone());
        tab
    }

    pub fn close_tab(&mut self, tab_id: &str) -> CoreResult<()> {
        let index = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        self.tabs.remove(index);
        self.history_map.remove(tab_id);

        if self.active_tab_id.as_deref() == Some(tab_id) {
            if !self.tabs.is_empty() {
                let next_idx = index.min(self.tabs.len() - 1);
                self.active_tab_id = Some(self.tabs[next_idx].id.clone());
            } else {
                self.active_tab_id = None;
            }
        }

        Ok(())
    }

    pub fn activate_tab(&mut self, tab_id: &str) -> CoreResult<()> {
        if !self.tabs.iter().any(|t| t.id == tab_id) {
            return Err(CoreError::TabNotFound(tab_id.to_string()));
        }
        self.active_tab_id = Some(tab_id.to_string());
        Ok(())
    }

    pub fn navigate_tab(&mut self, tab_id: &str, url: &str) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        let history = self
            .history_map
            .entry(tab_id.to_string())
            .or_insert_with(|| TabHistory {
                stack: vec![tab.url.clone()],
                index: 0,
            });

        history.stack.truncate(history.index + 1);
        history.stack.push(url.to_string());
        history.index += 1;

        tab.url = url.to_string();
        tab.title = if url.starts_with("http://") || url.starts_with("https://") {
            url.split("://")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or(url)
                .to_string()
        } else {
            url.to_string()
        };
        tab.can_go_back = history.index > 0;
        tab.can_go_forward = false;

        Ok(tab.clone())
    }

    pub fn go_back(&mut self, tab_id: &str) -> CoreResult<TabInfo> {
        let history = self
            .history_map
            .get_mut(tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        if history.index == 0 {
            return Err(CoreError::InvalidOperation(
                "Kein Zurueck-Verlauf vorhanden".to_string(),
            ));
        }

        history.index -= 1;
        let target_url = history.stack[history.index].clone();
        let can_back = history.index > 0;
        let can_forward = history.index < history.stack.len() - 1;

        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.url = target_url.clone();
        tab.title = target_url;
        tab.can_go_back = can_back;
        tab.can_go_forward = can_forward;

        Ok(tab.clone())
    }

    pub fn go_forward(&mut self, tab_id: &str) -> CoreResult<TabInfo> {
        let history = self
            .history_map
            .get_mut(tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        if history.index + 1 >= history.stack.len() {
            return Err(CoreError::InvalidOperation(
                "Kein Vorwaerts-Verlauf vorhanden".to_string(),
            ));
        }

        history.index += 1;
        let target_url = history.stack[history.index].clone();
        let can_back = history.index > 0;
        let can_forward = history.index < history.stack.len() - 1;

        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.url = target_url.clone();
        tab.title = target_url;
        tab.can_go_back = can_back;
        tab.can_go_forward = can_forward;

        Ok(tab.clone())
    }

    pub fn reload_tab(&mut self, tab_id: &str, _ignore_cache: bool) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.is_loading = false;
        Ok(tab.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_manager_lifecycle() {
        let mut mgr = TabManager::new();
        assert_eq!(mgr.get_tabs().len(), 1);

        let created = mgr.create_tab(Some(CreateTabRequest {
            url: Some("https://shinoho.org".to_string()),
            activate: Some(true),
            is_pinned: Some(false),
        }));
        assert_eq!(mgr.get_tabs().len(), 2);
        assert_eq!(mgr.get_active_tab().unwrap().id, created.id);

        let nav = mgr.navigate_tab(&created.id, "https://github.com").unwrap();
        assert_eq!(nav.url, "https://github.com");
        assert!(nav.can_go_back);
        assert!(!nav.can_go_forward);

        let back = mgr.go_back(&created.id).unwrap();
        assert_eq!(back.url, "https://shinoho.org");
        assert!(!back.can_go_back);
        assert!(back.can_go_forward);

        let fwd = mgr.go_forward(&created.id).unwrap();
        assert_eq!(fwd.url, "https://github.com");

        mgr.close_tab(&created.id).unwrap();
        assert_eq!(mgr.get_tabs().len(), 1);
    }
}
