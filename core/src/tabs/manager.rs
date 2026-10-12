//! Kern-Tab-Manager mit vollständigem Lebenszyklus und Tab-Gruppierung.

use crate::error::{CoreError, CoreResult};
use crate::tabs::models::{
    AssignTabGroupRequest, CreateTabGroupRequest, CreateTabRequest, TabGroup, TabHistoryEntry,
    TabId, TabInfo, UpdateTabGroupRequest, current_time_millis,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TabHistory {
    pub entries: Vec<TabHistoryEntry>,
    pub index: usize,
}

/// Vollständiger TabManager fuer den Shinoho-Browser (T-101).
#[derive(Debug)]
pub struct TabManager {
    tabs: Vec<TabInfo>,
    active_tab_id: Option<TabId>,
    history_map: HashMap<TabId, TabHistory>,
    groups: HashMap<String, TabGroup>,
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
            groups: HashMap::new(),
            id_counter: 0,
        };
        // Standard-Tab erzeugen
        manager.create_tab(Some(CreateTabRequest {
            url: Some("shinoho://hub".to_string()),
            activate: Some(true),
            is_pinned: Some(false),
            is_incognito: Some(false),
            incognito_session_id: None,
            group_id: None,
        }));
        manager
    }

    /// Gibt alle aktuell verwalteten Tabs zurück.
    pub fn get_tabs(&self) -> Vec<TabInfo> {
        self.tabs.clone()
    }

    /// Gibt den aktuell aktiven Tab zurück.
    pub fn get_active_tab(&self) -> Option<TabInfo> {
        if let Some(ref active_id) = self.active_tab_id {
            self.tabs.iter().find(|t| &t.id == active_id).cloned()
        } else {
            self.tabs.first().cloned()
        }
    }

    /// Gibt einen bestimmten Tab anhand seiner ID zurück.
    pub fn get_tab(&self, tab_id: &str) -> Option<TabInfo> {
        self.tabs.iter().find(|t| t.id == tab_id).cloned()
    }

    /// Erstellt einen neuen Tab (regulär oder inkognito).
    pub fn create_tab(&mut self, request: Option<CreateTabRequest>) -> TabInfo {
        self.id_counter += 1;
        let tab_id = format!("tab-{}", self.id_counter);
        let req = request.unwrap_or_default();
        let url = req.url.unwrap_or_else(|| "shinoho://hub".to_string());
        let mut tab = TabInfo::new(tab_id.clone(), url.clone());

        tab.is_pinned = req.is_pinned.unwrap_or(false);
        tab.is_incognito = req.is_incognito.unwrap_or(false);
        tab.incognito_session_id = req.incognito_session_id;
        tab.group_id = req.group_id;

        let now = current_time_millis();
        self.history_map.insert(
            tab_id.clone(),
            TabHistory {
                entries: vec![TabHistoryEntry {
                    url,
                    title: tab.title.clone(),
                    timestamp: now,
                }],
                index: 0,
            },
        );

        let should_activate = req.activate.unwrap_or(true);
        if should_activate {
            self.active_tab_id = Some(tab_id.clone());
        }

        // Pinned Tabs an den Anfang, normale Tabs ans Ende
        if tab.is_pinned {
            let last_pinned_idx = self
                .tabs
                .iter()
                .rposition(|t| t.is_pinned)
                .map(|i| i + 1)
                .unwrap_or(0);
            self.tabs.insert(last_pinned_idx, tab.clone());
        } else {
            self.tabs.push(tab.clone());
        }

        tab
    }

    /// Schliesst einen Tab anhand seiner ID.
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
                self.tabs[next_idx].last_accessed_at = current_time_millis();
            } else {
                self.active_tab_id = None;
            }
        }

        Ok(())
    }

    /// Schliesst alle Tabs ausser dem angegebenen.
    pub fn close_other_tabs(&mut self, keep_tab_id: &str) -> CoreResult<Vec<TabId>> {
        if !self.tabs.iter().any(|t| t.id == keep_tab_id) {
            return Err(CoreError::TabNotFound(keep_tab_id.to_string()));
        }

        let mut closed_ids = Vec::new();
        let mut i = 0;
        while i < self.tabs.len() {
            if self.tabs[i].id != keep_tab_id && !self.tabs[i].is_pinned {
                let id = self.tabs.remove(i).id;
                self.history_map.remove(&id);
                closed_ids.push(id);
            } else {
                i += 1;
            }
        }

        self.active_tab_id = Some(keep_tab_id.to_string());
        Ok(closed_ids)
    }

    /// Schliesst alle nicht-gepinnten Tabs rechts vom angegebenen Tab.
    pub fn close_tabs_to_the_right(&mut self, tab_id: &str) -> CoreResult<Vec<TabId>> {
        let pos = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        let mut closed_ids = Vec::new();
        let mut i = pos + 1;
        while i < self.tabs.len() {
            if !self.tabs[i].is_pinned {
                let id = self.tabs.remove(i).id;
                self.history_map.remove(&id);
                closed_ids.push(id);
            } else {
                i += 1;
            }
        }

        if self
            .active_tab_id
            .as_ref()
            .is_some_and(|active| closed_ids.contains(active))
        {
            self.active_tab_id = Some(tab_id.to_string());
        }

        Ok(closed_ids)
    }

    /// Aktiviert einen bestimmten Tab.
    pub fn activate_tab(&mut self, tab_id: &str) -> CoreResult<()> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.last_accessed_at = current_time_millis();
        self.active_tab_id = Some(tab_id.to_string());
        Ok(())
    }

    /// Dupliziert einen bestehenden Tab.
    pub fn duplicate_tab(&mut self, tab_id: &str) -> CoreResult<TabInfo> {
        let source = self
            .tabs
            .iter()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?
            .clone();

        let new_tab = self.create_tab(Some(CreateTabRequest {
            url: Some(source.url),
            activate: Some(true),
            is_pinned: Some(source.is_pinned),
            is_incognito: Some(source.is_incognito),
            incognito_session_id: source.incognito_session_id,
            group_id: source.group_id,
        }));

        Ok(new_tab)
    }

    /// Verschiebt die Position eines Tabs.
    pub fn move_tab(&mut self, tab_id: &str, new_index: usize) -> CoreResult<(usize, usize)> {
        let current_index = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        let clamped_index = new_index.min(self.tabs.len().saturating_sub(1));
        if current_index == clamped_index {
            return Ok((current_index, clamped_index));
        }

        let tab = self.tabs.remove(current_index);
        self.tabs.insert(clamped_index, tab);

        Ok((current_index, clamped_index))
    }

    /// Pinnt oder entpinnt einen Tab.
    pub fn set_pinned(&mut self, tab_id: &str, is_pinned: bool) -> CoreResult<TabInfo> {
        let index = self
            .tabs
            .iter()
            .position(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        let mut tab = self.tabs.remove(index);
        tab.is_pinned = is_pinned;

        if is_pinned {
            let insert_pos = self
                .tabs
                .iter()
                .rposition(|t| t.is_pinned)
                .map(|i| i + 1)
                .unwrap_or(0);
            self.tabs.insert(insert_pos, tab.clone());
        } else {
            self.tabs.push(tab.clone());
        }

        Ok(tab)
    }

    /// Schaltet einen Tab stumm oder laut.
    pub fn set_muted(&mut self, tab_id: &str, is_muted: bool) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.is_muted = is_muted;
        Ok(tab.clone())
    }

    /// Navigiert einen Tab zu einer neuen URL.
    pub fn navigate_tab(&mut self, tab_id: &str, url: &str) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        let now = current_time_millis();
        let history = self
            .history_map
            .entry(tab_id.to_string())
            .or_insert_with(|| TabHistory {
                entries: vec![TabHistoryEntry {
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    timestamp: now,
                }],
                index: 0,
            });

        history.entries.truncate(history.index + 1);
        history.entries.push(TabHistoryEntry {
            url: url.to_string(),
            title: url.to_string(),
            timestamp: now,
        });
        history.index += 1;

        tab.url = url.to_string();
        tab.title = parse_title_from_url(url);
        tab.can_go_back = history.index > 0;
        tab.can_go_forward = false;
        tab.last_accessed_at = now;

        Ok(tab.clone())
    }

    /// Geht im Verlauf zurueck.
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
        let entry = history.entries[history.index].clone();
        let can_back = history.index > 0;
        let can_forward = history.index < history.entries.len() - 1;

        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.url = entry.url;
        tab.title = entry.title;
        tab.can_go_back = can_back;
        tab.can_go_forward = can_forward;
        tab.last_accessed_at = current_time_millis();

        Ok(tab.clone())
    }

    /// Geht im Verlauf vorwaerts.
    pub fn go_forward(&mut self, tab_id: &str) -> CoreResult<TabInfo> {
        let history = self
            .history_map
            .get_mut(tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        if history.index + 1 >= history.entries.len() {
            return Err(CoreError::InvalidOperation(
                "Kein Vorwaerts-Verlauf vorhanden".to_string(),
            ));
        }

        history.index += 1;
        let entry = history.entries[history.index].clone();
        let can_back = history.index > 0;
        let can_forward = history.index < history.entries.len() - 1;

        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.url = entry.url;
        tab.title = entry.title;
        tab.can_go_back = can_back;
        tab.can_go_forward = can_forward;
        tab.last_accessed_at = current_time_millis();

        Ok(tab.clone())
    }

    /// Laedt den Tab neu.
    pub fn reload_tab(&mut self, tab_id: &str, _ignore_cache: bool) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        tab.is_loading = false;
        tab.last_accessed_at = current_time_millis();
        Ok(tab.clone())
    }

    /// Aktualisiert Titel und Favicon eines Tabs.
    pub fn update_metadata(
        &mut self,
        tab_id: &str,
        title: Option<String>,
        favicon: Option<String>,
    ) -> CoreResult<TabInfo> {
        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| CoreError::TabNotFound(tab_id.to_string()))?;

        if let Some(t) = title {
            tab.title = t;
        }
        if let Some(f) = favicon {
            tab.favicon = Some(f);
        }

        Ok(tab.clone())
    }

    // --- Tab-Gruppen Management ---

    /// Erstellt eine neue Tab-Gruppe.
    pub fn create_tab_group(&mut self, req: CreateTabGroupRequest) -> TabGroup {
        let group_id = format!("group-{}", self.groups.len() + 1);
        let group = TabGroup::new(group_id.clone(), req.title, req.color);

        for tab_id in &req.tab_ids {
            if let Some(tab) = self.tabs.iter_mut().find(|t| &t.id == tab_id) {
                tab.group_id = Some(group_id.clone());
            }
        }

        self.groups.insert(group_id, group.clone());
        group
    }

    /// Liefert alle definierten Tab-Gruppen.
    pub fn get_tab_groups(&self) -> Vec<TabGroup> {
        self.groups.values().cloned().collect()
    }

    /// Weist einen Tab einer Gruppe zu oder entfernt ihn aus einer Gruppe.
    pub fn assign_tab_to_group(&mut self, req: AssignTabGroupRequest) -> CoreResult<TabInfo> {
        if let Some(gid) = req
            .group_id
            .as_deref()
            .filter(|gid| !self.groups.contains_key(*gid))
        {
            return Err(CoreError::InvalidOperation(format!(
                "Gruppe {gid} existiert nicht"
            )));
        }

        let tab = self
            .tabs
            .iter_mut()
            .find(|t| t.id == req.tab_id)
            .ok_or_else(|| CoreError::TabNotFound(req.tab_id.clone()))?;

        tab.group_id = req.group_id;
        Ok(tab.clone())
    }

    /// Aktualisiert Metadaten einer Tab-Gruppe.
    pub fn update_tab_group(&mut self, req: UpdateTabGroupRequest) -> CoreResult<TabGroup> {
        let group = self.groups.get_mut(&req.group_id).ok_or_else(|| {
            CoreError::InvalidOperation(format!("Gruppe {} existiert nicht", req.group_id))
        })?;

        if let Some(title) = req.title {
            group.title = title;
        }
        if let Some(color) = req.color {
            group.color = color;
        }
        if let Some(collapsed) = req.collapsed {
            group.collapsed = collapsed;
        }

        Ok(group.clone())
    }
}

fn parse_title_from_url(url: &str) -> String {
    if url == "shinoho://hub" || url.is_empty() {
        "Neuer Tab".to_string()
    } else if url.starts_with("http://") || url.starts_with("https://") {
        url.split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(url)
            .to_string()
    } else {
        url.to_string()
    }
}
