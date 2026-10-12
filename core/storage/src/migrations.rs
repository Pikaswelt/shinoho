use rusqlite::{Connection, Transaction};

use crate::error::StorageError;

pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "Initial schema for history, bookmarks, settings, and downloads",
    sql: r#"
        -- History visits
        CREATE TABLE IF NOT EXISTS history_visits (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            title TEXT NOT NULL,
            visited_at INTEGER NOT NULL,
            visit_count INTEGER NOT NULL DEFAULT 1,
            typed INTEGER NOT NULL DEFAULT 0,
            favicon_url TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_history_url ON history_visits(url);
        CREATE INDEX IF NOT EXISTS idx_history_visited_at ON history_visits(visited_at DESC);

        -- Bookmarks
        CREATE TABLE IF NOT EXISTS bookmarks (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            title TEXT NOT NULL,
            url TEXT,
            is_folder INTEGER NOT NULL DEFAULT 0,
            position INTEGER NOT NULL DEFAULT 0,
            tags TEXT,
            favicon_url TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (parent_id) REFERENCES bookmarks(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_bookmarks_parent_position ON bookmarks(parent_id, position);

        -- Standard bookmark roots
        INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position, created_at, updated_at)
        VALUES ('root', NULL, 'Root', NULL, 1, 0, 0, 0);

        INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position, created_at, updated_at)
        VALUES ('toolbar', 'root', 'Bookmarks Bar', NULL, 1, 0, 0, 0);

        INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position, created_at, updated_at)
        VALUES ('other', 'root', 'Other Bookmarks', NULL, 1, 1, 0, 0);

        INSERT OR IGNORE INTO bookmarks (id, parent_id, title, url, is_folder, position, created_at, updated_at)
        VALUES ('mobile', 'root', 'Mobile Bookmarks', NULL, 1, 2, 0, 0);

        -- Settings
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- Downloads
        CREATE TABLE IF NOT EXISTS downloads (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            target_path TEXT NOT NULL,
            filename TEXT NOT NULL,
            total_bytes INTEGER,
            received_bytes INTEGER NOT NULL DEFAULT 0,
            state TEXT NOT NULL,
            error_message TEXT,
            started_at INTEGER NOT NULL,
            completed_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_downloads_started_at ON downloads(started_at DESC);
        "#,
}];

pub fn run_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    // Ensure foreign keys are enabled
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL,
            description TEXT NOT NULL
        );
        "#,
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current_version {
            let tx = conn.transaction()?;
            apply_migration(&tx, migration)?;
            tx.commit()?;
        }
    }

    Ok(())
}

fn apply_migration(tx: &Transaction, migration: &Migration) -> Result<(), StorageError> {
    tx.execute_batch(migration.sql)?;
    let now = chrono::Utc::now().timestamp_millis();
    tx.execute(
        "INSERT INTO _migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
        rusqlite::params![migration.version, now, migration.description],
    )?;
    Ok(())
}
