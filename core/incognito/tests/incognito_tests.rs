use shinoho_incognito::{IncognitoConfig, IncognitoManager, IncognitoSession};

#[test]
fn test_session_lifecycle_and_directory_cleanup() {
    let session = IncognitoSession::new("test-session-1".to_string(), IncognitoConfig::default())
        .expect("create session");

    let data_dir = session.data_directory_path();
    assert!(
        data_dir.exists(),
        "Temp dir should exist on disk while session is open"
    );

    // Storage is in-memory
    assert!(session.storage().is_in_memory());

    // Record a visit in incognito storage
    let visit = session
        .storage()
        .record_visit("https://secret-anime.org", "Secret Anime", true, None)
        .expect("record visit");
    assert_eq!(visit.title, "Secret Anime");

    // Explicitly wipe session
    session.wipe().expect("wipe");

    // Temp directory must be gone
    assert!(
        !data_dir.exists(),
        "Temp dir must be wiped after session ends"
    );
}

#[test]
fn test_manager_auto_close_when_all_tabs_closed() {
    let manager = IncognitoManager::default();

    let id = manager.create_session(None, None).expect("create session");
    assert!(manager.has_session(&id).expect("has session"));

    manager.register_tab(&id, "tab-1").expect("register tab 1");
    manager.register_tab(&id, "tab-2").expect("register tab 2");

    // Unregister tab 1 (not empty yet)
    let closed = manager
        .unregister_tab(&id, "tab-1", true)
        .expect("unregister tab 1");
    assert!(!closed);
    assert!(manager.has_session(&id).expect("has session"));

    // Unregister tab 2 (now empty, auto-close triggers)
    let closed = manager
        .unregister_tab(&id, "tab-2", true)
        .expect("unregister tab 2");
    assert!(closed);
    assert!(!manager.has_session(&id).expect("session should be closed"));
}

#[test]
fn test_isolated_sessions() {
    let manager = IncognitoManager::default();

    let id1 = manager.create_session(None, None).expect("session 1");
    let id2 = manager.create_session(None, None).expect("session 2");

    // Add visit in session 1
    manager
        .with_session_storage(&id1, |storage| {
            storage
                .record_visit("https://session1.com", "Session 1", true, None)
                .expect("record");
        })
        .expect("session 1 storage");

    // Session 2 history should be empty
    manager
        .with_session_storage(&id2, |storage| {
            let hist = storage.get_history(10, 0).expect("get history");
            assert_eq!(hist.len(), 0, "Session 2 must be isolated from Session 1");
        })
        .expect("session 2 storage");

    // Session 1 history has 1 entry
    manager
        .with_session_storage(&id1, |storage| {
            let hist = storage.get_history(10, 0).expect("get history");
            assert_eq!(hist.len(), 1);
            assert_eq!(hist[0].url, "https://session1.com");
        })
        .expect("session 1 storage");

    manager.close_all_sessions().expect("close all");
    assert_eq!(manager.active_session_count().expect("count"), 0);
}
