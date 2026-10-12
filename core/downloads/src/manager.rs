use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, RANGE, USER_AGENT};
use shinoho_storage::Storage;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::{RwLock, broadcast, watch};

use crate::error::DownloadError;
use crate::events::{DownloadEvent, DownloadProgressEvent};
use crate::item::{DownloadItem, DownloadOptions, DownloadState};
use crate::utils::{filename_from_url, resolve_unique_path, sanitize_filename};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadCommand {
    Run,
    Pause,
    Cancel,
}

pub struct DownloadManager {
    storage: Option<Arc<Storage>>,
    client: reqwest::Client,
    items: Arc<RwLock<HashMap<String, DownloadItem>>>,
    active_controllers: Arc<RwLock<HashMap<String, watch::Sender<DownloadCommand>>>>,
    event_sender: broadcast::Sender<DownloadEvent>,
    default_dir: PathBuf,
}

impl DownloadManager {
    pub fn new(storage: Option<Arc<Storage>>, default_dir: Option<PathBuf>) -> Self {
        let dir =
            default_dir.unwrap_or_else(|| dirs::download_dir().unwrap_or_else(std::env::temp_dir));

        let (event_sender, _) = broadcast::channel(128);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .unwrap_or_default();

        Self {
            storage,
            client,
            items: Arc::new(RwLock::new(HashMap::new())),
            active_controllers: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            default_dir: dir,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> {
        self.event_sender.subscribe()
    }

    pub fn default_dir(&self) -> &Path {
        &self.default_dir
    }

    pub async fn start_download(
        &self,
        url: &str,
        options: Option<DownloadOptions>,
    ) -> Result<DownloadItem, DownloadError> {
        let opts = options.unwrap_or_default();
        let target_dir = opts.target_dir.unwrap_or_else(|| self.default_dir.clone());
        tokio::fs::create_dir_all(&target_dir).await?;

        let initial_filename = opts
            .custom_filename
            .map(|f| sanitize_filename(&f))
            .unwrap_or_else(|| filename_from_url(url));

        let target_path = if opts.allow_overwrite {
            target_dir.join(&initial_filename)
        } else {
            resolve_unique_path(&target_dir, &initial_filename)
        };

        let target_path_str = target_path.to_string_lossy().to_string();
        let filename = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("download.bin")
            .to_string();

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        let item = DownloadItem {
            id: id.clone(),
            url: url.to_string(),
            target_path: target_path_str,
            filename,
            total_bytes: None,
            received_bytes: 0,
            state: DownloadState::Pending,
            error_message: None,
            speed_bps: 0,
            started_at: now,
            completed_at: None,
        };

        // Persist to storage
        if let Some(storage) = &self.storage {
            let st_item: shinoho_storage::DownloadItem = item.clone().into();
            storage.create_download(&st_item)?;
        }

        // Store in memory
        self.items.write().await.insert(id.clone(), item.clone());

        let _ = self.event_sender.send(DownloadEvent::Started(item.clone()));

        // Spawn async download task
        self.spawn_download_task(id, 0, opts.headers).await;

        Ok(item)
    }

    pub async fn pause_download(&self, id: &str) -> Result<(), DownloadError> {
        let controllers = self.active_controllers.read().await;
        if let Some(sender) = controllers.get(id) {
            let _ = sender.send(DownloadCommand::Pause);
            Ok(())
        } else {
            let mut items = self.items.write().await;
            if let Some(item) = items.get_mut(id) {
                if item.state == DownloadState::Downloading || item.state == DownloadState::Pending
                {
                    item.state = DownloadState::Paused;
                    item.speed_bps = 0;
                    if let Some(storage) = &self.storage {
                        let _ = storage.update_download_progress(
                            &item.id,
                            item.received_bytes,
                            item.total_bytes,
                            item.state.clone(),
                            item.error_message.clone(),
                        );
                    }
                    let _ = self
                        .event_sender
                        .send(DownloadEvent::Paused { id: id.to_string() });
                    Ok(())
                } else {
                    Err(DownloadError::InvalidState {
                        id: id.to_string(),
                        state: item.state.as_str().to_string(),
                    })
                }
            } else {
                Err(DownloadError::NotFound(id.to_string()))
            }
        }
    }

    pub async fn resume_download(&self, id: &str) -> Result<(), DownloadError> {
        let mut item = match self.get_download(id).await? {
            Some(it) => it,
            None => return Err(DownloadError::NotFound(id.to_string())),
        };

        if item.state != DownloadState::Paused && item.state != DownloadState::Failed {
            return Err(DownloadError::InvalidState {
                id: id.to_string(),
                state: item.state.as_str().to_string(),
            });
        }

        let existing_len = if Path::new(&item.target_path).exists() {
            tokio::fs::metadata(&item.target_path)
                .await
                .map(|m| m.len() as i64)
                .unwrap_or(0)
        } else {
            0
        };

        item.received_bytes = existing_len;
        item.state = DownloadState::Downloading;
        item.error_message = None;

        self.items
            .write()
            .await
            .insert(id.to_string(), item.clone());
        if let Some(storage) = &self.storage {
            let _ = storage.update_download_progress(
                &item.id,
                item.received_bytes,
                item.total_bytes,
                item.state.clone(),
                None,
            );
        }

        let _ = self
            .event_sender
            .send(DownloadEvent::Resumed { id: id.to_string() });

        self.spawn_download_task(id.to_string(), existing_len, HashMap::new())
            .await;

        Ok(())
    }

    pub async fn cancel_download(&self, id: &str) -> Result<(), DownloadError> {
        let controllers = self.active_controllers.read().await;
        if let Some(sender) = controllers.get(id) {
            let _ = sender.send(DownloadCommand::Cancel);
        }

        let mut items = self.items.write().await;
        if let Some(item) = items.get_mut(id) {
            item.state = DownloadState::Cancelled;
            item.speed_bps = 0;
            if let Some(storage) = &self.storage {
                let _ = storage.update_download_progress(
                    &item.id,
                    item.received_bytes,
                    item.total_bytes,
                    item.state.clone(),
                    None,
                );
            }
            let _ = self
                .event_sender
                .send(DownloadEvent::Cancelled { id: id.to_string() });
            Ok(())
        } else {
            Err(DownloadError::NotFound(id.to_string()))
        }
    }

    pub async fn retry_download(&self, id: &str) -> Result<DownloadItem, DownloadError> {
        let item = match self.get_download(id).await? {
            Some(it) => it,
            None => return Err(DownloadError::NotFound(id.to_string())),
        };

        if item.is_active() {
            return Err(DownloadError::InvalidState {
                id: id.to_string(),
                state: item.state.as_str().to_string(),
            });
        }

        // Delete partial file if exists
        let path = PathBuf::from(&item.target_path);
        let _ = tokio::fs::remove_file(path).await;

        let options = DownloadOptions {
            custom_filename: Some(item.filename),
            target_dir: Path::new(&item.target_path)
                .parent()
                .map(|p| p.to_path_buf()),
            headers: HashMap::new(),
            allow_overwrite: true,
        };

        self.start_download(&item.url, Some(options)).await
    }

    pub async fn get_download(&self, id: &str) -> Result<Option<DownloadItem>, DownloadError> {
        if let Some(item) = self.items.read().await.get(id) {
            return Ok(Some(item.clone()));
        }

        if let Some(storage) = &self.storage
            && let Some(st_item) = storage.get_download(id)?
        {
            let item: DownloadItem = st_item.into();
            self.items
                .write()
                .await
                .insert(id.to_string(), item.clone());
            return Ok(Some(item));
        }

        Ok(None)
    }

    pub async fn list_downloads(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<DownloadItem>, DownloadError> {
        if let Some(storage) = &self.storage {
            let st_items = storage.list_downloads(limit, offset)?;
            let mut results = Vec::new();
            let live_items = self.items.read().await;

            for item in st_items {
                if let Some(live) = live_items.get(&item.id) {
                    results.push(live.clone());
                } else {
                    results.push(item.into());
                }
            }
            Ok(results)
        } else {
            let items = self.items.read().await;
            let mut list: Vec<DownloadItem> = items.values().cloned().collect();
            list.sort_by_key(|a| std::cmp::Reverse(a.started_at));
            let paginated = list.into_iter().skip(offset).take(limit).collect();
            Ok(paginated)
        }
    }

    pub async fn delete_download(
        &self,
        id: &str,
        delete_file: bool,
    ) -> Result<bool, DownloadError> {
        let _ = self.cancel_download(id).await;

        let target_path = {
            let mut items = self.items.write().await;
            items.remove(id).map(|it| it.target_path)
        };

        if let Some(storage) = &self.storage {
            let _ = storage.delete_download(id)?;
        }

        if delete_file && let Some(path_str) = target_path {
            let p = PathBuf::from(path_str);
            let _ = tokio::fs::remove_file(p).await;
        }

        Ok(true)
    }

    pub async fn clear_completed(&self) -> Result<usize, DownloadError> {
        let mut count = 0;
        let mut to_remove = Vec::new();

        {
            let mut items = self.items.write().await;
            for (id, it) in items.iter() {
                if it.state == DownloadState::Completed {
                    to_remove.push(id.clone());
                }
            }
            for id in &to_remove {
                items.remove(id);
                count += 1;
            }
        }

        if let Some(storage) = &self.storage {
            for id in &to_remove {
                let _ = storage.delete_download(id)?;
            }
        }

        Ok(count)
    }

    async fn spawn_download_task(
        &self,
        id: String,
        range_start: i64,
        headers: HashMap<String, String>,
    ) {
        let (cmd_tx, cmd_rx) = watch::channel(DownloadCommand::Run);
        self.active_controllers
            .write()
            .await
            .insert(id.clone(), cmd_tx);

        let client = self.client.clone();
        let items = self.items.clone();
        let storage = self.storage.clone();
        let active_controllers = self.active_controllers.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            run_download_loop(
                id.clone(),
                range_start,
                headers,
                client,
                items,
                storage,
                cmd_rx,
                event_sender,
            )
            .await;

            active_controllers.write().await.remove(&id);
        });
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_download_loop(
    id: String,
    range_start: i64,
    headers: HashMap<String, String>,
    client: reqwest::Client,
    items: Arc<RwLock<HashMap<String, DownloadItem>>>,
    storage: Option<Arc<Storage>>,
    cmd_rx: watch::Receiver<DownloadCommand>,
    event_sender: broadcast::Sender<DownloadEvent>,
) {
    let (url, target_path, filename) = {
        let read = items.read().await;
        match read.get(&id) {
            Some(it) => (it.url.clone(), it.target_path.clone(), it.filename.clone()),
            None => return,
        }
    };

    // Transition to Downloading state
    {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(&id) {
            it.state = DownloadState::Downloading;
        }
    }

    let mut req_builder = client.get(&url);
    req_builder = req_builder.header(USER_AGENT, "Mozilla/5.0 Shinoho/0.1.0");

    for (k, v) in headers {
        req_builder = req_builder.header(k, v);
    }

    if range_start > 0 {
        req_builder = req_builder.header(RANGE, format!("bytes={range_start}-"));
    }

    let response = match req_builder.send().await {
        Ok(res) => res,
        Err(err) => {
            fail_download(&id, &err.to_string(), &items, &storage, &event_sender).await;
            return;
        }
    };

    if !response.status().is_success() {
        let msg = format!("HTTP {}", response.status());
        fail_download(&id, &msg, &items, &storage, &event_sender).await;
        return;
    }

    let is_partial = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    let content_len = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok());

    let total_bytes = if is_partial {
        content_len.map(|cl| cl + range_start)
    } else {
        content_len
    };

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .append(is_partial)
        .truncate(!is_partial)
        .open(&target_path)
        .await
    {
        Ok(f) => f,
        Err(err) => {
            fail_download(&id, &err.to_string(), &items, &storage, &event_sender).await;
            return;
        }
    };

    let mut received_bytes = if is_partial { range_start } else { 0 };

    {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(&id) {
            it.total_bytes = total_bytes;
            it.received_bytes = received_bytes;
        }
    }

    let mut stream = response.bytes_stream();
    let mut last_speed_update = Instant::now();
    let mut last_speed_bytes = received_bytes;
    let mut speed_bps = 0;
    let mut last_storage_update = Instant::now();

    while let Some(chunk_res) = stream.next().await {
        // Check for pause/cancel commands
        let cmd = *cmd_rx.borrow();
        if cmd != DownloadCommand::Run {
            match cmd {
                DownloadCommand::Pause => {
                    let _ = file.flush().await;
                    pause_download_state(
                        &id,
                        received_bytes,
                        total_bytes,
                        &items,
                        &storage,
                        &event_sender,
                    )
                    .await;
                    return;
                }
                DownloadCommand::Cancel => {
                    let _ = file.flush().await;
                    cancel_download_state(
                        &id,
                        received_bytes,
                        total_bytes,
                        &items,
                        &storage,
                        &event_sender,
                    )
                    .await;
                    return;
                }
                DownloadCommand::Run => {}
            }
        }

        let chunk = match chunk_res {
            Ok(c) => c,
            Err(err) => {
                fail_download(&id, &err.to_string(), &items, &storage, &event_sender).await;
                return;
            }
        };

        if let Err(err) = file.write_all(&chunk).await {
            fail_download(&id, &err.to_string(), &items, &storage, &event_sender).await;
            return;
        }

        received_bytes += chunk.len() as i64;

        let elapsed = last_speed_update.elapsed();
        if elapsed >= Duration::from_millis(250) {
            let secs = elapsed.as_secs_f64();
            if secs > 0.0 {
                speed_bps = ((received_bytes - last_speed_bytes) as f64 / secs) as u64;
            }
            last_speed_update = Instant::now();
            last_speed_bytes = received_bytes;

            // Update in-memory item
            {
                let mut write = items.write().await;
                if let Some(it) = write.get_mut(&id) {
                    it.received_bytes = received_bytes;
                    it.speed_bps = speed_bps;
                }
            }

            // Emit progress event
            let progress = DownloadProgressEvent {
                id: id.clone(),
                url: url.clone(),
                filename: filename.clone(),
                received_bytes,
                total_bytes,
                speed_bps,
                state: DownloadState::Downloading,
                progress_percent: total_bytes.and_then(|t| {
                    if t > 0 {
                        Some(((received_bytes as f64 / t as f64) * 100.0).clamp(0.0, 100.0) as f32)
                    } else {
                        None
                    }
                }),
            };
            let _ = event_sender.send(DownloadEvent::Progress(progress));
        }

        // Periodic storage persist (every 2 seconds)
        if last_storage_update.elapsed() >= Duration::from_secs(2) {
            if let Some(st) = storage.as_ref() {
                let _ = st.update_download_progress(
                    &id,
                    received_bytes,
                    total_bytes,
                    DownloadState::Downloading,
                    None,
                );
            }
            last_storage_update = Instant::now();
        }
    }

    // Flush file to disk
    if let Err(err) = file.flush().await {
        fail_download(&id, &err.to_string(), &items, &storage, &event_sender).await;
        return;
    }

    // Mark completed
    let now = chrono::Utc::now().timestamp_millis();
    let completed_item = {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(&id) {
            it.received_bytes = received_bytes;
            it.total_bytes = Some(received_bytes);
            it.state = DownloadState::Completed;
            it.speed_bps = 0;
            it.completed_at = Some(now);
            it.clone()
        } else {
            return;
        }
    };

    if let Some(st) = storage.as_ref() {
        let _ = st.update_download_progress(
            &id,
            received_bytes,
            Some(received_bytes),
            DownloadState::Completed,
            None,
        );
    }

    let _ = event_sender.send(DownloadEvent::Completed(completed_item));
}

async fn fail_download(
    id: &str,
    err: &str,
    items: &Arc<RwLock<HashMap<String, DownloadItem>>>,
    storage: &Option<Arc<Storage>>,
    event_sender: &broadcast::Sender<DownloadEvent>,
) {
    {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(id) {
            it.state = DownloadState::Failed;
            it.error_message = Some(err.to_string());
            it.speed_bps = 0;
        }
    }

    if let Some(st) = storage.as_ref() {
        let _ =
            st.update_download_progress(id, 0, None, DownloadState::Failed, Some(err.to_string()));
    }

    let _ = event_sender.send(DownloadEvent::Failed {
        id: id.to_string(),
        error: err.to_string(),
    });
}

async fn pause_download_state(
    id: &str,
    received_bytes: i64,
    total_bytes: Option<i64>,
    items: &Arc<RwLock<HashMap<String, DownloadItem>>>,
    storage: &Option<Arc<Storage>>,
    event_sender: &broadcast::Sender<DownloadEvent>,
) {
    {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(id) {
            it.state = DownloadState::Paused;
            it.received_bytes = received_bytes;
            it.speed_bps = 0;
        }
    }

    if let Some(st) = storage.as_ref() {
        let _ = st.update_download_progress(
            id,
            received_bytes,
            total_bytes,
            DownloadState::Paused,
            None,
        );
    }

    let _ = event_sender.send(DownloadEvent::Paused { id: id.to_string() });
}

async fn cancel_download_state(
    id: &str,
    received_bytes: i64,
    total_bytes: Option<i64>,
    items: &Arc<RwLock<HashMap<String, DownloadItem>>>,
    storage: &Option<Arc<Storage>>,
    event_sender: &broadcast::Sender<DownloadEvent>,
) {
    {
        let mut write = items.write().await;
        if let Some(it) = write.get_mut(id) {
            it.state = DownloadState::Cancelled;
            it.received_bytes = received_bytes;
            it.speed_bps = 0;
        }
    }

    if let Some(st) = storage.as_ref() {
        let _ = st.update_download_progress(
            id,
            received_bytes,
            total_bytes,
            DownloadState::Cancelled,
            None,
        );
    }

    let _ = event_sender.send(DownloadEvent::Cancelled { id: id.to_string() });
}
