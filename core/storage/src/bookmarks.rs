use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::StorageError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bookmark {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub position: i32,
    pub tags: Vec<String>,
    pub favicon_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkNode {
    pub item: Bookmark,
    pub children: Vec<BookmarkNode>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateBookmarkParams {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub title: String,
    pub url: Option<String>,
    pub is_folder: bool,
    pub position: Option<i32>,
    pub tags: Vec<String>,
    pub favicon_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateBookmarkParams {
    pub title: Option<String>,
    pub url: Option<Option<String>>,
    pub parent_id: Option<String>,
    pub position: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub favicon_url: Option<Option<String>>,
}

pub struct BookmarkManager;

impl BookmarkManager {
    pub fn create(
        conn: &Connection,
        params: CreateBookmarkParams,
    ) -> Result<Bookmark, StorageError> {
        let id = params.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let parent_id = params.parent_id.or_else(|| Some("toolbar".to_string()));
        let now = chrono::Utc::now().timestamp_millis();

        // Calculate next position if not specified
        let position = match params.position {
            Some(pos) => pos,
            None => {
                let max_pos: Option<i32> = match &parent_id {
                    Some(pid) => conn
                        .query_row(
                            "SELECT MAX(position) FROM bookmarks WHERE parent_id = ?1",
                            params![pid],
                            |row| row.get(0),
                        )
                        .ok(),
                    None => conn
                        .query_row(
                            "SELECT MAX(position) FROM bookmarks WHERE parent_id IS NULL",
                            [],
                            |row| row.get(0),
                        )
                        .ok(),
                };
                max_pos.map_or(0, |p| p + 1)
            }
        };

        let tags_str = if params.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&params.tags)?)
        };

        conn.execute(
            r#"
            INSERT INTO bookmarks (
                id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                id,
                parent_id,
                params.title,
                params.url,
                params.is_folder as i32,
                position,
                tags_str,
                params.favicon_url,
                now,
                now,
            ],
        )?;

        Ok(Bookmark {
            id,
            parent_id,
            title: params.title,
            url: params.url,
            is_folder: params.is_folder,
            position,
            tags: params.tags,
            favicon_url: params.favicon_url,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get(conn: &Connection, id: &str) -> Result<Option<Bookmark>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
            FROM bookmarks
            WHERE id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_bookmark(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn update(
        conn: &Connection,
        id: &str,
        params: UpdateBookmarkParams,
    ) -> Result<Bookmark, StorageError> {
        let current = Self::get(conn, id)?
            .ok_or_else(|| StorageError::NotFound(format!("Bookmark {} not found", id)))?;

        let now = chrono::Utc::now().timestamp_millis();
        let new_title = params.title.unwrap_or(current.title);
        let new_url = match params.url {
            Some(u) => u,
            None => current.url,
        };
        let new_parent_id = params.parent_id.or(current.parent_id);
        let new_position = params.position.unwrap_or(current.position);
        let new_tags = params.tags.unwrap_or(current.tags);
        let new_favicon = match params.favicon_url {
            Some(f) => f,
            None => current.favicon_url,
        };

        let tags_str = if new_tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&new_tags)?)
        };

        conn.execute(
            r#"
            UPDATE bookmarks
            SET title = ?1,
                url = ?2,
                parent_id = ?3,
                position = ?4,
                tags = ?5,
                favicon_url = ?6,
                updated_at = ?7
            WHERE id = ?8
            "#,
            params![
                new_title,
                new_url,
                new_parent_id,
                new_position,
                tags_str,
                new_favicon,
                now,
                id,
            ],
        )?;

        Ok(Bookmark {
            id: id.to_string(),
            parent_id: new_parent_id,
            title: new_title,
            url: new_url,
            is_folder: current.is_folder,
            position: new_position,
            tags: new_tags,
            favicon_url: new_favicon,
            created_at: current.created_at,
            updated_at: now,
        })
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<bool, StorageError> {
        // Prevent deleting built-in roots
        if id == "root" || id == "toolbar" || id == "other" || id == "mobile" {
            return Err(StorageError::Validation(format!(
                "Cannot delete system bookmark folder '{}'",
                id
            )));
        }

        let affected = conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn list_by_parent(
        conn: &Connection,
        parent_id: Option<&str>,
    ) -> Result<Vec<Bookmark>, StorageError> {
        let mut results = Vec::new();
        match parent_id {
            Some(pid) => {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
                    FROM bookmarks
                    WHERE parent_id = ?1
                    ORDER BY position ASC, created_at ASC
                    "#,
                )?;
                let rows = stmt.query_map(params![pid], Self::row_to_bookmark)?;
                for row in rows {
                    results.push(row?);
                }
            }
            None => {
                let mut stmt = conn.prepare(
                    r#"
                    SELECT id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
                    FROM bookmarks
                    WHERE parent_id IS NULL
                    ORDER BY position ASC, created_at ASC
                    "#,
                )?;
                let rows = stmt.query_map([], Self::row_to_bookmark)?;
                for row in rows {
                    results.push(row?);
                }
            }
        }
        Ok(results)
    }

    pub fn list_all(conn: &Connection) -> Result<Vec<Bookmark>, StorageError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
            FROM bookmarks
            ORDER BY parent_id NULLS FIRST, position ASC
            "#,
        )?;
        let rows = stmt.query_map([], Self::row_to_bookmark)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn search(conn: &Connection, query: &str) -> Result<Vec<Bookmark>, StorageError> {
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            r#"
            SELECT id, parent_id, title, url, is_folder, position, tags, favicon_url, created_at, updated_at
            FROM bookmarks
            WHERE title LIKE ?1 OR (url IS NOT NULL AND url LIKE ?1) OR (tags IS NOT NULL AND tags LIKE ?1)
            ORDER BY title ASC
            "#,
        )?;

        let rows = stmt.query_map(params![pattern], Self::row_to_bookmark)?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_tree(
        conn: &Connection,
        root_id: Option<&str>,
    ) -> Result<Vec<BookmarkNode>, StorageError> {
        let all = Self::list_all(conn)?;
        let target_parent = root_id.map(String::from);
        Ok(Self::build_tree(&all, target_parent.as_deref()))
    }

    fn build_tree(items: &[Bookmark], parent_id: Option<&str>) -> Vec<BookmarkNode> {
        items
            .iter()
            .filter(|b| b.parent_id.as_deref() == parent_id)
            .map(|b| {
                let children = if b.is_folder {
                    Self::build_tree(items, Some(&b.id))
                } else {
                    Vec::new()
                };
                BookmarkNode {
                    item: b.clone(),
                    children,
                }
            })
            .collect()
    }

    pub fn reorder(
        conn: &Connection,
        parent_id: Option<&str>,
        ids: &[String],
    ) -> Result<(), StorageError> {
        for (pos, id) in ids.iter().enumerate() {
            match parent_id {
                Some(pid) => {
                    conn.execute(
                        "UPDATE bookmarks SET position = ?1, parent_id = ?2 WHERE id = ?3",
                        params![pos as i32, pid, id],
                    )?;
                }
                None => {
                    conn.execute(
                        "UPDATE bookmarks SET position = ?1, parent_id = NULL WHERE id = ?2",
                        params![pos as i32, id],
                    )?;
                }
            }
        }
        Ok(())
    }

    fn row_to_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
        let is_folder_num: i32 = row.get(4)?;
        let tags_str: Option<String> = row.get(6)?;
        let tags = tags_str
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .unwrap_or_default();

        Ok(Bookmark {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            title: row.get(2)?,
            url: row.get(3)?,
            is_folder: is_folder_num != 0,
            position: row.get(5)?,
            tags,
            favicon_url: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }
}
