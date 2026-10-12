use thiserror::Error;

#[derive(Error, Debug)]
pub enum IncognitoError {
    #[error("Storage error: {0}")]
    Storage(#[from] shinoho_storage::StorageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session already exists: {0}")]
    SessionAlreadyExists(String),

    #[error("Lock poisoned")]
    LockPoisoned,
}
