use shinoho_core::userstyles::{
    AddUserStyleRequest, StyleRunAt, UpdateUserStyleRequest, UserStylesManager, UserStylesState,
    matches_pattern,
};

#[test]
fn test_url_matcher_patterns() {
    // 1. WebExtension match patterns
    assert!(matches_pattern(
        "*://*.youtube.com/*",
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    ));
    assert!(matches_pattern(
        "*://*.youtube.com/*",
        "http://m.youtube.com/feed"
    ));
    assert!(!matches_pattern(
        "*://*.youtube.com/*",
        "https://dailymotion.com/video/123"
    ));

    // 2. Exact schema & domain
    assert!(matches_pattern(
        "https://github.com/*",
        "https://github.com/Pikaswelt/shinoho"
    ));
    assert!(!matches_pattern(
        "http://github.com/*",
        "https://github.com/Pikaswelt/shinoho"
    ));

    // 3. Simple domain
    assert!(matches_pattern(
        "crunchyroll.com",
        "https://www.crunchyroll.com/de/series"
    ));
    assert!(matches_pattern(
        "*.crunchyroll.com",
        "https://beta.crunchyroll.com/anime"
    ));

    // 4. All URLs
    assert!(matches_pattern("<all_urls>", "https://example.org"));
    assert!(matches_pattern("*", "http://localhost:8080"));
}

#[test]
fn test_userstyles_manager_crud_and_matching() {
    let mut mgr = UserStylesManager::new();

    // Regel 1: Dark Mode fuer YouTube
    let r1 = mgr.add_rule(AddUserStyleRequest {
        id: Some("pikaswelt.dark-youtube".to_string()),
        name: "Dark YouTube".to_string(),
        match_patterns: vec!["*://*.youtube.com/*".to_string()],
        exclude_patterns: Some(vec!["*://*.youtube.com/embed/*".to_string()]),
        css: "body { background: #121212 !important; color: #fff !important; }".to_string(),
        enabled: Some(true),
        run_at: Some(StyleRunAt::DocumentStart),
        mod_id: Some("pikaswelt.dark-youtube".to_string()),
        priority: Some(10),
    });

    assert_eq!(r1.id, "pikaswelt.dark-youtube");
    assert_eq!(r1.name, "Dark YouTube");
    assert_eq!(mgr.get_rules().len(), 1);

    // Regel 2: Globaler Font
    let r2 = mgr.add_rule(AddUserStyleRequest {
        id: None,
        name: "Global Clean Font".to_string(),
        match_patterns: vec!["<all_urls>".to_string()],
        exclude_patterns: None,
        css: "* { font-family: 'Nunito', sans-serif !important; }".to_string(),
        enabled: Some(true),
        run_at: Some(StyleRunAt::DocumentEnd),
        mod_id: None,
        priority: Some(1),
    });

    assert!(r2.id.starts_with("userstyle-"));
    assert_eq!(mgr.get_rules().len(), 2);

    // Match fuer YouTube: Sollte beide Regeln matchen (r2 Prio 1, r1 Prio 10)
    let yt_url = "https://www.youtube.com/watch?v=123";
    let yt_matched = mgr.matching_rules_for_url(yt_url);
    assert_eq!(yt_matched.len(), 2);
    // r2 (prio 1) kommt vor r1 (prio 10)
    assert_eq!(yt_matched[0].id, r2.id);
    assert_eq!(yt_matched[1].id, r1.id);

    // Exclude Test: YouTube Embed
    let yt_embed = "https://www.youtube.com/embed/123";
    let embed_matched = mgr.matching_rules_for_url(yt_embed);
    // r1 ist excluded, nur r2 matcht
    assert_eq!(embed_matched.len(), 1);
    assert_eq!(embed_matched[0].id, r2.id);

    // Match fuer andere Seite: nur r2
    let github_url = "https://github.com/Pikaswelt/shinoho";
    let gh_matched = mgr.matching_rules_for_url(github_url);
    assert_eq!(gh_matched.len(), 1);
    assert_eq!(gh_matched[0].id, r2.id);

    // Toggle Regel: r1 deaktivieren
    let toggled = mgr.toggle_rule(&r1.id, false).unwrap();
    assert!(!toggled.enabled);
    let yt_after_disable = mgr.matching_rules_for_url(yt_url);
    assert_eq!(yt_after_disable.len(), 1);
    assert_eq!(yt_after_disable[0].id, r2.id);

    // Update Regel
    let updated = mgr
        .update_rule(UpdateUserStyleRequest {
            id: r1.id.clone(),
            name: Some("Ultra Dark YouTube".to_string()),
            match_patterns: None,
            exclude_patterns: None,
            css: Some("body { background: #000 !important; }".to_string()),
            enabled: Some(true),
            run_at: None,
            priority: None,
        })
        .unwrap();
    assert_eq!(updated.name, "Ultra Dark YouTube");
    assert_eq!(updated.css, "body { background: #000 !important; }");

    // Remove Regel
    let removed = mgr.remove_rule(&r2.id);
    assert!(removed);
    assert_eq!(mgr.get_rules().len(), 1);
}

#[test]
fn test_injection_script_generation() {
    let mut mgr = UserStylesManager::new();
    mgr.add_rule(AddUserStyleRequest {
        id: Some("custom-dark".to_string()),
        name: "Custom Dark".to_string(),
        match_patterns: vec!["https://anime.example.com/*".to_string()],
        exclude_patterns: None,
        css: "h1 { color: #ff6fae; }".to_string(),
        enabled: Some(true),
        run_at: Some(StyleRunAt::DocumentEnd),
        mod_id: None,
        priority: Some(5),
    });

    let script = mgr
        .generate_injection_script("https://anime.example.com/watch/ep1")
        .expect("Skript sollte generiert werden");

    assert!(script.contains("shinoho-userstyles-root"));
    assert!(script.contains("data-shinoho-userstyles"));
    assert!(script.contains("custom-dark"));
    assert!(script.contains("#ff6fae"));

    // Keine Injektion fuer nicht-passende URLs
    let none_script = mgr.generate_injection_script("https://other.org");
    assert!(none_script.is_none());
}

#[test]
fn test_userstyles_state_thread_safety() {
    let state = UserStylesState::new();
    let mut mgr = state.0.lock().unwrap();
    let rule = mgr.add_rule(AddUserStyleRequest {
        id: Some("test-rule".to_string()),
        name: "Test".to_string(),
        match_patterns: vec!["*".to_string()],
        exclude_patterns: None,
        css: "body { margin: 0; }".to_string(),
        enabled: Some(true),
        run_at: None,
        mod_id: None,
        priority: None,
    });
    assert_eq!(rule.id, "test-rule");
}
