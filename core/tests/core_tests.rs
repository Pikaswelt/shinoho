use shinoho_core::CoreError;
use shinoho_core::api::TabManagerState;
use shinoho_core::tabs::{CreateTabRequest, TabInfo};

#[test]
fn test_core_tab_initialization() {
    let tab = TabInfo::new("tab-test", "about:blank");
    assert_eq!(tab.id, "tab-test");
    assert_eq!(tab.url, "about:blank");
}

#[test]
fn test_core_error_display() {
    let err = CoreError::TabNotFound("tab-99".to_string());
    assert_eq!(err.to_string(), "Tab nicht gefunden: tab-99");
}

#[test]
fn test_core_api_tab_manager_state() {
    let state = TabManagerState::new();
    let mut mgr = state.0.lock().unwrap();

    let tab = mgr.create_tab(Some(CreateTabRequest {
        url: Some("https://example.org".to_string()),
        activate: Some(true),
        is_pinned: Some(false),
        ..Default::default()
    }));

    assert_eq!(tab.url, "https://example.org");
    assert_eq!(mgr.get_tabs().len(), 2);
}

#[test]
fn test_tab_info_json_compatibility() {
    let tab = TabInfo::new("tab-1", "https://shinoho.org");
    let serialized = serde_json::to_value(&tab).unwrap();

    assert!(serialized.get("id").is_some());
    assert!(serialized.get("url").is_some());
    assert!(serialized.get("isLoading").is_some());
    assert!(serialized.get("canGoBack").is_some());
    assert!(serialized.get("canGoForward").is_some());
    assert!(serialized.get("isPinned").is_some());
    assert!(serialized.get("isMuted").is_some());
    assert!(serialized.get("isIncognito").is_some());
    assert!(serialized.get("createdAt").is_some());
}

#[test]
fn test_tab_manager_lifecycle_and_groups() {
    let mut mgr = shinoho_core::tabs::TabManager::new();
    assert_eq!(mgr.get_tabs().len(), 1);

    // Neuen Tab erstellen
    let tab2 = mgr.create_tab(Some(CreateTabRequest {
        url: Some("https://docs.rs".to_string()),
        activate: Some(true),
        ..Default::default()
    }));
    assert_eq!(mgr.get_tabs().len(), 2);
    assert_eq!(mgr.get_active_tab().unwrap().id, tab2.id);

    // Pinnen und Muten
    let pinned = mgr.set_pinned(&tab2.id, true).unwrap();
    assert!(pinned.is_pinned);
    let muted = mgr.set_muted(&tab2.id, true).unwrap();
    assert!(muted.is_muted);

    // Duplizieren
    let dup = mgr.duplicate_tab(&tab2.id).unwrap();
    assert_eq!(dup.url, "https://docs.rs");
    assert_eq!(mgr.get_tabs().len(), 3);

    // Tab-Gruppe erstellen
    let group = mgr.create_tab_group(shinoho_core::tabs::CreateTabGroupRequest {
        title: "Recherche".to_string(),
        color: "#ff0055".to_string(),
        tab_ids: vec![tab2.id.clone(), dup.id.clone()],
    });
    assert_eq!(group.title, "Recherche");
    assert_eq!(mgr.get_tab_groups().len(), 1);

    // Tab Reorder (move)
    mgr.move_tab(&dup.id, 0).unwrap();
    let tabs = mgr.get_tabs();
    assert_eq!(tabs[0].id, dup.id);

    // Unpin dup and tab2 so close_other_tabs closes others
    mgr.set_pinned(&dup.id, false).unwrap();
    let closed = mgr.close_other_tabs(&tab2.id).unwrap();
    assert!(closed.contains(&dup.id));
    let remaining_tabs = mgr.get_tabs();
    assert_eq!(remaining_tabs.len(), 1);
    assert_eq!(remaining_tabs[0].id, tab2.id);
}
