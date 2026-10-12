//! Datenmodelle fuer Tabs, Gruppen und History.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Eindeutige Kennung fuer einen Tab.
pub type TabId = String;

/// Detaillierte Information ueber einen Tab.
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
    pub is_incognito: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incognito_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    pub created_at: i64,
    pub last_accessed_at: i64,
}

impl TabInfo {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        let now = current_time_millis();
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
            is_incognito: false,
            incognito_session_id: None,
            group_id: None,
            favicon: None,
            created_at: now,
            last_accessed_at: now,
        }
    }
}

/// Gruppe zur Organisation mehrerer Tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabGroup {
    pub id: String,
    pub title: String,
    pub color: String,
    pub collapsed: bool,
}

impl TabGroup {
    pub fn new(id: impl Into<String>, title: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            color: color.into(),
            collapsed: false,
        }
    }
}

/// Eintrag in der Historie eines Tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabHistoryEntry {
    pub url: String,
    pub title: String,
    pub timestamp: i64,
}

/// Request zum Erstellen eines neuen Tabs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTabRequest {
    pub url: Option<String>,
    pub activate: Option<bool>,
    pub is_pinned: Option<bool>,
    pub is_incognito: Option<bool>,
    pub incognito_session_id: Option<String>,
    pub group_id: Option<String>,
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

/// Request zum Verschieben der Position eines Tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveTabRequest {
    pub tab_id: TabId,
    pub new_index: usize,
}

/// Request zum Pinnen/Entpinnen eines Tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPinnedRequest {
    pub tab_id: TabId,
    pub is_pinned: bool,
}

/// Request zum Stummschalten eines Tabs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMutedRequest {
    pub tab_id: TabId,
    pub is_muted: bool,
}

/// Request zum Erstellen einer Tab-Gruppe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTabGroupRequest {
    pub title: String,
    pub color: String,
    pub tab_ids: Vec<TabId>,
}

/// Request zum Aktualisieren einer Tab-Gruppe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTabGroupRequest {
    pub group_id: String,
    pub title: Option<String>,
    pub color: Option<String>,
    pub collapsed: Option<bool>,
}

/// Request zum Zuweisen eines Tabs zu einer Gruppe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignTabGroupRequest {
    pub tab_id: TabId,
    pub group_id: Option<String>,
}

/// Tab-Events gemaess core/api types.ts und Shinoho-Erweiterungen.
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
    #[serde(rename_all = "camelCase")]
    TabMoved {
        tab_id: TabId,
        from_index: usize,
        to_index: usize,
    },
    #[serde(rename_all = "camelCase")]
    TabGroupCreated { group: TabGroup },
    #[serde(rename_all = "camelCase")]
    TabGroupUpdated { group: TabGroup },
}

pub fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
