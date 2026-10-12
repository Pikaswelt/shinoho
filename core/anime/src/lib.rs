//! Anime-Tracker auf Basis von AniList.
//!
//! Die Crate baut die GraphQL-Anfragen und wertet die Antworten aus; das Senden übernimmt der
//! HTTP-Client der App (`POST` an [`API_URL`] mit `Authorization: Bearer <token>`).
//! Login läuft über den Implicit Grant von AniList: [`authorize_url`] im Browser-Tab öffnen,
//! die Weiterleitung abfangen und mit [`parse_redirect`] den Token herauslösen.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const API_URL: &str = "https://graphql.anilist.co";
pub const AUTHORIZE_URL: &str = "https://anilist.co/api/v2/oauth/authorize";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AnimeError {
    #[error("Kein Zugangstoken in der Weiterleitung")]
    MissingToken,
    #[error("AniList meldet: {0}")]
    Api(String),
    #[error("Unerwartete Antwort von AniList: {0}")]
    Format(String),
}

impl From<serde_json::Error> for AnimeError {
    fn from(e: serde_json::Error) -> Self {
        AnimeError::Format(e.to_string())
    }
}

/// URL für den Login-Tab (Implicit Grant).
pub fn authorize_url(client_id: &str) -> String {
    format!("{AUTHORIZE_URL}?client_id={client_id}&response_type=token")
}

/// Zugangsdaten nach dem Login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub access_token: String,
    /// Gültigkeit in Sekunden ab Login.
    pub expires_in: Option<u64>,
}

/// Holt den Token aus der Weiterleitungs-URL (`…#access_token=…&token_type=Bearer&expires_in=…`).
pub fn parse_redirect(url: &str) -> Result<Session, AnimeError> {
    let fragment = url.split_once('#').map(|(_, f)| f).unwrap_or_default();
    let mut token = None;
    let mut expires_in = None;
    for pair in fragment.split('&') {
        match pair.split_once('=') {
            Some(("access_token", v)) if !v.is_empty() => token = Some(v.to_string()),
            Some(("expires_in", v)) => expires_in = v.parse().ok(),
            _ => {}
        }
    }
    token
        .map(|access_token| Session {
            access_token,
            expires_in,
        })
        .ok_or(AnimeError::MissingToken)
}

/// Fertiger Request-Body für `POST` an [`API_URL`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GraphQlRequest {
    pub query: &'static str,
    pub variables: Value,
}

const VIEWER_QUERY: &str = "query { Viewer { id name avatar { medium } } }";

const WATCHING_QUERY: &str = "query ($userId: Int) {
  MediaListCollection(userId: $userId, type: ANIME, status_in: [CURRENT, REPEATING]) {
    lists { entries {
      progress
      status
      media {
        id
        episodes
        siteUrl
        title { romaji english native }
        coverImage { large }
        nextAiringEpisode { episode airingAt }
      }
    } }
  }
}";

const SAVE_PROGRESS_MUTATION: &str =
    "mutation ($mediaId: Int, $progress: Int, $status: MediaListStatus) {
  SaveMediaListEntry(mediaId: $mediaId, progress: $progress, status: $status) { id progress status }
}";

/// Wer ist eingeloggt?
pub fn viewer_request() -> GraphQlRequest {
    GraphQlRequest {
        query: VIEWER_QUERY,
        variables: json!({}),
    }
}

/// Was schaut der Nutzer gerade?
pub fn watching_request(user_id: u64) -> GraphQlRequest {
    GraphQlRequest {
        query: WATCHING_QUERY,
        variables: json!({ "userId": user_id }),
    }
}

/// Folgen-Fortschritt speichern. Ist die letzte Folge erreicht, wird der Eintrag als
/// abgeschlossen markiert.
pub fn save_progress_request(entry: &AnimeEntry, progress: u32) -> GraphQlRequest {
    let completed = entry.episodes.is_some_and(|total| progress >= total);
    GraphQlRequest {
        query: SAVE_PROGRESS_MUTATION,
        variables: json!({
            "mediaId": entry.media_id,
            "progress": progress,
            "status": if completed { "COMPLETED" } else { "CURRENT" },
        }),
    }
}

/// Eingeloggter AniList-Nutzer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viewer {
    pub id: u64,
    pub name: String,
    pub avatar: Option<String>,
}

/// Nächste Folge laut Sendeplan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextEpisode {
    pub episode: u32,
    /// Sendezeit in Sekunden seit 1970.
    pub airing_at: i64,
}

/// Ein Anime auf der Watchlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimeEntry {
    pub media_id: u64,
    pub title: String,
    pub progress: u32,
    pub episodes: Option<u32>,
    pub cover: Option<String>,
    pub url: Option<String>,
    pub next_episode: Option<NextEpisode>,
}

impl AnimeEntry {
    /// Ausgestrahlte, aber noch nicht gesehene Folgen.
    pub fn unwatched(&self) -> u32 {
        let aired = match (&self.next_episode, self.episodes) {
            (Some(next), _) => next.episode.saturating_sub(1),
            (None, Some(total)) => total,
            (None, None) => self.progress,
        };
        aired.saturating_sub(self.progress)
    }
}

/// Bevorzugte Sprache für Titel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TitleLanguage {
    #[default]
    Romaji,
    English,
    Native,
}

pub fn parse_viewer(body: &str) -> Result<Viewer, AnimeError> {
    let data = data_of(body)?;
    let v = &data["Viewer"];
    Ok(Viewer {
        id: v["id"].as_u64().ok_or_else(|| missing("Viewer.id"))?,
        name: v["name"].as_str().unwrap_or_default().to_string(),
        avatar: v["avatar"]["medium"].as_str().map(str::to_string),
    })
}

/// Wertet die Watchlist aus; Einträge mit neuen Folgen stehen vorne.
pub fn parse_watching(body: &str, lang: TitleLanguage) -> Result<Vec<AnimeEntry>, AnimeError> {
    let data = data_of(body)?;
    let lists = data["MediaListCollection"]["lists"]
        .as_array()
        .ok_or_else(|| missing("MediaListCollection.lists"))?;
    let mut out: Vec<AnimeEntry> = lists
        .iter()
        .filter_map(|l| l["entries"].as_array())
        .flatten()
        .filter_map(|e| {
            let media = &e["media"];
            Some(AnimeEntry {
                media_id: media["id"].as_u64()?,
                title: pick_title(&media["title"], lang),
                progress: e["progress"].as_u64().unwrap_or(0) as u32,
                episodes: media["episodes"].as_u64().map(|n| n as u32),
                cover: media["coverImage"]["large"].as_str().map(str::to_string),
                url: media["siteUrl"].as_str().map(str::to_string),
                next_episode: media["nextAiringEpisode"]["episode"].as_u64().map(|ep| {
                    NextEpisode {
                        episode: ep as u32,
                        airing_at: media["nextAiringEpisode"]["airingAt"].as_i64().unwrap_or(0),
                    }
                }),
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.unwatched()
            .cmp(&a.unwatched())
            .then(a.title.cmp(&b.title))
    });
    Ok(out)
}

fn pick_title(title: &Value, lang: TitleLanguage) -> String {
    let order = match lang {
        TitleLanguage::Romaji => ["romaji", "english", "native"],
        TitleLanguage::English => ["english", "romaji", "native"],
        TitleLanguage::Native => ["native", "romaji", "english"],
    };
    order
        .iter()
        .find_map(|k| title[*k].as_str().filter(|s| !s.is_empty()))
        .unwrap_or("?")
        .to_string()
}

fn data_of(body: &str) -> Result<Value, AnimeError> {
    let mut v: Value = serde_json::from_str(body)?;
    if let Some(msg) = v["errors"][0]["message"].as_str() {
        return Err(AnimeError::Api(msg.to_string()));
    }
    match v.get_mut("data").map(Value::take) {
        Some(data) if !data.is_null() => Ok(data),
        _ => Err(missing("data")),
    }
}

fn missing(field: &str) -> AnimeError {
    AnimeError::Format(format!("Feld {field} fehlt"))
}
