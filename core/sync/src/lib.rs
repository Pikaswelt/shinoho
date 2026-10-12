//! Sync ohne Server: Lesezeichen und Einstellungen werden in eine verschlüsselte
//! Datei exportiert und auf einem anderen Gerät wieder importiert.
//!
//! Dateiformat (alle Felder hintereinander):
//! `MAGIC (4) | VERSION (1) | SALT (16) | NONCE (12) | CIPHERTEXT`
//!
//! Der Schlüssel wird mit Argon2id aus der Passphrase abgeleitet, verschlüsselt wird mit
//! ChaCha20-Poly1305. Eine falsche Passphrase oder eine veränderte Datei fällt beim
//! Entschlüsseln auf.

use std::collections::{HashMap, HashSet};

use argon2::Argon2;
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use serde::{Deserialize, Serialize};
use shinoho_storage::{
    Bookmark, CreateBookmarkParams, Storage, StorageError, UpdateBookmarkParams,
};
use thiserror::Error;

/// Dateikennung von Shinoho-Sync-Dateien.
pub const MAGIC: &[u8; 4] = b"SHSY";
/// Aktuelle Version des Dateiformats.
pub const FORMAT_VERSION: u8 = 1;
/// Empfohlene Dateiendung.
pub const FILE_EXTENSION: &str = "shinoho-sync";

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = MAGIC.len() + 1 + SALT_LEN + NONCE_LEN;
const MIN_PASSPHRASE_LEN: usize = 8;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Die Passphrase muss mindestens {MIN_PASSPHRASE_LEN} Zeichen lang sein")]
    PassphraseTooShort,
    #[error("Keine Shinoho-Sync-Datei")]
    NotASyncFile,
    #[error("Dateiversion {0} wird nicht unterstützt")]
    UnsupportedVersion(u8),
    #[error("Falsche Passphrase oder beschädigte Datei")]
    Decrypt,
    #[error("Verschlüsselung fehlgeschlagen")]
    Encrypt,
    #[error("Schlüsselableitung fehlgeschlagen: {0}")]
    KeyDerivation(String),
    #[error("Ungültiger Inhalt: {0}")]
    Format(#[from] serde_json::Error),
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Inhalt einer Sync-Datei (vor der Verschlüsselung).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncBundle {
    pub version: u8,
    pub created_at: i64,
    pub bookmarks: Vec<Bookmark>,
    pub settings: HashMap<String, String>,
}

/// Ergebnis eines Imports.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub bookmarks_created: usize,
    pub bookmarks_updated: usize,
    /// Lokal neuer oder gleich alt – unverändert gelassen.
    pub bookmarks_skipped: usize,
    /// Übergeordneter Ordner fehlt in Datei und Datenbank.
    pub bookmarks_orphaned: usize,
    pub settings_written: usize,
}

/// Liest alle Lesezeichen und Einstellungen aus `storage`.
pub fn collect(storage: &Storage) -> Result<SyncBundle, SyncError> {
    Ok(SyncBundle {
        version: FORMAT_VERSION,
        created_at: chrono::Utc::now().timestamp_millis(),
        bookmarks: storage.list_all_bookmarks()?,
        settings: storage.get_all_settings()?,
    })
}

/// Exportiert Lesezeichen und Einstellungen verschlüsselt.
pub fn export_encrypted(storage: &Storage, passphrase: &str) -> Result<Vec<u8>, SyncError> {
    encrypt(&collect(storage)?, passphrase)
}

/// Entschlüsselt `data` und führt den Inhalt in `storage` zusammen.
pub fn import_encrypted(
    storage: &Storage,
    data: &[u8],
    passphrase: &str,
) -> Result<ImportReport, SyncError> {
    apply(storage, &decrypt(data, passphrase)?)
}

/// Verschlüsselt ein Bundle mit der Passphrase.
pub fn encrypt(bundle: &SyncBundle, passphrase: &str) -> Result<Vec<u8>, SyncError> {
    if passphrase.chars().count() < MIN_PASSPHRASE_LEN {
        return Err(SyncError::PassphraseTooShort);
    }
    let mut salt = [0u8; SALT_LEN];
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);

    let cipher = cipher_for(passphrase, &salt)?;
    let plaintext = serde_json::to_vec(bundle)?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_ref())
        .map_err(|_| SyncError::Encrypt)?;

    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Entschlüsselt eine Sync-Datei.
pub fn decrypt(data: &[u8], passphrase: &str) -> Result<SyncBundle, SyncError> {
    if data.len() < HEADER_LEN || &data[..MAGIC.len()] != MAGIC {
        return Err(SyncError::NotASyncFile);
    }
    let version = data[MAGIC.len()];
    if version != FORMAT_VERSION {
        return Err(SyncError::UnsupportedVersion(version));
    }
    let salt_start = MAGIC.len() + 1;
    let nonce_start = salt_start + SALT_LEN;
    let salt = &data[salt_start..nonce_start];
    let nonce = &data[nonce_start..HEADER_LEN];

    let cipher = cipher_for(passphrase, salt)?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), &data[HEADER_LEN..])
        .map_err(|_| SyncError::Decrypt)?;
    Ok(serde_json::from_slice(&plaintext)?)
}

/// Führt ein Bundle in `storage` zusammen: Lesezeichen gleicher ID werden nur
/// überschrieben, wenn die Datei neuer ist; Einstellungen aus der Datei gewinnen.
pub fn apply(storage: &Storage, bundle: &SyncBundle) -> Result<ImportReport, SyncError> {
    let mut report = ImportReport::default();

    let local: HashMap<String, Bookmark> = storage
        .list_all_bookmarks()?
        .into_iter()
        .map(|b| (b.id.clone(), b))
        .collect();
    let mut known: HashSet<String> = local.keys().cloned().collect();

    // Eltern vor Kindern anlegen: so lange Runden drehen, bis nichts mehr geht.
    let mut pending: Vec<&Bookmark> = bundle.bookmarks.iter().collect();
    loop {
        let before = pending.len();
        let mut rest = Vec::new();
        for item in pending {
            let parent_ready = item.parent_id.as_ref().is_none_or(|p| known.contains(p));
            if !parent_ready {
                rest.push(item);
                continue;
            }
            match local.get(&item.id) {
                Some(existing) if existing.updated_at >= item.updated_at => {
                    report.bookmarks_skipped += 1;
                }
                Some(_) => {
                    storage.update_bookmark(
                        &item.id,
                        UpdateBookmarkParams {
                            title: Some(item.title.clone()),
                            url: Some(item.url.clone()),
                            parent_id: item.parent_id.clone(),
                            position: Some(item.position),
                            tags: Some(item.tags.clone()),
                            favicon_url: Some(item.favicon_url.clone()),
                        },
                    )?;
                    report.bookmarks_updated += 1;
                }
                None => {
                    storage.create_bookmark(CreateBookmarkParams {
                        id: Some(item.id.clone()),
                        parent_id: item.parent_id.clone(),
                        title: item.title.clone(),
                        url: item.url.clone(),
                        is_folder: item.is_folder,
                        position: Some(item.position),
                        tags: item.tags.clone(),
                        favicon_url: item.favicon_url.clone(),
                    })?;
                    report.bookmarks_created += 1;
                }
            }
            known.insert(item.id.clone());
        }
        if rest.is_empty() || rest.len() == before {
            report.bookmarks_orphaned = rest.len();
            break;
        }
        pending = rest;
    }

    for (key, value) in &bundle.settings {
        storage.set_setting(key, value)?;
        report.settings_written += 1;
    }
    Ok(report)
}

fn cipher_for(passphrase: &str, salt: &[u8]) -> Result<ChaCha20Poly1305, SyncError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| SyncError::KeyDerivation(e.to_string()))?;
    Ok(ChaCha20Poly1305::new(Key::from_slice(&key)))
}
