use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Storage error: {0}")]
    Storage(#[from] shinoho_storage::StorageError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Download '{0}' not found")]
    NotFound(String),

    #[error("Download '{id}' is in invalid state: {state}")]
    InvalidState { id: String, state: String },

    #[error("Download '{0}' was cancelled")]
    Cancelled(String),

    #[error("Invalid URL: {0}")]
    Url(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Other download error: {0}")]
    Other(String),
}
