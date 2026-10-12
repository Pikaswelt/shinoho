use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
pub use shinoho_storage::DownloadState;

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
    pub speed_bps: u64,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

impl DownloadItem {
    pub fn progress_percent(&self) -> Option<f32> {
        self.total_bytes.and_then(|total| {
            if total > 0 {
                Some(((self.received_bytes as f64 / total as f64) * 100.0).clamp(0.0, 100.0) as f32)
            } else {
                None
            }
        })
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Pending | DownloadState::Downloading
        )
    }

    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            DownloadState::Completed | DownloadState::Cancelled | DownloadState::Failed
        )
    }
}

impl From<shinoho_storage::DownloadItem> for DownloadItem {
    fn from(item: shinoho_storage::DownloadItem) -> Self {
        Self {
            id: item.id,
            url: item.url,
            target_path: item.target_path,
            filename: item.filename,
            total_bytes: item.total_bytes,
            received_bytes: item.received_bytes,
            state: item.state,
            error_message: item.error_message,
            speed_bps: 0,
            started_at: item.started_at,
            completed_at: item.completed_at,
        }
    }
}

impl From<DownloadItem> for shinoho_storage::DownloadItem {
    fn from(item: DownloadItem) -> Self {
        Self {
            id: item.id,
            url: item.url,
            target_path: item.target_path,
            filename: item.filename,
            total_bytes: item.total_bytes,
            received_bytes: item.received_bytes,
            state: item.state,
            error_message: item.error_message,
            started_at: item.started_at,
            completed_at: item.completed_at,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub custom_filename: Option<String>,
    pub target_dir: Option<PathBuf>,
    pub headers: HashMap<String, String>,
    pub allow_overwrite: bool,
}
