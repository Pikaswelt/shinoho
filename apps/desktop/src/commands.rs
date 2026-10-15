//! Tauri-Commands fuer History, Lesezeichen, Einstellungen, Inkognito, Adblock und Downloads.
//!
//! Alle Commands geben `Result<_, String>` zurueck (wie in core/api) und lesen den
//! Zustand aus [`AppServices`]. Eingabe-Typen mit verschachtelten Optionen werden ueber
//! eigene Request-DTOs abgebildet, weil sie nicht direkt aus JSON deserialisierbar sind.

use std::collections::HashMap;

use serde::Deserialize;
use shinoho_adblock::{AdblockDecision, AdblockStats};
use shinoho_downloads::{DownloadItem, DownloadOptions};
use shinoho_hub::HubState;
use shinoho_incognito::IncognitoConfig;
use shinoho_storage::{
    Bookmark, BookmarkNode, CreateBookmarkParams, HistoryVisit, ImportSummary, TopSite,
    UpdateBookmarkParams,
};
use tauri::State;

use crate::services::AppServices;

const DEFAULT_PAGE_SIZE: usize = 50;

fn to_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// --- Request-DTOs ---

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecordVisitRequest {
    pub url: String,
    pub title: String,
    pub typed: bool,
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CreateBookmarkRequest {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub position: Option<i32>,
    pub tags: Vec<String>,
    pub favicon_url: Option<String>,
}

impl From<CreateBookmarkRequest> for CreateBookmarkParams {
    fn from(r: CreateBookmarkRequest) -> Self {
        CreateBookmarkParams {
            id: r.id,
            parent_id: r.parent_id,
            title: r.title,
            url: r.url,
            is_folder: r.is_folder,
            position: r.position,
            tags: r.tags,
            favicon_url: r.favicon_url,
        }
    }
}

/// Partielles Update. Ein leerer String bei `url` oder `faviconUrl` loescht das Feld.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateBookmarkRequest {
    pub title: Option<String>,
    pub url: Option<String>,
    pub parent_id: Option<String>,
    pub position: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub favicon_url: Option<String>,
}

fn clearable(value: Option<String>) -> Option<Option<String>> {
    value.map(|v| if v.is_empty() { None } else { Some(v) })
}

impl From<UpdateBookmarkRequest> for UpdateBookmarkParams {
    fn from(r: UpdateBookmarkRequest) -> Self {
        UpdateBookmarkParams {
            title: r.title,
            url: clearable(r.url),
            parent_id: r.parent_id,
            position: r.position,
            tags: r.tags,
            favicon_url: clearable(r.favicon_url),
        }
    }
}

// --- History ---

#[tauri::command]
pub fn record_visit(
    state: State<'_, AppServices>,
    request: RecordVisitRequest,
) -> Result<HistoryVisit, String> {
    state
        .storage
        .record_visit(
            &request.url,
            &request.title,
            request.typed,
            request.favicon_url.as_deref(),
        )
        .map_err(to_err)
}

#[tauri::command]
pub fn get_history(
    state: State<'_, AppServices>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<HistoryVisit>, String> {
    state
        .storage
        .get_history(limit.unwrap_or(DEFAULT_PAGE_SIZE), offset.unwrap_or(0))
        .map_err(to_err)
}

#[tauri::command]
pub fn search_history(
    state: State<'_, AppServices>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<HistoryVisit>, String> {
    state
        .storage
        .search_history(&query, limit.unwrap_or(DEFAULT_PAGE_SIZE))
        .map_err(to_err)
}

#[tauri::command]
pub fn get_top_sites(
    state: State<'_, AppServices>,
    limit: Option<usize>,
) -> Result<Vec<TopSite>, String> {
    state
        .storage
        .get_top_sites(limit.unwrap_or(DEFAULT_PAGE_SIZE))
        .map_err(to_err)
}

#[tauri::command]
pub fn delete_history_visit(state: State<'_, AppServices>, id: i64) -> Result<bool, String> {
    state.storage.delete_history_visit(id).map_err(to_err)
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppServices>) -> Result<usize, String> {
    state.storage.clear_history().map_err(to_err)
}

#[tauri::command]
pub fn clear_history_range(
    state: State<'_, AppServices>,
    from_ms: i64,
    to_ms: i64,
) -> Result<usize, String> {
    state
        .storage
        .clear_history_range(from_ms, to_ms)
        .map_err(to_err)
}

// --- Lesezeichen ---

#[tauri::command]
pub fn create_bookmark(
    state: State<'_, AppServices>,
    request: CreateBookmarkRequest,
) -> Result<Bookmark, String> {
    state
        .storage
        .create_bookmark(request.into())
        .map_err(to_err)
}

#[tauri::command]
pub fn get_bookmark(state: State<'_, AppServices>, id: String) -> Result<Option<Bookmark>, String> {
    state.storage.get_bookmark(&id).map_err(to_err)
}

#[tauri::command]
pub fn update_bookmark(
    state: State<'_, AppServices>,
    id: String,
    request: UpdateBookmarkRequest,
) -> Result<Bookmark, String> {
    state
        .storage
        .update_bookmark(&id, request.into())
        .map_err(to_err)
}

#[tauri::command]
pub fn delete_bookmark(state: State<'_, AppServices>, id: String) -> Result<bool, String> {
    state.storage.delete_bookmark(&id).map_err(to_err)
}

#[tauri::command]
pub fn list_bookmarks(
    state: State<'_, AppServices>,
    parent_id: Option<String>,
) -> Result<Vec<Bookmark>, String> {
    state
        .storage
        .list_bookmarks(parent_id.as_deref())
        .map_err(to_err)
}

#[tauri::command]
pub fn get_bookmark_tree(
    state: State<'_, AppServices>,
    root_id: Option<String>,
) -> Result<Vec<BookmarkNode>, String> {
    state
        .storage
        .get_bookmark_tree(root_id.as_deref())
        .map_err(to_err)
}

#[tauri::command]
pub fn search_bookmarks(
    state: State<'_, AppServices>,
    query: String,
) -> Result<Vec<Bookmark>, String> {
    state.storage.search_bookmarks(&query).map_err(to_err)
}

#[tauri::command]
pub fn export_bookmarks_html(
    state: State<'_, AppServices>,
    root_id: Option<String>,
) -> Result<String, String> {
    state
        .storage
        .export_bookmarks_html(root_id.as_deref())
        .map_err(to_err)
}

#[tauri::command]
pub fn import_bookmarks_html(
    state: State<'_, AppServices>,
    html: String,
    target_parent_id: Option<String>,
) -> Result<ImportSummary, String> {
    state
        .storage
        .import_bookmarks_html(&html, target_parent_id.as_deref())
        .map_err(to_err)
}

// --- Einstellungen ---

#[tauri::command]
pub fn get_setting(state: State<'_, AppServices>, key: String) -> Result<Option<String>, String> {
    state.storage.get_setting(&key).map_err(to_err)
}

#[tauri::command]
pub fn set_setting(
    state: State<'_, AppServices>,
    key: String,
    value: String,
) -> Result<(), String> {
    state.storage.set_setting(&key, &value).map_err(to_err)
}

#[tauri::command]
pub fn get_all_settings(state: State<'_, AppServices>) -> Result<HashMap<String, String>, String> {
    state.storage.get_all_settings().map_err(to_err)
}

#[tauri::command]
pub fn delete_setting(state: State<'_, AppServices>, key: String) -> Result<bool, String> {
    state.storage.delete_setting(&key).map_err(to_err)
}

// --- Hub ---

#[tauri::command]
pub fn get_hub_state(state: State<'_, AppServices>) -> Result<Option<HubState>, String> {
    shinoho_hub::load(&state.storage).map_err(to_err)
}

#[tauri::command]
pub fn save_hub_state(state: State<'_, AppServices>, hub_state: HubState) -> Result<(), String> {
    shinoho_hub::save(&state.storage, &hub_state).map_err(to_err)
}

// --- Inkognito ---

#[tauri::command]
pub fn create_incognito_session(
    state: State<'_, AppServices>,
    custom_id: Option<String>,
    config: Option<IncognitoConfig>,
) -> Result<String, String> {
    state
        .incognito
        .create_session(custom_id.as_deref(), config)
        .map_err(to_err)
}

#[tauri::command]
pub fn close_incognito_session(
    state: State<'_, AppServices>,
    session_id: String,
) -> Result<(), String> {
    state.incognito.close_session(&session_id).map_err(to_err)
}

#[tauri::command]
pub fn close_all_incognito_sessions(state: State<'_, AppServices>) -> Result<(), String> {
    state.incognito.close_all_sessions().map_err(to_err)
}

#[tauri::command]
pub fn list_incognito_sessions(state: State<'_, AppServices>) -> Result<Vec<String>, String> {
    state.incognito.active_session_ids().map_err(to_err)
}

#[tauri::command]
pub fn register_incognito_tab(
    state: State<'_, AppServices>,
    session_id: String,
    tab_id: String,
) -> Result<(), String> {
    state
        .incognito
        .register_tab(&session_id, &tab_id)
        .map_err(to_err)
}

/// Gibt `true` zurueck, wenn die Sitzung durch das Entfernen des Tabs geschlossen wurde.
#[tauri::command]
pub fn unregister_incognito_tab(
    state: State<'_, AppServices>,
    session_id: String,
    tab_id: String,
    auto_close_if_empty: bool,
) -> Result<bool, String> {
    state
        .incognito
        .unregister_tab(&session_id, &tab_id, auto_close_if_empty)
        .map_err(to_err)
}

// --- Adblock ---

#[tauri::command]
pub fn get_adblock_stats(state: State<'_, AppServices>) -> AdblockStats {
    state.adblock.stats()
}

#[tauri::command]
pub fn set_adblock_enabled(state: State<'_, AppServices>, enabled: bool) {
    state.adblock.set_enabled(enabled);
}

#[tauri::command]
pub fn whitelist_domain(state: State<'_, AppServices>, domain: String, whitelisted: bool) {
    if whitelisted {
        state.adblock.whitelist_domain(&domain);
    } else {
        state.adblock.unwhitelist_domain(&domain);
    }
}

#[tauri::command]
pub fn check_network_request(
    state: State<'_, AppServices>,
    url: String,
    source_url: String,
    request_type: String,
) -> AdblockDecision {
    state
        .adblock
        .check_network_request(&url, &source_url, &request_type)
}

#[tauri::command]
pub fn get_hidden_elements_css(state: State<'_, AppServices>, url: String) -> Option<String> {
    state.adblock.hidden_elements_css(&url)
}

#[tauri::command]
pub fn add_adblock_filter_list(
    state: State<'_, AppServices>,
    content: String,
) -> Result<usize, String> {
    state.adblock.add_filter_list(&content).map_err(to_err)
}

// --- Downloads ---

#[tauri::command]
pub async fn start_download(
    state: State<'_, AppServices>,
    url: String,
    options: Option<DownloadOptions>,
) -> Result<DownloadItem, String> {
    state
        .downloads
        .start_download(&url, options)
        .await
        .map_err(to_err)
}

#[tauri::command]
pub async fn pause_download(state: State<'_, AppServices>, id: String) -> Result<(), String> {
    state.downloads.pause_download(&id).await.map_err(to_err)
}

#[tauri::command]
pub async fn resume_download(state: State<'_, AppServices>, id: String) -> Result<(), String> {
    state.downloads.resume_download(&id).await.map_err(to_err)
}

#[tauri::command]
pub async fn cancel_download(state: State<'_, AppServices>, id: String) -> Result<(), String> {
    state.downloads.cancel_download(&id).await.map_err(to_err)
}

#[tauri::command]
pub async fn retry_download(
    state: State<'_, AppServices>,
    id: String,
) -> Result<DownloadItem, String> {
    state.downloads.retry_download(&id).await.map_err(to_err)
}

#[tauri::command]
pub async fn get_download(
    state: State<'_, AppServices>,
    id: String,
) -> Result<Option<DownloadItem>, String> {
    state.downloads.get_download(&id).await.map_err(to_err)
}

#[tauri::command]
pub async fn list_downloads(
    state: State<'_, AppServices>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<DownloadItem>, String> {
    state
        .downloads
        .list_downloads(limit.unwrap_or(DEFAULT_PAGE_SIZE), offset.unwrap_or(0))
        .await
        .map_err(to_err)
}

#[tauri::command]
pub async fn delete_download(
    state: State<'_, AppServices>,
    id: String,
    delete_file: bool,
) -> Result<bool, String> {
    state
        .downloads
        .delete_download(&id, delete_file)
        .await
        .map_err(to_err)
}

#[tauri::command]
pub async fn clear_completed_downloads(state: State<'_, AppServices>) -> Result<usize, String> {
    state.downloads.clear_completed().await.map_err(to_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bookmark_url_clears_field() {
        let req = UpdateBookmarkRequest {
            url: Some(String::new()),
            ..Default::default()
        };
        let params: UpdateBookmarkParams = req.into();
        assert_eq!(params.url, Some(None));
    }

    #[test]
    fn non_empty_bookmark_url_is_set() {
        let req = UpdateBookmarkRequest {
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        let params: UpdateBookmarkParams = req.into();
        assert_eq!(params.url, Some(Some("https://example.com".into())));
    }

    #[test]
    fn missing_bookmark_url_is_untouched() {
        let params: UpdateBookmarkParams = UpdateBookmarkRequest::default().into();
        assert_eq!(params.url, None);
    }

    #[test]
    fn create_request_deserializes_camel_case() {
        let json = r#"{"title":"Doc","url":"https://a.test","isFolder":false,"tags":["x"]}"#;
        let req: CreateBookmarkRequest = serde_json::from_str(json).expect("json");
        let params: CreateBookmarkParams = req.into();
        assert_eq!(params.title, "Doc");
        assert_eq!(params.url.as_deref(), Some("https://a.test"));
        assert_eq!(params.tags, vec!["x".to_string()]);
    }
}
