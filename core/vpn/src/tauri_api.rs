//! Tauri-Anbindung des VPN (T-403), nur mit Feature `tauri`.
//!
//! **Für die WebViews (T-111):** In `setup` [`init`] aufrufen und danach jede WebView mit
//! `.proxy_url(state.proxy_url().parse()?)` bauen. Alle Tabs laufen dann über die
//! [`Switchboard`]-Weiche; VPN an/aus und Kill-Switch wirken sofort, ohne Tabs neu zu erzeugen.
//!
//! - Befehle: `vpn_status`, `vpn_import_config`, `vpn_connect`, `vpn_disconnect`,
//!   `vpn_set_kill_switch`, `vpn_providers`
//! - Event: `shinoho://vpn-changed` mit [`VpnStatus`]
//!
//! Die importierte Konfiguration (mit privatem Schlüssel) liegt nur im Arbeitsspeicher.

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::providers::{load_bundled_providers, ProviderList};
use crate::switchboard::{Route, Switchboard};
use crate::tunnel::Tunnel;
use crate::wgconfig::WgConfig;

pub const EVENT_VPN_CHANGED: &str = "shinoho://vpn-changed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VpnMode {
    /// VPN aus, Verkehr geht direkt.
    Off,
    Connecting,
    /// Verkehr geht durch den Tunnel.
    On,
    /// Kill-Switch: VPN soll an sein, ist es aber nicht – alles wird blockiert.
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnStatus {
    pub mode: VpnMode,
    /// Name der importierten Konfiguration (z. B. Dateiname).
    pub config_name: Option<String>,
    pub kill_switch: bool,
    /// Proxy, über den die WebViews laufen.
    pub proxy_url: Option<String>,
    /// WebRTC ist gesperrt (VPN an, verbindet gerade oder Kill-Switch greift).
    pub webrtc_blocked: bool,
    pub error: Option<String>,
}

struct Inner {
    config: Option<(String, WgConfig)>,
    tunnel: Option<Tunnel>,
    mode: VpnMode,
    kill_switch: bool,
    error: Option<String>,
}

/// Verwalteter Zustand (`app.manage(VpnState::default())`).
#[derive(Clone)]
pub struct VpnState {
    inner: Arc<Mutex<Inner>>,
    board: Arc<OnceLock<Switchboard>>,
}

impl Default for VpnState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                config: None,
                tunnel: None,
                mode: VpnMode::Off,
                kill_switch: true,
                error: None,
            })),
            board: Arc::new(OnceLock::new()),
        }
    }
}

impl VpnState {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Proxy-URL für alle WebViews, z. B. `socks5://127.0.0.1:PORT`. `None` vor [`init`].
    pub fn proxy_url(&self) -> Option<String> {
        self.board.get().map(Switchboard::proxy_url)
    }

    /// Für `initialization_script` jeder neuen WebView (WebRTC-Leak-Schutz, T-404).
    pub fn webrtc_guard_script(&self) -> String {
        crate::leak_guard::webrtc_guard_script(self.lock().mode != VpnMode::Off)
    }

    pub fn status(&self) -> VpnStatus {
        let inner = self.lock();
        VpnStatus {
            mode: inner.mode,
            config_name: inner.config.as_ref().map(|(n, _)| n.clone()),
            kill_switch: inner.kill_switch,
            proxy_url: self.proxy_url(),
            webrtc_blocked: inner.mode != VpnMode::Off,
            error: inner.error.clone(),
        }
    }

    /// Setzt Modus und Weiche passend zueinander.
    fn apply(&self, inner: &mut Inner, mode: VpnMode) {
        inner.mode = mode;
        let route = match (&inner.tunnel, mode) {
            (Some(t), VpnMode::On) => Route::Tunnel(t.socks_addr()),
            (_, VpnMode::Off) => Route::Direct,
            // Verbinden oder Fehler: nur mit Kill-Switch blockieren.
            _ if inner.kill_switch => Route::Blocked,
            _ => Route::Direct,
        };
        if let Some(board) = self.board.get() {
            board.set_route(route);
        }
    }
}

/// Startet die Weiche. In `setup` aufrufen, **bevor** WebViews erzeugt werden.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> std::io::Result<()> {
    let state = app.state::<VpnState>().inner().clone();
    let board = tauri::async_runtime::block_on(Switchboard::start(Route::Direct))?;
    let _ = state.board.set(board);
    Ok(())
}

fn changed<R: Runtime>(app: &AppHandle<R>, state: &VpnState) -> VpnStatus {
    let status = state.status();
    let _ = app.emit(EVENT_VPN_CHANGED, status.clone());
    status
}

#[tauri::command]
pub fn vpn_status(state: State<'_, VpnState>) -> VpnStatus {
    state.status()
}

#[tauri::command]
pub fn vpn_providers() -> ProviderList {
    load_bundled_providers()
}

/// Liest eine WireGuard-`.conf` ein (verbindet noch nicht).
#[tauri::command]
pub fn vpn_import_config<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, VpnState>,
    name: String,
    text: String,
) -> Result<VpnStatus, String> {
    let config: WgConfig = text.parse().map_err(|e| format!("{e}"))?;
    {
        let mut inner = state.lock();
        inner.config = Some((name, config));
        inner.error = None;
    }
    Ok(changed(&app, &state))
}

#[tauri::command]
pub async fn vpn_connect<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, VpnState>,
) -> Result<VpnStatus, String> {
    let config = {
        let mut inner = state.lock();
        let Some((_, config)) = inner.config.clone() else {
            return Err("Keine VPN-Konfiguration importiert".into());
        };
        if let Some(old) = inner.tunnel.take() {
            old.stop();
        }
        inner.error = None;
        state.apply(&mut inner, VpnMode::Connecting);
        config
    };
    changed(&app, &state);

    let result = Tunnel::start(&config).await;
    {
        let mut inner = state.lock();
        match result {
            Ok(tunnel) => {
                inner.tunnel = Some(tunnel);
                state.apply(&mut inner, VpnMode::On);
            }
            Err(e) => {
                inner.error = Some(e.to_string());
                let mode = if inner.kill_switch {
                    VpnMode::Blocked
                } else {
                    VpnMode::Off
                };
                state.apply(&mut inner, mode);
            }
        }
    }
    Ok(changed(&app, &state))
}

#[tauri::command]
pub fn vpn_disconnect<R: Runtime>(app: AppHandle<R>, state: State<'_, VpnState>) -> VpnStatus {
    {
        let mut inner = state.lock();
        if let Some(t) = inner.tunnel.take() {
            t.stop();
        }
        inner.error = None;
        state.apply(&mut inner, VpnMode::Off);
    }
    changed(&app, &state)
}

#[tauri::command]
pub fn vpn_set_kill_switch<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, VpnState>,
    enabled: bool,
) -> VpnStatus {
    {
        let mut inner = state.lock();
        inner.kill_switch = enabled;
        let mode = match inner.mode {
            VpnMode::Blocked if !enabled => VpnMode::Off,
            m => m,
        };
        state.apply(&mut inner, mode);
    }
    changed(&app, &state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn modes_drive_the_switchboard() {
        let state = VpnState::default();
        state
            .board
            .set(Switchboard::start(Route::Direct).await.unwrap())
            .ok()
            .unwrap();
        let board = || state.board.get().unwrap().route();
        assert!(state
            .proxy_url()
            .unwrap()
            .starts_with("socks5://127.0.0.1:"));

        // Verbindungsaufbau mit Kill-Switch: blockieren, nichts geht direkt raus.
        state.apply(&mut state.lock(), VpnMode::Connecting);
        assert_eq!(board(), Route::Blocked);
        // Ohne Kill-Switch darf es währenddessen direkt weitergehen.
        state.lock().kill_switch = false;
        state.apply(&mut state.lock(), VpnMode::Connecting);
        assert_eq!(board(), Route::Direct);
        // Aus = direkt.
        state.apply(&mut state.lock(), VpnMode::Off);
        assert_eq!(board(), Route::Direct);
        assert_eq!(state.status().mode, VpnMode::Off);
    }

    #[test]
    fn status_json_shape() {
        let state = VpnState::default();
        let json = serde_json::to_string(&state.status()).unwrap();
        assert_eq!(
            json,
            r#"{"mode":"off","configName":null,"killSwitch":true,"proxyUrl":null,"webrtcBlocked":false,"error":null}"#
        );
    }
}
