use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::StorageError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryVisit {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub visited_at: i64,
    pub visit_count: i64,
    pub typed: bool,
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopSite {
    pub url: String,
    pub title: String,
    pub visit_count: i64,
    pub last_visited: i64,
}

pub struct HistoryManager;

impl HistoryManager {
    pub fn record_visit(
        conn: &Connection,
        url: &str,
        title: &str,
        typed: bool,
        favicon_url: Option<&str>,
    ) -> Result<HistoryVisit, StorageError> {
        let now = chrono::Utc::now().timestamp_millis();

        // Check if an entry for this exact URL already exists
        let existing: Option<(i64, i64, String, Option<String>)> = conn
            .query_row(
                "SELECT id, visit_count, title, favicon_url FROM history_visits WHERE url = ?1",
                params![url],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .ok();

        if let Some((id, count, existing_title, existing_favicon)) = existing {
            let new_title = if title.trim().is_empty() {
                existing_title
            } else {
                title.to_string()
            };
            let new_favicon = favicon_url.map(|s| s.to_string()).or(existing_favicon);

            conn.execute(
                r#"
                UPDATE history_visits
                SET title = ?1,
                    visited_at = ?2,
                    visit_count = ?3,
                    typed = CASE WHEN ?4 = 1 THEN 1 ELSE typed END,
                    favicon_url = ?5
                WHERE id = ?6
                "#,
                params![new_title, now, count + 1, typed as i32, new_favicon, id],
            )?;

            Ok(HistoryVisit {
                id,
                url: url.to_string(),
                title: new_title,
                visited_at: now,
                visit_count: count + 1,
                typed,
                favicon_url: new_favicon,
            })
        } else {
            let final_title = if title.trim().is_empty() {
                url.to_string()
            } else {
                title.to_string()
            };

            conn.execute(
                r#"
                INSERT INTO history_visits (url, title, visited_at, visit_count, typed, favicon_url)
                VALUES (?1, ?2, ?3, 1, ?4, ?5)
                "#,
                params![url, final_title, now, typed as i32, favicon_url],
            )?;

            let id = conn.last_insert_rowid();

            Ok(HistoryVisit {
                id,
                url: url.to_string(),
                title: final_title,
                visited_at: now,
                visit_count: 1,
                typed,
                favicon_url: favicon_url.map(String::from),
            })
        }
    }

    pub fn list(
        conn: &Connection,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryVisit>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, title, visited_at, visit_count, typed, favicon_url
            FROM history_visits
            ORDER BY visited_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            let typed_num: i32 = row.get(5)?;
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
                visit_count: row.get(4)?,
                typed: typed_num != 0,
                favicon_url: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn search(
        conn: &Connection,
        query: &str,
        limit: usize,
    ) -> Result<Vec<HistoryVisit>, StorageError> {
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, title, visited_at, visit_count, typed, favicon_url
            FROM history_visits
            WHERE url LIKE ?1 OR title LIKE ?1
            ORDER BY visited_at DESC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            let typed_num: i32 = row.get(5)?;
            Ok(HistoryVisit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: row.get(3)?,
                visit_count: row.get(4)?,
                typed: typed_num != 0,
                favicon_url: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_top_sites(conn: &Connection, limit: usize) -> Result<Vec<TopSite>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT url, title, visit_count, visited_at
            FROM history_visits
            ORDER BY visit_count DESC, visited_at DESC
            LIMIT ?1
            "#,
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(TopSite {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_count: row.get(2)?,
                last_visited: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn delete_visit(conn: &Connection, id: i64) -> Result<bool, StorageError> {
        let affected = conn.execute("DELETE FROM history_visits WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn clear(conn: &Connection) -> Result<usize, StorageError> {
        let affected = conn.execute("DELETE FROM history_visits", [])?;
        Ok(affected)
    }

    pub fn clear_range(conn: &Connection, from_ms: i64, to_ms: i64) -> Result<usize, StorageError> {
        let affected = conn.execute(
            "DELETE FROM history_visits WHERE visited_at >= ?1 AND visited_at <= ?2",
            params![from_ms, to_ms],
        )?;
        Ok(affected)
    }
}
