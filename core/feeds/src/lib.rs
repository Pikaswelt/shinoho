//! Free-Games-Feed für den Shinoho Hub.
//!
//! Quelle ist die offene GamerPower-API ([`GAMERPOWER_URL`]), die Gratis-Aktionen von Epic,
//! Steam, GOG, Prime Gaming und anderen bündelt. Der Abruf selbst passiert im Kern bzw. in der
//! UI (HTTP-Client der App); diese Crate kümmert sich nur um Parsen, Filtern, Sortieren und
//! einen Cache, damit der Hub nicht bei jedem Öffnen ins Netz muss.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Gratis-Spiele (keine DLCs/Beta-Keys), neueste zuerst.
pub const GAMERPOWER_URL: &str = "https://www.gamerpower.com/api/giveaways?type=game&sort-by=date";

/// So lange gilt ein Cache als frisch.
pub const CACHE_TTL_MS: i64 = 3 * 60 * 60 * 1000;

#[derive(Debug, Error)]
pub enum FeedError {
    #[error("Ungültige Antwort der Feed-Quelle: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Shop bzw. Plattform einer Aktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Store {
    Epic,
    Steam,
    Gog,
    Prime,
    Itch,
    Other,
}

impl Store {
    fn from_platforms(platforms: &str) -> Store {
        let p = platforms.to_ascii_lowercase();
        if p.contains("epic") {
            Store::Epic
        } else if p.contains("steam") {
            Store::Steam
        } else if p.contains("gog") {
            Store::Gog
        } else if p.contains("prime") || p.contains("amazon") {
            Store::Prime
        } else if p.contains("itch") {
            Store::Itch
        } else {
            Store::Other
        }
    }
}

/// Ein Gratis-Spiel, so wie es der Hub anzeigt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreeGame {
    pub id: u64,
    pub title: String,
    pub store: Store,
    /// Ursprünglicher Preis, z. B. `"$7.99"`, falls bekannt.
    pub worth: Option<String>,
    pub image: String,
    pub url: String,
    /// Ende der Aktion in ms seit 1970, falls bekannt.
    pub ends_at: Option<i64>,
}

#[derive(Deserialize)]
struct RawGiveaway {
    id: u64,
    title: String,
    worth: Option<String>,
    #[serde(default)]
    thumbnail: String,
    #[serde(default)]
    image: String,
    open_giveaway_url: String,
    #[serde(default)]
    platforms: String,
    end_date: Option<String>,
    #[serde(default)]
    status: String,
}

/// Parst die GamerPower-Antwort. Abgelaufene oder inaktive Aktionen fallen weg.
pub fn parse_gamerpower(json: &str, now_ms: i64) -> Result<Vec<FreeGame>, FeedError> {
    // Ohne Treffer liefert die API ein Objekt statt einer Liste.
    if json.trim_start().starts_with('{') {
        return Ok(Vec::new());
    }
    let raw: Vec<RawGiveaway> = serde_json::from_str(json)?;
    Ok(raw
        .into_iter()
        .filter(|g| g.status.is_empty() || g.status.eq_ignore_ascii_case("active"))
        .map(|g| FreeGame {
            id: g.id,
            title: clean_title(&g.title),
            store: Store::from_platforms(&g.platforms),
            worth: g.worth.filter(|w| !w.is_empty() && w != "N/A"),
            image: if g.image.is_empty() {
                g.thumbnail
            } else {
                g.image
            },
            url: g.open_giveaway_url,
            ends_at: g.end_date.as_deref().and_then(parse_date),
        })
        .filter(|g| g.ends_at.is_none_or(|end| end > now_ms))
        .collect())
}

/// Bald endende Aktionen zuerst, unbefristete am Ende; optional nur bestimmte Shops.
pub fn select(mut games: Vec<FreeGame>, stores: &[Store], limit: usize) -> Vec<FreeGame> {
    if !stores.is_empty() {
        games.retain(|g| stores.contains(&g.store));
    }
    games.sort_by_key(|g| (g.ends_at.is_none(), g.ends_at.unwrap_or(i64::MAX)));
    games.truncate(limit);
    games
}

/// Zwischengespeicherter Feed (z. B. als JSON in den Einstellungen).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedCache {
    pub fetched_at: i64,
    pub games: Vec<FreeGame>,
}

impl FeedCache {
    pub fn new(games: Vec<FreeGame>, now_ms: i64) -> Self {
        Self {
            fetched_at: now_ms,
            games,
        }
    }

    /// `true`, wenn neu abgerufen werden sollte.
    pub fn is_stale(&self, now_ms: i64) -> bool {
        now_ms - self.fetched_at >= CACHE_TTL_MS
    }

    /// Gecachte Spiele ohne die inzwischen abgelaufenen.
    pub fn current(&self, now_ms: i64) -> Vec<FreeGame> {
        self.games
            .iter()
            .filter(|g| g.ends_at.is_none_or(|end| end > now_ms))
            .cloned()
            .collect()
    }
}

fn parse_date(s: &str) -> Option<i64> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|d| DateTime::<Utc>::from_naive_utc_and_offset(d, Utc).timestamp_millis())
}

/// „Fireside Feelings (Steam) Giveaway“ → „Fireside Feelings“.
fn clean_title(title: &str) -> String {
    let t = title.trim();
    let t = t.strip_suffix(" Giveaway").unwrap_or(t);
    match (t.rfind(" ("), t.ends_with(')')) {
        (Some(i), true) => t[..i].to_string(),
        _ => t.to_string(),
    }
}
