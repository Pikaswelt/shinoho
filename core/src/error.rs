//! Fehlerbehandlung im Shinoho-Kern.

use thiserror::Error;

/// Zentraler Fehlertyp fuer den Shinoho-Kern.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Tab nicht gefunden: {0}")]
    TabNotFound(String),

    #[error("Ungueltige URL: {0}")]
    InvalidUrl(String),

    #[error("Ungueltige Operation: {0}")]
    InvalidOperation(String),

    #[error("Interner Fehler: {0}")]
    Internal(String),
}

/// Result-Alias fuer den Shinoho-Kern.
pub type CoreResult<T> = Result<T, CoreError>;
