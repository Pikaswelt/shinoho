use serde::{Deserialize, Serialize};
use shinoho_storage::{
    CreateBookmarkParams, DownloadItem, DownloadState, Storage, UpdateBookmarkParams,
};

#[test]
fn test_history_crud_and_queries() {
    let storage = Storage::open_in_memory().expect("open in memory");

    // Record visits
    let v1 = storage
        .record_visit(
            "https://shinoho.org",
            "Shinoho Home",
            true,
            Some("https://shinoho.org/favicon.ico"),
        )
        .expect("record visit 1");
    assert_eq!(v1.visit_count, 1);
    assert!(v1.typed);

    // Revisit the same url
    let v1_again = storage
        .record_visit(
            "https://shinoho.org",
            "Shinoho - Next Gen Browser",
            false,
            None,
        )
        .expect("record visit 1 again");
    assert_eq!(v1_again.id, v1.id);
    assert_eq!(v1_again.visit_count, 2);
    assert_eq!(v1_again.title, "Shinoho - Next Gen Browser");
    // favicon preserved
    assert_eq!(
        v1_again.favicon_url.as_deref(),
        Some("https://shinoho.org/favicon.ico")
    );

    // Add another site
    storage
        .record_visit("https://rust-lang.org", "Rust Language", true, None)
        .expect("record visit 2");

    let history = storage.get_history(10, 0).expect("get history");
    assert_eq!(history.len(), 2);

    // Search history
    let search_results = storage.search_history("Browser", 10).expect("search");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].url, "https://shinoho.org");

    // Top sites
    let top_sites = storage.get_top_sites(10).expect("top sites");
    assert_eq!(top_sites.len(), 2);
    assert_eq!(top_sites[0].url, "https://shinoho.org");
    assert_eq!(top_sites[0].visit_count, 2);

    // Delete single visit
    let deleted = storage.delete_history_visit(v1.id).expect("delete visit");
    assert!(deleted);
    let history_after = storage
        .get_history(10, 0)
        .expect("get history after delete");
    assert_eq!(history_after.len(), 1);

    // Clear history
    let cleared = storage.clear_history().expect("clear history");
    assert_eq!(cleared, 1);
    let history_empty = storage.get_history(10, 0).expect("get empty history");
    assert_eq!(history_empty.len(), 0);
}

#[test]
fn test_bookmarks_and_tree() {
    let storage = Storage::open_in_memory().expect("open in memory");

    // Root folders should exist
    let toolbar = storage.get_bookmark("toolbar").expect("get toolbar");
    assert!(toolbar.is_some());
    assert!(toolbar.unwrap().is_folder);

    // Create a folder in toolbar
    let dev_folder = storage
        .create_bookmark(CreateBookmarkParams {
            title: "Development".to_string(),
            parent_id: Some("toolbar".to_string()),
            is_folder: true,
            ..Default::default()
        })
        .expect("create folder");
    assert!(dev_folder.is_folder);

    // Create bookmark inside dev folder
    let b1 = storage
        .create_bookmark(CreateBookmarkParams {
            title: "GitHub".to_string(),
            url: Some("https://github.com".to_string()),
            parent_id: Some(dev_folder.id.clone()),
            tags: vec!["git".to_string(), "code".to_string()],
            ..Default::default()
        })
        .expect("create bookmark 1");
    assert_eq!(b1.parent_id.as_deref(), Some(dev_folder.id.as_str()));
    assert_eq!(b1.tags, vec!["git", "code"]);

    // Create bookmark in toolbar directly
    let b2 = storage
        .create_bookmark(CreateBookmarkParams {
            title: "Docs".to_string(),
            url: Some("https://docs.rs".to_string()),
            parent_id: Some("toolbar".to_string()),
            ..Default::default()
        })
        .expect("create bookmark 2");

    // Check tree structure
    let tree = storage.get_bookmark_tree(Some("toolbar")).expect("tree");
    assert_eq!(tree.len(), 2); // dev_folder and Docs
    let dev_node = tree.iter().find(|n| n.item.id == dev_folder.id).unwrap();
    assert_eq!(dev_node.children.len(), 1);
    assert_eq!(dev_node.children[0].item.id, b1.id);

    // Search bookmarks
    let search_res = storage.search_bookmarks("git").expect("search");
    assert_eq!(search_res.len(), 1);
    assert_eq!(search_res[0].id, b1.id);

    // Reorder
    storage
        .reorder_bookmarks(Some("toolbar"), &[b2.id.clone(), dev_folder.id.clone()])
        .expect("reorder");
    let list_reordered = storage.list_bookmarks(Some("toolbar")).expect("list");
    assert_eq!(list_reordered[0].id, b2.id);
    assert_eq!(list_reordered[1].id, dev_folder.id);

    // Update bookmark
    let updated = storage
        .update_bookmark(
            &b1.id,
            UpdateBookmarkParams {
                title: Some("GitHub Portal".to_string()),
                ..Default::default()
            },
        )
        .expect("update");
    assert_eq!(updated.title, "GitHub Portal");

    // Deleting root folder must fail
    assert!(storage.delete_bookmark("toolbar").is_err());

    // Deleting dev_folder cascades to b1
    let deleted = storage
        .delete_bookmark(&dev_folder.id)
        .expect("delete folder");
    assert!(deleted);
    assert!(storage.get_bookmark(&b1.id).expect("get b1").is_none());
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ThemeConfig {
    name: String,
    dark_mode: bool,
    accent_color: String,
}

#[test]
fn test_settings_kv_and_json() {
    let storage = Storage::open_in_memory().expect("open in memory");

    // String setting
    storage
        .set_setting("browser.search_engine", "duckduckgo")
        .expect("set setting");
    let val = storage
        .get_setting("browser.search_engine")
        .expect("get setting");
    assert_eq!(val.as_deref(), Some("duckduckgo"));

    // JSON setting
    let theme = ThemeConfig {
        name: "cyberpunk".to_string(),
        dark_mode: true,
        accent_color: "#ff007f".to_string(),
    };
    storage
        .set_setting_json("ui.theme", &theme)
        .expect("set json");
    let loaded_theme: Option<ThemeConfig> = storage.get_setting_json("ui.theme").expect("get json");
    assert_eq!(loaded_theme, Some(theme));

    // Get all settings
    let all = storage.get_all_settings().expect("all settings");
    assert!(all.contains_key("browser.search_engine"));
    assert!(all.contains_key("ui.theme"));

    // Delete setting
    let deleted = storage
        .delete_setting("browser.search_engine")
        .expect("delete setting");
    assert!(deleted);
    assert!(storage
        .get_setting("browser.search_engine")
        .expect("get")
        .is_none());
}

#[test]
fn test_downloads() {
    let storage = Storage::open_in_memory().expect("open in memory");

    let item = DownloadItem {
        id: "dl-1".to_string(),
        url: "https://example.com/file.zip".to_string(),
        target_path: "/downloads/file.zip".to_string(),
        filename: "file.zip".to_string(),
        total_bytes: Some(1024 * 1024),
        received_bytes: 0,
        state: DownloadState::Downloading,
        error_message: None,
        started_at: chrono::Utc::now().timestamp_millis(),
        completed_at: None,
    };

    storage.create_download(&item).expect("create download");

    let loaded = storage.get_download("dl-1").expect("get download").unwrap();
    assert_eq!(loaded.filename, "file.zip");
    assert_eq!(loaded.state, DownloadState::Downloading);

    // Progress update
    storage
        .update_download_progress(
            "dl-1",
            1024 * 1024,
            Some(1024 * 1024),
            DownloadState::Completed,
            None,
        )
        .expect("update progress");

    let completed = storage.get_download("dl-1").expect("get download").unwrap();
    assert_eq!(completed.state, DownloadState::Completed);
    assert!(completed.completed_at.is_some());

    let list = storage.list_downloads(10, 0).expect("list");
    assert_eq!(list.len(), 1);

    let deleted = storage.delete_download("dl-1").expect("delete download");
    assert!(deleted);
    assert!(storage.get_download("dl-1").expect("get").is_none());
}

#[test]
fn test_persistent_disk_storage() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("shinoho_test.db");

    {
        let storage = Storage::open(&db_path).expect("open disk storage");
        assert!(!storage.is_in_memory());
        assert_eq!(storage.path(), Some(db_path.as_path()));

        storage
            .set_setting("persist.key", "persisted_value")
            .expect("set persist");
        storage
            .record_visit("https://persisted.com", "Persisted Page", true, None)
            .expect("record persist visit");
    }

    // Reopen and check data
    {
        let storage = Storage::open(&db_path).expect("reopen disk storage");
        let val = storage.get_setting("persist.key").expect("get persist");
        assert_eq!(val.as_deref(), Some("persisted_value"));

        let hist = storage.get_history(10, 0).expect("get history");
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].url, "https://persisted.com");
    }
}
