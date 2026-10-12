use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::StorageError;

pub struct SettingsManager;

impl SettingsManager {
    pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, StorageError> {
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), StorageError> {
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            r#"
            INSERT INTO settings (key, value, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn get_json<T: DeserializeOwned>(
        conn: &Connection,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        match Self::get(conn, key)? {
            Some(val) => Ok(Some(serde_json::from_str(&val)?)),
            None => Ok(None),
        }
    }

    pub fn set_json<T: Serialize>(
        conn: &Connection,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError> {
        let serialized = serde_json::to_string(value)?;
        Self::set(conn, key, &serialized)
    }

    pub fn get_all(conn: &Connection) -> Result<HashMap<String, String>, StorageError> {
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            let k: String = row.get(0)?;
            let v: String = row.get(1)?;
            Ok((k, v))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn delete(conn: &Connection, key: &str) -> Result<bool, StorageError> {
        let affected = conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(affected > 0)
    }
}
