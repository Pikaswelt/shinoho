use std::sync::Arc;
use std::time::Duration;

use shinoho_downloads::{DownloadEvent, DownloadItem, DownloadManager, DownloadState};
use shinoho_storage::Storage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Helper mock HTTP server for testing real streaming downloads without external internet.
async fn spawn_mock_http_server(body: Vec<u8>) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://127.0.0.1:{}/testfile.bin", addr.port());

    let handle = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let body_clone = body.clone();
            tokio::spawn(async move {
                let mut buf = [0u8; 2048];
                let n = socket.read(&mut buf).await.unwrap_or(0);
                let req_str = String::from_utf8_lossy(&buf[..n]);

                // Check Range header
                let (status, slice) = if let Some(range_line) = req_str
                    .lines()
                    .find(|l| l.to_lowercase().starts_with("range:"))
                {
                    if let Some(start_str) = range_line.split('=').nth(1) {
                        let start: usize =
                            start_str.trim().trim_end_matches('-').parse().unwrap_or(0);
                        if start < body_clone.len() {
                            (
                                format!(
                                    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {}-{}/{}\r\n",
                                    start,
                                    body_clone.len() - 1,
                                    body_clone.len()
                                ),
                                &body_clone[start..],
                            )
                        } else {
                            (
                                "HTTP/1.1 416 Range Not Satisfiable\r\n".to_string(),
                                &body_clone[0..0],
                            )
                        }
                    } else {
                        ("HTTP/1.1 200 OK\r\n".to_string(), &body_clone[..])
                    }
                } else {
                    ("HTTP/1.1 200 OK\r\n".to_string(), &body_clone[..])
                };

                let resp = format!(
                    "{}Content-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                    status,
                    slice.len()
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                let _ = socket.write_all(slice).await;
                let _ = socket.flush().await;
            });
        }
    });

    (url, handle)
}

#[tokio::test]
async fn test_download_lifecycle_and_storage_sync() {
    let storage = Arc::new(Storage::open_in_memory().unwrap());
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = DownloadManager::new(Some(storage.clone()), Some(temp_dir.path().to_path_buf()));

    let test_data = b"Hello, Shinoho Browser Downloads!".repeat(20);
    let (url, _server) = spawn_mock_http_server(test_data.clone()).await;

    let mut event_rx = manager.subscribe();

    let item = manager
        .start_download(&url, None)
        .await
        .expect("start download");

    assert_eq!(item.state, DownloadState::Pending);

    // Wait for completed event
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                match event {
                    Ok(DownloadEvent::Completed(c)) if c.id == item.id => {
                        break;
                    }
                    Ok(DownloadEvent::Failed { id, error }) if id == item.id => {
                        panic!("Download failed: {error}");
                    }
                    _ => {}
                }
            }
            _ = &mut timeout => {
                panic!("Timeout waiting for download completion");
            }
        }
    }

    // Check disk content
    let final_item = manager.get_download(&item.id).await.unwrap().unwrap();
    assert_eq!(final_item.state, DownloadState::Completed);
    assert_eq!(final_item.received_bytes, test_data.len() as i64);

    let disk_bytes = std::fs::read(&final_item.target_path).unwrap();
    assert_eq!(disk_bytes, test_data);

    // Verify persistence in storage
    let st_item = storage.get_download(&item.id).unwrap().unwrap();
    assert_eq!(st_item.state, DownloadState::Completed);
    assert_eq!(st_item.received_bytes, test_data.len() as i64);
}

#[tokio::test]
async fn test_cancel_download() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = DownloadManager::new(None, Some(temp_dir.path().to_path_buf()));

    // Large payload to allow cancellation before completion
    let test_data = vec![42u8; 1024 * 1024 * 5];
    let (url, _server) = spawn_mock_http_server(test_data).await;

    let item = manager
        .start_download(&url, None)
        .await
        .expect("start download");

    // Cancel download
    manager.cancel_download(&item.id).await.expect("cancel");

    tokio::time::sleep(Duration::from_millis(50)).await;

    let updated = manager.get_download(&item.id).await.unwrap().unwrap();
    assert_eq!(updated.state, DownloadState::Cancelled);
}

#[tokio::test]
async fn test_list_and_delete_download() {
    let storage = Arc::new(Storage::open_in_memory().unwrap());
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = DownloadManager::new(Some(storage.clone()), Some(temp_dir.path().to_path_buf()));

    let (url, _server) = spawn_mock_http_server(b"small data".to_vec()).await;

    let item = manager
        .start_download(&url, None)
        .await
        .expect("start download");

    let list = manager.list_downloads(10, 0).await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, item.id);

    // Delete download
    let deleted = manager
        .delete_download(&item.id, true)
        .await
        .expect("delete");
    assert!(deleted);

    let get_after = manager.get_download(&item.id).await.unwrap();
    assert!(get_after.is_none());
}

#[tokio::test]
async fn test_unique_filename_resolution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manager = DownloadManager::new(None, Some(temp_dir.path().to_path_buf()));

    // Pre-create file with same name
    let existing_file = temp_dir.path().join("testfile.bin");
    std::fs::write(&existing_file, b"existing").unwrap();

    let (url, _server) = spawn_mock_http_server(b"new data".to_vec()).await;

    let item = manager
        .start_download(&url, None)
        .await
        .expect("start download");

    assert_eq!(item.filename, "testfile (1).bin");
}

#[test]
fn test_download_item_helpers() {
    let mut item = DownloadItem {
        id: "test".to_string(),
        url: "https://example.com/file.zip".to_string(),
        target_path: "/tmp/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: Some(1000),
        received_bytes: 250,
        state: DownloadState::Downloading,
        error_message: None,
        speed_bps: 50000,
        started_at: 1000,
        completed_at: None,
    };

    assert_eq!(item.progress_percent(), Some(25.0));
    assert!(item.is_active());
    assert!(!item.is_finished());

    item.state = DownloadState::Completed;
    assert!(!item.is_active());
    assert!(item.is_finished());
}
