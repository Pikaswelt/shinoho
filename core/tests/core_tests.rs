use shinoho_core::CoreError;
use shinoho_core::tabs::TabInfo;

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
