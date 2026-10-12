//! Schnittstelle Kern <-> UI (core/api v0).
//!
//! Stellt Tauri-Commands und Event-Konstanten fuer Tabs und Navigation bereit,
//! abgestimmt auf core/api/types.ts und ui/src/api/tabs.ts.

use crate::tabs::{
    CreateTabRequest, NavigateTabRequest, ReloadTabRequest, TabActionRequest, TabEvent, TabInfo,
    TabManager,
};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub const EVENT_TAB_CREATED: &str = "shinoho://tab-created";
pub const EVENT_TAB_UPDATED: &str = "shinoho://tab-updated";
pub const EVENT_TAB_CLOSED: &str = "shinoho://tab-closed";
pub const EVENT_TAB_ACTIVATED: &str = "shinoho://tab-activated";

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
