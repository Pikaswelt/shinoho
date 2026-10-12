pub mod bookmarks;
pub mod bookmarks_html;
pub mod downloads;
pub mod error;
pub mod history;
pub mod migrations;
pub mod settings;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};

pub use bookmarks::*;
pub use bookmarks_html::*;
pub use downloads::*;
pub use error::StorageError;
pub use history::*;
pub use settings::*;

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    in_memory: bool,
    path: Option<PathBuf>,
}

impl Storage {
    /// Open or create an in-memory database (suitable for tests or incognito mode).
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        migrations::run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            in_memory: true,
            path: None,
        })
    }

    /// Open or create a persistent database at the given filesystem path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let p = path.as_ref().to_path_buf();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::Validation(format!(
                    "Failed to create storage directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let mut conn = Connection::open(&p)?;
        migrations::run_migrations(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            in_memory: false,
            path: Some(p),
        })
    }

    pub fn is_in_memory(&self) -> bool {
        self.in_memory
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn with_conn<F, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce(&Connection) -> Result<R, StorageError>,
    {
        let conn = self.conn.lock().map_err(|_| StorageError::LockPoisoned)?;
        f(&conn)
    }

    // --- History ---

    pub fn record_visit(
        &self,
        url: &str,
        title: &str,
        typed: bool,
        favicon_url: Option<&str>,
    ) -> Result<HistoryVisit, StorageError> {
        self.with_conn(|conn| HistoryManager::record_visit(conn, url, title, typed, favicon_url))
    }

    pub fn get_history(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryVisit>, StorageError> {
        self.with_conn(|conn| HistoryManager::list(conn, limit, offset))
    }

    pub fn search_history(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<HistoryVisit>, StorageError> {
        self.with_conn(|conn| HistoryManager::search(conn, query, limit))
    }

    pub fn get_top_sites(&self, limit: usize) -> Result<Vec<TopSite>, StorageError> {
        self.with_conn(|conn| HistoryManager::get_top_sites(conn, limit))
    }

    pub fn delete_history_visit(&self, id: i64) -> Result<bool, StorageError> {
        self.with_conn(|conn| HistoryManager::delete_visit(conn, id))
    }

    pub fn clear_history(&self) -> Result<usize, StorageError> {
        self.with_conn(HistoryManager::clear)
    }

    pub fn clear_history_range(&self, from_ms: i64, to_ms: i64) -> Result<usize, StorageError> {
        self.with_conn(|conn| HistoryManager::clear_range(conn, from_ms, to_ms))
    }

    // --- Bookmarks ---

    pub fn create_bookmark(&self, params: CreateBookmarkParams) -> Result<Bookmark, StorageError> {
        self.with_conn(|conn| BookmarkManager::create(conn, params))
    }

    pub fn get_bookmark(&self, id: &str) -> Result<Option<Bookmark>, StorageError> {
        self.with_conn(|conn| BookmarkManager::get(conn, id))
    }

    pub fn update_bookmark(
        &self,
        id: &str,
        params: UpdateBookmarkParams,
    ) -> Result<Bookmark, StorageError> {
        self.with_conn(|conn| BookmarkManager::update(conn, id, params))
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<bool, StorageError> {
        self.with_conn(|conn| BookmarkManager::delete(conn, id))
    }

    pub fn list_bookmarks(&self, parent_id: Option<&str>) -> Result<Vec<Bookmark>, StorageError> {
        self.with_conn(|conn| BookmarkManager::list_by_parent(conn, parent_id))
    }

    pub fn list_all_bookmarks(&self) -> Result<Vec<Bookmark>, StorageError> {
        self.with_conn(BookmarkManager::list_all)
    }

    pub fn get_bookmark_tree(
        &self,
        root_id: Option<&str>,
    ) -> Result<Vec<BookmarkNode>, StorageError> {
        self.with_conn(|conn| BookmarkManager::get_tree(conn, root_id))
    }

    pub fn search_bookmarks(&self, query: &str) -> Result<Vec<Bookmark>, StorageError> {
        self.with_conn(|conn| BookmarkManager::search(conn, query))
    }

    pub fn reorder_bookmarks(
        &self,
        parent_id: Option<&str>,
        ids: &[String],
    ) -> Result<(), StorageError> {
        self.with_conn(|conn| BookmarkManager::reorder(conn, parent_id, ids))
    }

    pub fn export_bookmarks_html(&self, root_id: Option<&str>) -> Result<String, StorageError> {
        let tree = self.get_bookmark_tree(root_id)?;
        Ok(BookmarkHtml::export_tree(&tree))
    }

    pub fn import_bookmarks_html(
        &self,
        html: &str,
        target_parent_id: Option<&str>,
    ) -> Result<ImportSummary, StorageError> {
        let items = BookmarkHtml::parse(html);
        BookmarkHtml::import_to_storage(self, &items, target_parent_id)
    }

    // --- Settings ---

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.with_conn(|conn| SettingsManager::get(conn, key))
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.with_conn(|conn| SettingsManager::set(conn, key, value))
    }

    pub fn get_setting_json<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        self.with_conn(|conn| SettingsManager::get_json(conn, key))
    }

    pub fn set_setting_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), StorageError> {
        self.with_conn(|conn| SettingsManager::set_json(conn, key, value))
    }

    pub fn get_all_settings(&self) -> Result<HashMap<String, String>, StorageError> {
        self.with_conn(SettingsManager::get_all)
    }

    pub fn delete_setting(&self, key: &str) -> Result<bool, StorageError> {
        self.with_conn(|conn| SettingsManager::delete(conn, key))
    }

    // --- Downloads ---

    pub fn create_download(&self, item: &DownloadItem) -> Result<(), StorageError> {
        self.with_conn(|conn| DownloadsManager::insert(conn, item))
    }

    pub fn update_download_progress(
        &self,
        id: &str,
        received_bytes: i64,
        total_bytes: Option<i64>,
        state: DownloadState,
        error_message: Option<String>,
    ) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            DownloadsManager::update_progress(
                conn,
                id,
                received_bytes,
                total_bytes,
                state,
                error_message,
            )
        })
    }

    pub fn get_download(&self, id: &str) -> Result<Option<DownloadItem>, StorageError> {
        self.with_conn(|conn| DownloadsManager::get(conn, id))
    }

    pub fn list_downloads(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DownloadItem>, StorageError> {
        self.with_conn(|conn| DownloadsManager::list(conn, limit, offset))
    }

    pub fn delete_download(&self, id: &str) -> Result<bool, StorageError> {
        self.with_conn(|conn| DownloadsManager::delete(conn, id))
    }

    pub fn clear_downloads(&self) -> Result<usize, StorageError> {
        self.with_conn(DownloadsManager::clear)
    }
}
