use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdblockError {
    #[error("Filter list error: {0}")]
    FilterList(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Lock poisoned")]
    LockPoisoned,
}
