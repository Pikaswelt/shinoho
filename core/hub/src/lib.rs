//! Persistenter Zustand der Startseite (Hub).
//!
//! Der Hub-Zustand wird als JSON in den Einstellungen von `shinoho-storage` abgelegt.
//! Die Felder entsprechen dem TypeScript-Typ `HubState` in der UI (camelCase), damit die
//! UI den Zustand unverändert übernehmen und zurückschicken kann.

use serde::{Deserialize, Serialize};
use shinoho_storage::{Storage, StorageError};

const HUB_STATE_KEY: &str = "hub.state";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WallpaperSettings {
    pub active_id: String,
    pub custom_url: Option<String>,
    pub blur: f64,
    pub brightness: f64,
    pub opacity: f64,
    pub particles_enabled: bool,
    pub particle_density: u32,
    pub sync_with_theme: Option<bool>,
}

impl Default for WallpaperSettings {
    fn default() -> Self {
        Self {
            active_id: "sakura".to_string(),
            custom_url: None,
            blur: 0.0,
            brightness: 1.0,
            opacity: 1.0,
            particles_enabled: true,
            particle_density: 40,
            sync_with_theme: Some(true),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeedDialItem {
    pub id: String,
    pub title: String,
    pub url: String,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub clicks: u64,
    pub pinned: bool,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HubWidgets {
    pub anime_countdown: bool,
    pub free_games: bool,
    pub mascot: bool,
    pub clock: bool,
}

impl Default for HubWidgets {
    fn default() -> Self {
        Self {
            anime_countdown: true,
            free_games: true,
            mascot: true,
            clock: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HubState {
    pub wallpaper: WallpaperSettings,
    pub speed_dial: Vec<SpeedDialItem>,
    pub selected_search_engine: String,
    pub widgets: HubWidgets,
}

impl Default for HubState {
    fn default() -> Self {
        Self {
            wallpaper: WallpaperSettings::default(),
            speed_dial: Vec::new(),
            selected_search_engine: "duckduckgo".to_string(),
            widgets: HubWidgets::default(),
        }
    }
}

/// Liest den gespeicherten Hub-Zustand. `None`, wenn noch nichts gespeichert wurde.
pub fn load(storage: &Storage) -> Result<Option<HubState>, StorageError> {
    storage.get_setting_json(HUB_STATE_KEY)
}

/// Speichert den Hub-Zustand vollständig (ersetzt den bisherigen Stand).
pub fn save(storage: &Storage, state: &HubState) -> Result<(), StorageError> {
    storage.set_setting_json(HUB_STATE_KEY, state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> SpeedDialItem {
        SpeedDialItem {
            id: id.to_string(),
            title: format!("Seite {id}"),
            url: format!("https://example.com/{id}"),
            clicks: 3,
            pinned: true,
            created_at: 1_700_000_000_000,
            ..Default::default()
        }
    }

    #[test]
    fn load_returns_none_when_nothing_saved() {
        let storage = Storage::open_in_memory().unwrap();
        assert_eq!(load(&storage).unwrap(), None);
    }

    #[test]
    fn save_then_load_roundtrips() {
        let storage = Storage::open_in_memory().unwrap();
        let mut state = HubState::default();
        state.wallpaper.active_id = "nebula".to_string();
        state.wallpaper.blur = 4.5;
        state.speed_dial.push(item("a"));
        state.selected_search_engine = "google".to_string();
        state.widgets.clock = false;

        save(&storage, &state).unwrap();
        assert_eq!(load(&storage).unwrap(), Some(state));
    }

    #[test]
    fn uses_camel_case_keys_for_the_ui() {
        let json = serde_json::to_value(HubState::default()).unwrap();
        assert!(json.get("speedDial").is_some());
        assert!(json.get("selectedSearchEngine").is_some());
        assert!(json["wallpaper"].get("particleDensity").is_some());
        assert!(json["widgets"].get("animeCountdown").is_some());
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let state: HubState = serde_json::from_str(r#"{"selectedSearchEngine":"bing"}"#).unwrap();
        assert_eq!(state.selected_search_engine, "bing");
        assert_eq!(state.wallpaper, WallpaperSettings::default());
        assert!(state.widgets.mascot);
    }
}
