//! Laufzeit-Dienste des Browsers: Storage, Inkognito, Adblock und Downloads.
//!
//! Die Dienste werden beim Start in `setup` erzeugt und als Tauri-State verwaltet.
//! Download-Events werden aus dem Kanal des DownloadManagers an das Frontend weitergereicht.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use shinoho_adblock::AdblockEngine;
use shinoho_downloads::events::{
    EVENT_DOWNLOAD_CANCELLED, EVENT_DOWNLOAD_COMPLETED, EVENT_DOWNLOAD_FAILED,
    EVENT_DOWNLOAD_PAUSED, EVENT_DOWNLOAD_PROGRESS, EVENT_DOWNLOAD_RESUMED, EVENT_DOWNLOAD_STARTED,
};
use shinoho_downloads::{DownloadEvent, DownloadManager};
use shinoho_incognito::{IncognitoConfig, IncognitoManager};
use shinoho_storage::Storage;
use tauri::{AppHandle, Emitter};

/// Alle Backend-Dienste, die die UI ueber Tauri-Commands anspricht.
pub struct AppServices {
    pub storage: Arc<Storage>,
    pub incognito: IncognitoManager,
    pub adblock: AdblockEngine,
    pub downloads: DownloadManager,
}

impl AppServices {
    /// Oeffnet die SQLite-Datenbank unter `db_path` und initialisiert alle Dienste.
    pub fn new(db_path: &Path, download_dir: PathBuf) -> Result<Self, String> {
        let storage = Arc::new(Storage::open(db_path).map_err(|e| e.to_string())?);
        Ok(Self {
            downloads: DownloadManager::new(Some(storage.clone()), Some(download_dir)),
            storage,
            incognito: IncognitoManager::new(IncognitoConfig::default()),
            adblock: AdblockEngine::with_default_rules(),
        })
    }
}

/// Leitet Download-Events des DownloadManagers als Tauri-Events an alle Fenster weiter.
pub fn forward_download_events(app: AppHandle, downloads: &DownloadManager) {
    let mut rx = downloads.subscribe();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let name = event_name(&event);
                    let _ = app.emit(name, event);
                }
                // Bei Lag werden verpasste Events uebersprungen, der Kanal bleibt aktiv.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                // Kanal geschlossen: der DownloadManager wurde beendet.
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn event_name(event: &DownloadEvent) -> &'static str {
    match event {
        DownloadEvent::Started(_) => EVENT_DOWNLOAD_STARTED,
        DownloadEvent::Progress(_) => EVENT_DOWNLOAD_PROGRESS,
        DownloadEvent::Paused { .. } => EVENT_DOWNLOAD_PAUSED,
        DownloadEvent::Resumed { .. } => EVENT_DOWNLOAD_RESUMED,
        DownloadEvent::Completed(_) => EVENT_DOWNLOAD_COMPLETED,
        DownloadEvent::Cancelled { .. } => EVENT_DOWNLOAD_CANCELLED,
        DownloadEvent::Failed { .. } => EVENT_DOWNLOAD_FAILED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_open_on_disk_database() {
        let dir = tempfile::tempdir().expect("tempdir");
        let services = AppServices::new(&dir.path().join("shinoho.sqlite3"), dir.path().into())
            .expect("services");
        assert!(
            services
                .storage
                .get_all_settings()
                .expect("settings")
                .is_empty()
        );
    }

    #[test]
    fn adblock_is_enabled_by_default() {
        assert!(AdblockEngine::with_default_rules().is_enabled());
    }

    #[test]
    fn event_names_match_download_events() {
        let item = shinoho_downloads::DownloadItem {
            id: "1".into(),
            url: "https://example.com/a".into(),
            target_path: "/tmp/a".into(),
            filename: "a".into(),
            total_bytes: None,
            received_bytes: 0,
            state: shinoho_storage::DownloadState::Downloading,
            error_message: None,
            speed_bps: 0,
            started_at: 0,
            completed_at: None,
        };
        assert_eq!(
            event_name(&DownloadEvent::Started(item)),
            EVENT_DOWNLOAD_STARTED
        );
        assert_eq!(
            event_name(&DownloadEvent::Cancelled { id: "1".into() }),
            EVENT_DOWNLOAD_CANCELLED
        );
    }
}
