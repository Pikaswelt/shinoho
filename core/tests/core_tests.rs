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
    assert!(serialized.get("createdAt").is_some());
}
