//! Tauri-Anbindung des Gaming-Modus (T-603), nur mit Feature `tauri`.
//!
//! - Befehle: `gaming_status`, `gaming_set_auto`, `gaming_set_rules`, `control_stats` (T-202)
//! - Event: `shinoho://gaming-changed` mit [`GamingStatus`], sobald ein Spiel startet oder endet
//! - [`start_watcher`] prüft die Prozessliste alle 5 Sekunden im Hintergrund

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::gaming::{GameRules, GameWatcher, GamingEvent};
use crate::monitor::{ResourceMonitor, ResourceStats};
use crate::sleeper::{SleepChanges, TabSleeper, TabState};
use crate::Preset;

pub const EVENT_GAMING_CHANGED: &str = "shinoho://gaming-changed";
/// Tabs einfrieren/aufwecken/drosseln (T-203), Inhalt: [`SleepChanges`].
pub const EVENT_TAB_SLEEP: &str = "shinoho://tab-sleep";
const SLEEP_INTERVAL: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_secs(5);

/// Zustand, den die UI anzeigt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamingStatus {
    /// Ein Spiel läuft gerade.
    pub active: bool,
    pub game: Option<String>,
    /// Bei laufendem Spiel automatisch auf das Gaming-Preset wechseln.
    pub auto: bool,
    /// Das Preset, das gerade gilt.
    pub preset: Preset,
}

#[derive(Debug)]
struct Inner {
    game: Option<String>,
    auto: bool,
    rules: GameRules,
    /// Preset des Nutzers, wenn kein Spiel läuft.
    base_preset: Preset,
}

impl Inner {
    fn status(&self) -> GamingStatus {
        let active = self.game.is_some();
        GamingStatus {
            active,
            game: self.game.clone(),
            auto: self.auto,
            preset: if active && self.auto {
                Preset::Gaming
            } else {
                self.base_preset
            },
        }
    }
}

/// Verwalteter Zustand für Tauri (`app.manage(GamingState::default())`).
#[derive(Debug, Clone)]
pub struct GamingState(Arc<Mutex<Inner>>);

impl Default for GamingState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Inner {
            game: None,
            auto: true,
            rules: GameRules::default(),
            base_preset: Preset::default(),
        })))
    }
}

impl GamingState {
    pub fn status(&self) -> GamingStatus {
        self.lock().status()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        // Ein Panik-Fall in einem anderen Thread soll den Gaming-Modus nicht lahmlegen.
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Netzwerk-Limiter (T-204) passend zum gerade geltenden Preset setzen.
fn apply_bandwidth<R: Runtime>(app: &AppHandle<R>, status: &GamingStatus) {
    if let Some(vpn) = app.try_state::<shinoho_vpn::tauri_api::VpnState>() {
        // Die Bandbreite hängt nicht vom Geräte-RAM ab.
        vpn.set_bandwidth_limit_kbps(status.preset.limits(0).net_limit_kbps);
    }
}

#[tauri::command]
pub fn gaming_status(state: State<'_, GamingState>) -> GamingStatus {
    state.status()
}

#[tauri::command]
pub fn gaming_set_auto<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, GamingState>,
    enabled: bool,
) -> GamingStatus {
    let status = {
        let mut inner = state.lock();
        inner.auto = enabled;
        inner.status()
    };
    apply_bandwidth(&app, &status);
    let _ = app.emit(EVENT_GAMING_CHANGED, status.clone());
    status
}

#[tauri::command]
pub fn gaming_set_rules(state: State<'_, GamingState>, rules: GameRules) -> GamingStatus {
    let mut inner = state.lock();
    inner.rules = rules;
    inner.status()
}

/// Verwalteter Zustand für die Messung (`app.manage(MonitorState::default())`).
#[derive(Default)]
pub struct MonitorState(Mutex<ResourceMonitor>);

/// RAM und CPU des ganzen Shinoho-Prozessbaums (T-202). Der erste Aufruf liefert CPU = 0.
#[tauri::command]
pub fn control_stats(state: State<'_, MonitorState>) -> ResourceStats {
    state.0.lock().unwrap_or_else(|e| e.into_inner()).sample()
}

/// Zustand des Tab-Schlafs (`app.manage(SleeperState::default())`).
#[derive(Default, Clone)]
pub struct SleeperState(Arc<Mutex<TabSleeper>>);

/// IDs der Tabs, die gerade schlafen.
#[tauri::command]
pub fn control_asleep_tabs(state: State<'_, SleeperState>) -> Vec<String> {
    let s = state.0.lock().unwrap_or_else(|e| e.into_inner());
    let mut ids: Vec<String> = s.asleep().map(str::to_string).collect();
    ids.sort();
    ids
}

/// Ein Tab wird benutzt: sofort wecken (meldet `shinoho://tab-sleep` mit `wake`).
#[tauri::command]
pub fn control_touch_tab<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, SleeperState>,
    tab_id: String,
) {
    let woke = state
        .0
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .touch(&tab_id);
    if woke {
        let changes = SleepChanges {
            wake: vec![tab_id],
            ..Default::default()
        };
        let _ = app.emit(EVENT_TAB_SLEEP, changes);
    }
}

/// Startet die Tab-Schlaf-Schleife (alle 15 s). Braucht `TabManagerState`, `GamingState`
/// und `SleeperState` im verwalteten Zustand.
pub fn start_sleeper<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    let sleeper = app.state::<SleeperState>().inner().clone();
    let gaming = app.state::<GamingState>().inner().clone();
    std::thread::Builder::new()
        .name("shinoho-tab-sleep".into())
        .spawn(move || {
            let mut monitor = ResourceMonitor::new();
            loop {
                std::thread::sleep(SLEEP_INTERVAL);
                let stats = monitor.sample();
                let tabs: Vec<TabState> = {
                    let tm = app.state::<shinoho_core::api::TabManagerState>();
                    let mgr = tm.0.lock().unwrap_or_else(|e| e.into_inner());
                    let active = mgr.get_active_tab().map(|t| t.id);
                    mgr.get_tabs()
                        .into_iter()
                        .map(|t| TabState {
                            active: active.as_deref() == Some(t.id.as_str()),
                            pinned: t.is_pinned,
                            last_accessed_at: t.last_accessed_at,
                            audible: false,
                            capturing: false,
                            id: t.id,
                        })
                        .collect()
                };
                let limits = gaming.status().preset.limits(stats.device_ram_mb);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                let changes = sleeper.0.lock().unwrap_or_else(|e| e.into_inner()).update(
                    &limits,
                    &tabs,
                    stats.total_ram_mb,
                    now,
                );
                if !changes.is_empty() {
                    let _ = app.emit(EVENT_TAB_SLEEP, changes);
                }
            }
        })
        .expect("Tab-Schlaf konnte nicht gestartet werden");
}

/// Startet den Hintergrund-Wächter. `GamingState` muss vorher mit `manage` registriert sein.
pub fn start_watcher<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    let state = app.state::<GamingState>().inner().clone();
    std::thread::Builder::new()
        .name("shinoho-gaming".into())
        .spawn(move || {
            let mut watcher = GameWatcher::new(state.lock().rules.clone());
            loop {
                watcher.set_rules(state.lock().rules.clone());
                if let Some(event) = watcher.poll() {
                    let status = {
                        let mut inner = state.lock();
                        inner.game = match event {
                            GamingEvent::Started(game) => Some(game),
                            GamingEvent::Stopped => None,
                        };
                        inner.status()
                    };
                    apply_bandwidth(&app, &status);
                    let _ = app.emit(EVENT_GAMING_CHANGED, status);
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        })
        .expect("Gaming-Wächter konnte nicht gestartet werden");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_switches_only_with_auto() {
        let state = GamingState::default();
        assert_eq!(state.status().preset, Preset::Balanced);
        state.lock().game = Some("cs2".into());
        let s = state.status();
        assert!(s.active);
        assert_eq!(s.preset, Preset::Gaming);
        state.lock().auto = false;
        assert_eq!(state.status().preset, Preset::Balanced);
    }

    #[test]
    fn status_json_shape() {
        let state = GamingState::default();
        state.lock().game = Some("Minecraft".into());
        let json = serde_json::to_string(&state.status()).unwrap();
        assert_eq!(
            json,
            r#"{"active":true,"game":"Minecraft","auto":true,"preset":"gaming"}"#
        );
    }
}
