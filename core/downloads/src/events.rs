use serde::{Deserialize, Serialize};

use crate::item::{DownloadItem, DownloadState};

pub const EVENT_DOWNLOAD_STARTED: &str = "shinoho://download-started";
pub const EVENT_DOWNLOAD_PROGRESS: &str = "shinoho://download-progress";
pub const EVENT_DOWNLOAD_PAUSED: &str = "shinoho://download-paused";
pub const EVENT_DOWNLOAD_RESUMED: &str = "shinoho://download-resumed";
pub const EVENT_DOWNLOAD_COMPLETED: &str = "shinoho://download-completed";
pub const EVENT_DOWNLOAD_CANCELLED: &str = "shinoho://download-cancelled";
pub const EVENT_DOWNLOAD_FAILED: &str = "shinoho://download-failed";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub received_bytes: i64,
    pub total_bytes: Option<i64>,
    pub speed_bps: u64,
    pub state: DownloadState,
    pub progress_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DownloadEvent {
    Started(DownloadItem),
    Progress(DownloadProgressEvent),
    Paused { id: String },
    Resumed { id: String },
    Completed(DownloadItem),
    Cancelled { id: String },
    Failed { id: String, error: String },
}
