use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::StorageError;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    #[default]
    Pending,
    Downloading,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

impl std::str::FromStr for DownloadState {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let state = match s {
            "downloading" => Self::Downloading,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            "failed" => Self::Failed,
            _ => Self::Pending,
        };
        Ok(state)
    }
}

impl DownloadState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub target_path: String,
    pub filename: String,
    pub total_bytes: Option<i64>,
    pub received_bytes: i64,
    pub state: DownloadState,
    pub error_message: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

pub struct DownloadsManager;

impl DownloadsManager {
    pub fn insert(conn: &Connection, item: &DownloadItem) -> Result<(), StorageError> {
        conn.execute(
            r#"
            INSERT INTO downloads (
                id, url, target_path, filename, total_bytes, received_bytes, state, error_message, started_at, completed_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                item.id,
                item.url,
                item.target_path,
                item.filename,
                item.total_bytes,
                item.received_bytes,
                item.state.as_str(),
                item.error_message,
                item.started_at,
                item.completed_at,
            ],
        )?;
        Ok(())
    }

    pub fn update_progress(
        conn: &Connection,
        id: &str,
        received_bytes: i64,
        total_bytes: Option<i64>,
        state: DownloadState,
        error_message: Option<String>,
    ) -> Result<(), StorageError> {
        let completed_at = if state == DownloadState::Completed {
            Some(chrono::Utc::now().timestamp_millis())
        } else {
            None
        };

        conn.execute(
            r#"
            UPDATE downloads
            SET received_bytes = ?1,
                total_bytes = COALESCE(?2, total_bytes),
                state = ?3,
                error_message = ?4,
                completed_at = COALESCE(?5, completed_at)
            WHERE id = ?6
            "#,
            params![
                received_bytes,
                total_bytes,
                state.as_str(),
                error_message,
                completed_at,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn get(conn: &Connection, id: &str) -> Result<Option<DownloadItem>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, target_path, filename, total_bytes, received_bytes, state, error_message, started_at, completed_at
            FROM downloads
            WHERE id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let state_str: String = row.get(6)?;
            Ok(Some(DownloadItem {
                id: row.get(0)?,
                url: row.get(1)?,
                target_path: row.get(2)?,
                filename: row.get(3)?,
                total_bytes: row.get(4)?,
                received_bytes: row.get(5)?,
                state: state_str.parse().unwrap_or_default(),
                error_message: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list(
        conn: &Connection,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DownloadItem>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, target_path, filename, total_bytes, received_bytes, state, error_message, started_at, completed_at
            FROM downloads
            ORDER BY started_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let state_str: String = row.get(6)?;
            Ok(DownloadItem {
                id: row.get(0)?,
                url: row.get(1)?,
                target_path: row.get(2)?,
                filename: row.get(3)?,
                total_bytes: row.get(4)?,
                received_bytes: row.get(5)?,
                state: state_str.parse().unwrap_or_default(),
                error_message: row.get(7)?,
                started_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<bool, StorageError> {
        let affected = conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn clear(conn: &Connection) -> Result<usize, StorageError> {
        let affected = conn.execute("DELETE FROM downloads", [])?;
        Ok(affected)
    }
}
