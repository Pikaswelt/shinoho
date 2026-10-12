//! Schnittstelle Kern <-> UI (core/api v0) und Tab-Manager Erweiterungen (T-101).

use crate::tabs::{
    AssignTabGroupRequest, CreateTabGroupRequest, CreateTabRequest, MoveTabRequest,
    NavigateTabRequest, ReloadTabRequest, SetMutedRequest, SetPinnedRequest, TabActionRequest,
    TabEvent, TabGroup, TabInfo, TabManager, UpdateTabGroupRequest,
};
use crate::userstyles::{
    AddUserStyleRequest, GetStylesForUrlRequest, InjectStylesRequest, InjectionResult,
    RemoveUserStyleRequest, SiteStylesResponse, ToggleUserStyleRequest, UpdateUserStyleRequest,
    UserStyleRule, UserStylesState,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub const EVENT_TAB_CREATED: &str = "shinoho://tab-created";
pub const EVENT_TAB_UPDATED: &str = "shinoho://tab-updated";
pub const EVENT_TAB_CLOSED: &str = "shinoho://tab-closed";
pub const EVENT_TAB_ACTIVATED: &str = "shinoho://tab-activated";
pub const EVENT_TAB_MOVED: &str = "shinoho://tab-moved";
pub const EVENT_TAB_GROUP_CREATED: &str = "shinoho://tab-group-created";
pub const EVENT_TAB_GROUP_UPDATED: &str = "shinoho://tab-group-updated";

pub const EVENT_STYLES_INJECTED: &str = "shinoho://styles-injected";
pub const EVENT_USERSTYLE_ADDED: &str = "shinoho://userstyle-added";
pub const EVENT_USERSTYLE_UPDATED: &str = "shinoho://userstyle-updated";
pub const EVENT_USERSTYLE_REMOVED: &str = "shinoho://userstyle-removed";

/// Events fuer Userstyles und CSS-Injektion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum StyleEvent {
    #[serde(rename_all = "camelCase")]
    StylesInjected { injection: InjectionResult },
    #[serde(rename_all = "camelCase")]
    UserStyleAdded { rule: UserStyleRule },
    #[serde(rename_all = "camelCase")]
    UserStyleUpdated { rule: UserStyleRule },
    #[serde(rename_all = "camelCase")]
    UserStyleRemoved { id: String },
}

/// Verwalteter Zustand des TabManagers fuer Tauri.
#[derive(Debug, Default)]
pub struct TabManagerState(pub Mutex<TabManager>);

impl TabManagerState {
    pub fn new() -> Self {
        Self(Mutex::new(TabManager::new()))
    }
}

/// Liefert alle geoeffneten Tabs zurueck.
#[tauri::command]
pub fn get_tabs(state: State<'_, TabManagerState>) -> Result<Vec<TabInfo>, String> {
    let mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    Ok(mgr.get_tabs())
}

/// Liefert den aktuell aktiven Tab zurueck.
#[tauri::command]
pub fn get_active_tab(state: State<'_, TabManagerState>) -> Result<Option<TabInfo>, String> {
    let mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    Ok(mgr.get_active_tab())
}

/// Erstellt einen neuen Tab und emittiert shinoho://tab-created (sowie shinoho://tab-activated falls aktiviert).
#[tauri::command]
pub fn create_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: Option<CreateTabRequest>,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let should_activate = request.as_ref().and_then(|r| r.activate).unwrap_or(true);
    let tab = mgr.create_tab(request);

    let _ = app.emit(EVENT_TAB_CREATED, TabEvent::TabCreated { tab: tab.clone() });

    if should_activate {
        let _ = app.emit(
            EVENT_TAB_ACTIVATED,
            TabEvent::TabActivated {
                tab_id: tab.id.clone(),
            },
        );
    }

    Ok(tab)
}

/// Schliesst einen Tab anhand seiner ID und emittiert shinoho://tab-closed.
#[tauri::command]
pub fn close_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab_id = request.tab_id;
    mgr.close_tab(&tab_id).map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_TAB_CLOSED,
        TabEvent::TabClosed {
            tab_id: tab_id.clone(),
        },
    );

    if let Some(active) = mgr.get_active_tab() {
        let _ = app.emit(
            EVENT_TAB_ACTIVATED,
            TabEvent::TabActivated { tab_id: active.id },
        );
    }

    Ok(())
}

/// Aktiviert einen Tab und emittiert shinoho://tab-activated.
#[tauri::command]
pub fn activate_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<(), String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab_id = request.tab_id;
    mgr.activate_tab(&tab_id).map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_ACTIVATED, TabEvent::TabActivated { tab_id });

    Ok(())
}

/// Navigiert einen Tab zu einer neuen URL und emittiert shinoho://tab-updated.
#[tauri::command]
pub fn navigate_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: NavigateTabRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab = mgr
        .navigate_tab(&request.tab_id, &request.url)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });

    Ok(tab)
}

/// Geht im Verlauf eines Tabs zurueck und emittiert shinoho://tab-updated.
#[tauri::command]
pub fn go_back(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab = mgr.go_back(&request.tab_id).map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });

    Ok(tab)
}

/// Geht im Verlauf eines Tabs vorwaerts und emittiert shinoho://tab-updated.
#[tauri::command]
pub fn go_forward(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab = mgr.go_forward(&request.tab_id).map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });

    Ok(tab)
}

/// Laedt einen Tab neu und emittiert shinoho://tab-updated.
#[tauri::command]
pub fn reload_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: ReloadTabRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;

    let tab = mgr
        .reload_tab(&request.tab_id, request.ignore_cache.unwrap_or(false))
        .map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });

    Ok(tab)
}

/// Verschiebt die Position eines Tabs.
#[tauri::command]
pub fn move_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: MoveTabRequest,
) -> Result<(usize, usize), String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let tab_id = request.tab_id.clone();
    let res = mgr
        .move_tab(&tab_id, request.new_index)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_TAB_MOVED,
        TabEvent::TabMoved {
            tab_id,
            from_index: res.0,
            to_index: res.1,
        },
    );

    Ok(res)
}

/// Pinnt oder entpinnt einen Tab.
#[tauri::command]
pub fn set_pinned(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: SetPinnedRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let tab = mgr
        .set_pinned(&request.tab_id, request.is_pinned)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });
    Ok(tab)
}

/// Schaltet einen Tab stumm oder laut.
#[tauri::command]
pub fn set_muted(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: SetMutedRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let tab = mgr
        .set_muted(&request.tab_id, request.is_muted)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });
    Ok(tab)
}

/// Dupliziert einen Tab.
#[tauri::command]
pub fn duplicate_tab(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let new_tab = mgr
        .duplicate_tab(&request.tab_id)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_TAB_CREATED,
        TabEvent::TabCreated {
            tab: new_tab.clone(),
        },
    );
    let _ = app.emit(
        EVENT_TAB_ACTIVATED,
        TabEvent::TabActivated {
            tab_id: new_tab.id.clone(),
        },
    );

    Ok(new_tab)
}

/// Schliesst alle anderen Tabs.
#[tauri::command]
pub fn close_other_tabs(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: TabActionRequest,
) -> Result<Vec<String>, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let closed = mgr
        .close_other_tabs(&request.tab_id)
        .map_err(|e| e.to_string())?;

    for id in &closed {
        let _ = app.emit(EVENT_TAB_CLOSED, TabEvent::TabClosed { tab_id: id.clone() });
    }

    Ok(closed)
}

/// Erstellt eine Tab-Gruppe.
#[tauri::command]
pub fn create_tab_group(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: CreateTabGroupRequest,
) -> Result<TabGroup, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let group = mgr.create_tab_group(request);

    let _ = app.emit(
        EVENT_TAB_GROUP_CREATED,
        TabEvent::TabGroupCreated {
            group: group.clone(),
        },
    );

    Ok(group)
}

/// Gibt alle Tab-Gruppen zurueck.
#[tauri::command]
pub fn get_tab_groups(state: State<'_, TabManagerState>) -> Result<Vec<TabGroup>, String> {
    let mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    Ok(mgr.get_tab_groups())
}

/// Weist einen Tab einer Gruppe zu.
#[tauri::command]
pub fn assign_tab_to_group(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: AssignTabGroupRequest,
) -> Result<TabInfo, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let tab = mgr
        .assign_tab_to_group(request)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(EVENT_TAB_UPDATED, TabEvent::TabUpdated { tab: tab.clone() });
    Ok(tab)
}

/// Aktualisiert eine Tab-Gruppe.
#[tauri::command]
pub fn update_tab_group(
    app: AppHandle,
    state: State<'_, TabManagerState>,
    request: UpdateTabGroupRequest,
) -> Result<TabGroup, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let group = mgr.update_tab_group(request).map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_TAB_GROUP_UPDATED,
        TabEvent::TabGroupUpdated {
            group: group.clone(),
        },
    );
    Ok(group)
}

// --- Seiten-CSS / Userstyles Commands (T-303) ---

/// Registriert einen neuen Userstyle.
#[tauri::command]
pub fn add_userstyle(
    app: AppHandle,
    state: State<'_, UserStylesState>,
    request: AddUserStyleRequest,
) -> Result<UserStyleRule, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let rule = mgr.add_rule(request);

    let _ = app.emit(
        EVENT_USERSTYLE_ADDED,
        StyleEvent::UserStyleAdded { rule: rule.clone() },
    );
    Ok(rule)
}

/// Aktualisiert eine bestehende Userstyle-Regel.
#[tauri::command]
pub fn update_userstyle(
    app: AppHandle,
    state: State<'_, UserStylesState>,
    request: UpdateUserStyleRequest,
) -> Result<UserStyleRule, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let rule = mgr.update_rule(request).map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_USERSTYLE_UPDATED,
        StyleEvent::UserStyleUpdated { rule: rule.clone() },
    );
    Ok(rule)
}

/// Schaltet eine Userstyle-Regel ein oder aus.
#[tauri::command]
pub fn toggle_userstyle(
    app: AppHandle,
    state: State<'_, UserStylesState>,
    request: ToggleUserStyleRequest,
) -> Result<UserStyleRule, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let rule = mgr
        .toggle_rule(&request.id, request.enabled)
        .map_err(|e| e.to_string())?;

    let _ = app.emit(
        EVENT_USERSTYLE_UPDATED,
        StyleEvent::UserStyleUpdated { rule: rule.clone() },
    );
    Ok(rule)
}

/// Loescht eine Userstyle-Regel.
#[tauri::command]
pub fn remove_userstyle(
    app: AppHandle,
    state: State<'_, UserStylesState>,
    request: RemoveUserStyleRequest,
) -> Result<bool, String> {
    let mut mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    let removed = mgr.remove_rule(&request.id);

    if removed {
        let _ = app.emit(
            EVENT_USERSTYLE_REMOVED,
            StyleEvent::UserStyleRemoved { id: request.id },
        );
    }
    Ok(removed)
}

/// Liefert alle registrierten Userstyles.
#[tauri::command]
pub fn get_userstyles(state: State<'_, UserStylesState>) -> Result<Vec<UserStyleRule>, String> {
    let mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    Ok(mgr.get_rules())
}

/// Ermittelt alle passenden Styles und das Injektionsskript fuer eine gegebene URL.
#[tauri::command]
pub fn get_styles_for_url(
    state: State<'_, UserStylesState>,
    request: GetStylesForUrlRequest,
) -> Result<SiteStylesResponse, String> {
    let mgr = state.0.lock().map_err(|e| format!("Lock-Fehler: {e}"))?;
    Ok(mgr.query_styles_for_url(&request.url))
}

/// Injiziert Userstyles fuer den angegebenen Tab.
#[tauri::command]
pub fn inject_styles_for_tab(
    app: AppHandle,
    tab_state: State<'_, TabManagerState>,
    style_state: State<'_, UserStylesState>,
    request: InjectStylesRequest,
) -> Result<InjectionResult, String> {
    let url = if let Some(u) = request.url {
        u
    } else {
        let tab_mgr = tab_state
            .0
            .lock()
            .map_err(|e| format!("Lock-Fehler: {e}"))?;
        let tab = tab_mgr
            .get_tab(&request.tab_id)
            .ok_or_else(|| format!("Tab {} nicht gefunden", request.tab_id))?;
        tab.url
    };

    let style_mgr = style_state
        .0
        .lock()
        .map_err(|e| format!("Lock-Fehler: {e}"))?;
    let result = style_mgr.evaluate_injection(&request.tab_id, &url);

    let _ = app.emit(
        EVENT_STYLES_INJECTED,
        StyleEvent::StylesInjected {
            injection: result.clone(),
        },
    );

    Ok(result)
}
