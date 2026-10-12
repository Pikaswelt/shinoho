//! Release-Kalender: welche Folgen laufen diese Woche?

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{data_of, missing, pick_title, AnimeError, GraphQlRequest, TitleLanguage};

const SCHEDULE_QUERY: &str = "query ($from: Int, $to: Int, $page: Int) {
  Page(page: $page, perPage: 50) {
    pageInfo { hasNextPage }
    airingSchedules(airingAt_greater: $from, airingAt_lesser: $to, sort: TIME) {
      episode
      airingAt
      media { id isAdult siteUrl title { romaji english native } coverImage { large } }
    }
  }
}";

const DAY_SECS: i64 = 24 * 60 * 60;

/// Sendeplan zwischen `from` und `to` (Sekunden seit 1970), Seite `page` (ab 1).
pub fn schedule_request(from: i64, to: i64, page: u32) -> GraphQlRequest {
    GraphQlRequest {
        query: SCHEDULE_QUERY,
        variables: json!({ "from": from, "to": to, "page": page.max(1) }),
    }
}

/// Eine Ausstrahlung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Airing {
    pub media_id: u64,
    pub title: String,
    pub episode: u32,
    /// Sendezeit in Sekunden seit 1970.
    pub airing_at: i64,
    pub cover: Option<String>,
    pub url: Option<String>,
    /// Steht auf der Watchlist des Nutzers.
    pub on_watchlist: bool,
}

/// Eine Seite des Sendeplans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePage {
    pub items: Vec<Airing>,
    pub has_next_page: bool,
}

/// Wertet eine Seite aus. Inhalte ab 18 fallen weg; `watchlist` markiert eigene Anime.
pub fn parse_schedule(
    body: &str,
    lang: TitleLanguage,
    watchlist: &[u64],
) -> Result<SchedulePage, AnimeError> {
    let data = data_of(body)?;
    let page = &data["Page"];
    let raw = page["airingSchedules"]
        .as_array()
        .ok_or_else(|| missing("Page.airingSchedules"))?;
    let items = raw
        .iter()
        .filter(|a| !a["media"]["isAdult"].as_bool().unwrap_or(false))
        .filter_map(|a| {
            let media = &a["media"];
            let media_id = media["id"].as_u64()?;
            Some(Airing {
                media_id,
                title: pick_title(&media["title"], lang),
                episode: a["episode"].as_u64()? as u32,
                airing_at: a["airingAt"].as_i64()?,
                cover: str_of(&media["coverImage"]["large"]),
                url: str_of(&media["siteUrl"]),
                on_watchlist: watchlist.contains(&media_id),
            })
        })
        .collect();
    Ok(SchedulePage {
        items,
        has_next_page: page["pageInfo"]["hasNextPage"].as_bool().unwrap_or(false),
    })
}

/// Ein Tag im Kalender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Day {
    /// Beginn des Tages (lokale Zeit) in Sekunden seit 1970.
    pub starts_at: i64,
    pub items: Vec<Airing>,
}

/// Teilt Ausstrahlungen nach lokalen Tagen auf (`utc_offset` in Sekunden, z. B. 7200 für
/// MESZ). Eigene Anime stehen innerhalb eines Tages vorne, sonst nach Uhrzeit.
pub fn group_by_day(mut items: Vec<Airing>, utc_offset: i64) -> Vec<Day> {
    items.sort_by_key(|a| (a.airing_at, !a.on_watchlist));
    let mut days: Vec<Day> = Vec::new();
    for item in items {
        let local = item.airing_at + utc_offset;
        let starts_at = local - local.rem_euclid(DAY_SECS) - utc_offset;
        match days.last_mut() {
            Some(day) if day.starts_at == starts_at => day.items.push(item),
            _ => days.push(Day {
                starts_at,
                items: vec![item],
            }),
        }
    }
    for day in &mut days {
        day.items.sort_by_key(|a| (!a.on_watchlist, a.airing_at));
    }
    days
}

fn str_of(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}
