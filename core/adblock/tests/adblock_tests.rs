use shinoho_adblock::AdblockEngine;

#[test]
fn test_default_rules_blocking() {
    let blocker = AdblockEngine::with_default_rules();

    // Tracker should be blocked
    let res = blocker.check_network_request(
        "https://google-analytics.com/analytics.js",
        "https://example.com",
        "script",
    );
    assert!(res.blocked, "Google Analytics must be blocked");

    // Ad server should be blocked
    let res2 = blocker.check_network_request(
        "https://doubleclick.net/ad.js",
        "https://news.com",
        "script",
    );
    assert!(res2.blocked, "DoubleClick must be blocked");

    // Legitimate asset should not be blocked
    let res3 = blocker.check_network_request(
        "https://shinoho.org/style.css",
        "https://shinoho.org",
        "stylesheet",
    );
    assert!(!res3.blocked, "Shinoho asset should not be blocked");

    let stats = blocker.stats();
    assert_eq!(stats.total_checked, 3);
    assert_eq!(stats.total_blocked, 2);
}

#[test]
fn test_custom_rules_and_filter_list() {
    let blocker = AdblockEngine::new();

    let res_before = blocker.check_network_request(
        "https://custom-ad-network.com/banner.gif",
        "https://example.com",
        "image",
    );
    assert!(!res_before.blocked);

    blocker
        .add_filter_rules(&["||custom-ad-network.com^".to_string()])
        .expect("add rule");

    let res_after = blocker.check_network_request(
        "https://custom-ad-network.com/banner.gif",
        "https://example.com",
        "image",
    );
    assert!(
        res_after.blocked,
        "Custom ad network must be blocked after adding rule"
    );
}

#[test]
fn test_whitelist_domain() {
    let blocker = AdblockEngine::with_default_rules();

    // Before whitelisting
    let res = blocker.check_network_request(
        "https://google-analytics.com/analytics.js",
        "https://friendly-site.org",
        "script",
    );
    assert!(res.blocked);

    // Whitelist friendly-site.org
    blocker.whitelist_domain("friendly-site.org");
    assert!(blocker.is_domain_whitelisted("friendly-site.org"));

    let res_whitelisted = blocker.check_network_request(
        "https://google-analytics.com/analytics.js",
        "https://friendly-site.org",
        "script",
    );
    assert!(!res_whitelisted.blocked);
    assert!(res_whitelisted.exception);
}

#[test]
fn test_toggle_enabled() {
    let blocker = AdblockEngine::with_default_rules();
    assert!(blocker.is_enabled());

    blocker.set_enabled(false);
    assert!(!blocker.is_enabled());

    let res = blocker.check_network_request(
        "https://google-analytics.com/analytics.js",
        "https://example.com",
        "script",
    );
    assert!(!res.blocked, "Disabled blocker should not block requests");

    blocker.set_enabled(true);
    let res2 = blocker.check_network_request(
        "https://google-analytics.com/analytics.js",
        "https://example.com",
        "script",
    );
    assert!(res2.blocked, "Re-enabled blocker must block requests");
}

#[test]
fn test_cosmetic_filtering_css() {
    let blocker = AdblockEngine::with_default_rules();
    let css = blocker.hidden_elements_css("https://example.com");
    if let Some(css_rules) = css {
        assert!(css_rules.contains("display: none !important"));
    }
}
