//! Lokale Erfolge („100 Tabs geschlossen“, „Erste Mod installiert“) mit XP und Leveln.
//!
//! Der Kern meldet Ereignisse über [`Achievements::record`]. Zurück kommen die dabei neu
//! freigeschalteten Erfolge, damit die UI eine Benachrichtigung zeigen kann. Zähler und
//! freigeschaltete Erfolge liegen in den Einstellungen von `shinoho-storage`; im
//! Inkognito-Modus wird einfach nichts gemeldet.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shinoho_storage::{Storage, StorageError};

const COUNTERS_KEY: &str = "achievements.counters";
const UNLOCKED_KEY: &str = "achievements.unlocked";

/// Ereignisse, die Erfolge voranbringen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    TabOpened,
    TabClosed,
    PageVisited,
    BookmarkAdded,
    AdBlocked,
    ModInstalled,
    ThemeChanged,
    IncognitoOpened,
    VpnConnected,
    DownloadFinished,
    AnimeEpisodeTracked,
}

impl Event {
    fn key(self) -> &'static str {
        match self {
            Event::TabOpened => "tab_opened",
            Event::TabClosed => "tab_closed",
            Event::PageVisited => "page_visited",
            Event::BookmarkAdded => "bookmark_added",
            Event::AdBlocked => "ad_blocked",
            Event::ModInstalled => "mod_installed",
            Event::ThemeChanged => "theme_changed",
            Event::IncognitoOpened => "incognito_opened",
            Event::VpnConnected => "vpn_connected",
            Event::DownloadFinished => "download_finished",
            Event::AnimeEpisodeTracked => "anime_episode_tracked",
        }
    }
}

/// Ein Erfolg. Titel und Beschreibung stehen in den i18n-Dateien der UI unter
/// `achievements.<id>.title` / `achievements.<id>.description`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Definition {
    pub id: &'static str,
    pub event: Event,
    pub goal: u64,
    pub xp: u64,
}

/// Alle Erfolge der ersten Version.
pub const DEFINITIONS: &[Definition] = &[
    Definition {
        id: "first_steps",
        event: Event::PageVisited,
        goal: 1,
        xp: 10,
    },
    Definition {
        id: "explorer",
        event: Event::PageVisited,
        goal: 1_000,
        xp: 100,
    },
    Definition {
        id: "tab_hoarder",
        event: Event::TabOpened,
        goal: 100,
        xp: 50,
    },
    Definition {
        id: "tab_cleaner",
        event: Event::TabClosed,
        goal: 100,
        xp: 50,
    },
    Definition {
        id: "collector",
        event: Event::BookmarkAdded,
        goal: 10,
        xp: 30,
    },
    Definition {
        id: "ad_slayer",
        event: Event::AdBlocked,
        goal: 1_000,
        xp: 100,
    },
    Definition {
        id: "first_mod",
        event: Event::ModInstalled,
        goal: 1,
        xp: 50,
    },
    Definition {
        id: "stylist",
        event: Event::ThemeChanged,
        goal: 5,
        xp: 30,
    },
    Definition {
        id: "ninja",
        event: Event::IncognitoOpened,
        goal: 1,
        xp: 20,
    },
    Definition {
        id: "tunnel_runner",
        event: Event::VpnConnected,
        goal: 1,
        xp: 30,
    },
    Definition {
        id: "hoarder",
        event: Event::DownloadFinished,
        goal: 50,
        xp: 50,
    },
    Definition {
        id: "binge_watcher",
        event: Event::AnimeEpisodeTracked,
        goal: 100,
        xp: 100,
    },
];

/// Themes, die ab einem Level freigeschaltet werden.
pub const LEVEL_REWARDS: &[(u32, &str)] =
    &[(2, "sakura-night"), (4, "neon-arcade"), (6, "shino-gold")];

/// Fortschritt eines Erfolgs für die UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub id: &'static str,
    pub current: u64,
    pub goal: u64,
    pub xp: u64,
    /// Zeitpunkt der Freischaltung in ms seit 1970, sonst `None`.
    pub unlocked_at: Option<i64>,
}

/// Gesamtstand: XP, Level und freigeschaltete Themes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub xp: u64,
    pub level: u32,
    /// XP, die bis zum nächsten Level noch fehlen.
    pub xp_to_next_level: u64,
    pub unlocked_themes: Vec<&'static str>,
}

/// XP, die insgesamt für `level` nötig sind (Level 1 = 0 XP, dann 100, 300, 600, …).
pub fn xp_for_level(level: u32) -> u64 {
    let n = u64::from(level.saturating_sub(1));
    50 * n * (n + 1)
}

/// Level zu einer XP-Summe.
pub fn level_for_xp(xp: u64) -> u32 {
    let mut level = 1;
    while xp >= xp_for_level(level + 1) {
        level += 1;
    }
    level
}

pub struct Achievements<'a> {
    storage: &'a Storage,
}

impl<'a> Achievements<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Zählt ein Ereignis `amount`-mal und gibt die dabei neu freigeschalteten Erfolge zurück.
    pub fn record(&self, event: Event, amount: u64) -> Result<Vec<Definition>, StorageError> {
        let mut counters = self.counters()?;
        let count = counters.entry(event.key().to_string()).or_insert(0);
        *count = count.saturating_add(amount);
        let count = *count;
        self.storage.set_setting_json(COUNTERS_KEY, &counters)?;

        let mut unlocked = self.unlocked()?;
        let now = chrono::Utc::now().timestamp_millis();
        let fresh: Vec<Definition> = DEFINITIONS
            .iter()
            .filter(|d| d.event == event && count >= d.goal && !unlocked.contains_key(d.id))
            .copied()
            .collect();
        if !fresh.is_empty() {
            for d in &fresh {
                unlocked.insert(d.id.to_string(), now);
            }
            self.storage.set_setting_json(UNLOCKED_KEY, &unlocked)?;
        }
        Ok(fresh)
    }

    /// Fortschritt aller Erfolge in der Reihenfolge von [`DEFINITIONS`].
    pub fn progress(&self) -> Result<Vec<Progress>, StorageError> {
        let counters = self.counters()?;
        let unlocked = self.unlocked()?;
        Ok(DEFINITIONS
            .iter()
            .map(|d| Progress {
                id: d.id,
                current: counters
                    .get(d.event.key())
                    .copied()
                    .unwrap_or(0)
                    .min(d.goal),
                goal: d.goal,
                xp: d.xp,
                unlocked_at: unlocked.get(d.id).copied(),
            })
            .collect())
    }

    pub fn summary(&self) -> Result<Summary, StorageError> {
        let unlocked = self.unlocked()?;
        let xp: u64 = DEFINITIONS
            .iter()
            .filter(|d| unlocked.contains_key(d.id))
            .map(|d| d.xp)
            .sum();
        let level = level_for_xp(xp);
        Ok(Summary {
            xp,
            level,
            xp_to_next_level: xp_for_level(level + 1) - xp,
            unlocked_themes: LEVEL_REWARDS
                .iter()
                .filter(|(min, _)| level >= *min)
                .map(|(_, theme)| *theme)
                .collect(),
        })
    }

    /// Setzt alle Zähler und Erfolge zurück.
    pub fn reset(&self) -> Result<(), StorageError> {
        self.storage.delete_setting(COUNTERS_KEY)?;
        self.storage.delete_setting(UNLOCKED_KEY)?;
        Ok(())
    }

    fn counters(&self) -> Result<HashMap<String, u64>, StorageError> {
        Ok(self
            .storage
            .get_setting_json(COUNTERS_KEY)?
            .unwrap_or_default())
    }

    fn unlocked(&self) -> Result<HashMap<String, i64>, StorageError> {
        Ok(self
            .storage
            .get_setting_json(UNLOCKED_KEY)?
            .unwrap_or_default())
    }
}
