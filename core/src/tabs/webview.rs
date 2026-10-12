//! WebView-Koordinator fuer Multi-WebView Tab-Management in Tauri 2.

use crate::tabs::models::TabId;
use std::collections::HashMap;

/// Repraesentiert den Status eines nativen WebViews fuer einen Tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebViewTabState {
    pub tab_id: TabId,
    pub webview_label: String,
    pub is_created: bool,
    pub is_visible: bool,
    pub url: String,
}

impl WebViewTabState {
    pub fn new(tab_id: impl Into<String>, url: impl Into<String>) -> Self {
        let id = tab_id.into();
        let label = format!("webview-{}", id.replace(':', "-"));
        Self {
            tab_id: id,
            webview_label: label,
            is_created: false,
            is_visible: false,
            url: url.into(),
        }
    }
}

/// Koordinator fuer mehrere WebViews im Hauptfenster.
#[derive(Debug, Default)]
pub struct WebViewCoordinator {
    views: HashMap<TabId, WebViewTabState>,
    active_tab_id: Option<TabId>,
}

impl WebViewCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert einen neuen Tab beim WebView-Koordinator.
    pub fn register_tab(&mut self, tab_id: &str, url: &str) -> WebViewTabState {
        let state = WebViewTabState::new(tab_id, url);
        self.views.insert(tab_id.to_string(), state.clone());
        state
    }

    /// Entfernt einen Tab und markiert den zugehoerigen WebView zum Loeschen.
    pub fn unregister_tab(&mut self, tab_id: &str) -> Option<WebViewTabState> {
        let removed = self.views.remove(tab_id);
        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = None;
        }
        removed
    }

    /// Wechselt den aktiven Tab und berechnet, welche WebViews sichtbar sein sollen.
    pub fn switch_active_tab(&mut self, tab_id: &str) -> (Option<String>, Option<String>) {
        let previous_label = self
            .active_tab_id
            .as_ref()
            .and_then(|prev_id| self.views.get_mut(prev_id))
            .map(|view| {
                view.is_visible = false;
                view.webview_label.clone()
            });

        let next_label = if let Some(view) = self.views.get_mut(tab_id) {
            view.is_visible = true;
            self.active_tab_id = Some(tab_id.to_string());
            Some(view.webview_label.clone())
        } else {
            None
        };

        (previous_label, next_label)
    }

    /// Gibt das Label des WebViews fuer einen Tab zurueck.
    pub fn get_label(&self, tab_id: &str) -> Option<&str> {
        self.views.get(tab_id).map(|v| v.webview_label.as_str())
    }

    /// Prueft, ob ein Tab bereits registriert ist.
    pub fn has_tab(&self, tab_id: &str) -> bool {
        self.views.contains_key(tab_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_coordinator_switching() {
        let mut coord = WebViewCoordinator::new();
        coord.register_tab("tab-1", "https://google.com");
        coord.register_tab("tab-2", "https://shinoho.org");

        let (prev, next) = coord.switch_active_tab("tab-1");
        assert_eq!(prev, None);
        assert_eq!(next, Some("webview-tab-1".to_string()));

        let (prev2, next2) = coord.switch_active_tab("tab-2");
        assert_eq!(prev2, Some("webview-tab-1".to_string()));
        assert_eq!(next2, Some("webview-tab-2".to_string()));

        assert!(coord.has_tab("tab-1"));
        let removed = coord.unregister_tab("tab-1");
        assert!(removed.is_some());
        assert!(!coord.has_tab("tab-1"));
    }
}
