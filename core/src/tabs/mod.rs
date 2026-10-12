//! Grundlegendes Tab-Modell fuer Shinoho.

use serde::{Deserialize, Serialize};

/// Repraesentiert den Zustand eines einzelnen Tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_pinned: bool,
    pub is_incognito: bool,
    pub is_loading: bool,
}

impl TabInfo {
    pub fn new(id: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: "Neuer Tab".to_string(),
            url: url.into(),
            is_active: false,
            is_pinned: false,
            is_incognito: false,
            is_loading: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_info_creation() {
        let tab = TabInfo::new("tab-1", "https://example.com");
        assert_eq!(tab.id, "tab-1");
        assert_eq!(tab.url, "https://example.com");
        assert_eq!(tab.title, "Neuer Tab");
        assert!(!tab.is_active);
        assert!(!tab.is_pinned);
        assert!(!tab.is_incognito);
    }
}
