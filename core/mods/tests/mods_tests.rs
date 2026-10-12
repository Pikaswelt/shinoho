use std::fs;
use std::path::Path;

use shinoho_mods::*;

const SAKURA: &str = r#"{
  "id": "pikaswelt.sakura-night",
  "name": "Sakura Night",
  "version": "1.0.0",
  "apiVersion": 0,
  "author": "Pikaswelt",
  "theme": "theme.css",
  "sites": [
    { "match": ["*://*.youtube.com/*"], "css": ["sites/youtube.css"] }
  ],
  "widgets": [ { "id": "clock", "entry": "widgets/clock.html", "area": "hub" } ],
  "settings": [
    { "key": "petals", "type": "boolean", "default": true, "label": "Kirschblüten anzeigen" }
  ],
  "permissions": ["ui.theme", "storage"]
}"#;

fn write_mod(dir: &Path, manifest: &str) {
    fs::create_dir_all(dir.join("sites")).unwrap();
    fs::create_dir_all(dir.join("widgets")).unwrap();
    fs::write(dir.join("mod.json"), manifest).unwrap();
    fs::write(dir.join("theme.css"), ":root { --sh-accent: #ff6fae; }").unwrap();
    fs::write(
        dir.join("sites/youtube.css"),
        "ytd-app { filter: hue-rotate(20deg); }",
    )
    .unwrap();
    fs::write(dir.join("widgets/clock.html"), "<div>12:00</div>").unwrap();
}

fn with(json: &str, field: &str, value: serde_json::Value) -> Manifest {
    let mut v: serde_json::Value = serde_json::from_str(json).unwrap();
    v[field] = value;
    serde_json::from_value(v).unwrap()
}

#[test]
fn example_manifest_from_docs_is_valid() {
    let m = Manifest::from_json(SAKURA).unwrap();
    m.validate(None).unwrap();
    assert_eq!(
        m.files(),
        ["theme.css", "sites/youtube.css", "widgets/clock.html"]
    );
}

#[test]
fn invalid_manifests_are_rejected() {
    use serde_json::json;
    let cases = [
        with(SAKURA, "id", json!("NoDot")),
        with(SAKURA, "version", json!("1.0")),
        with(SAKURA, "apiVersion", json!(7)),
        with(SAKURA, "theme", json!("../../evil.css")),
        with(SAKURA, "theme", json!("/etc/passwd")),
        with(SAKURA, "permissions", json!(["net"])),
        with(
            SAKURA,
            "sites",
            json!([{ "match": ["*://*.x.com/*"], "js": ["a.js"] }]),
        ),
        with(SAKURA, "sites", json!([{ "match": ["ftp://x/*"] }])),
        with(
            SAKURA,
            "settings",
            json!([{ "key": "a", "type": "select" }]),
        ),
        with(
            SAKURA,
            "widgets",
            json!([{ "id": "w", "entry": "w.html", "area": "toolbar" }]),
        ),
    ];
    for m in cases {
        assert!(m.validate(None).is_err(), "sollte ungültig sein: {m:?}");
    }
    assert!(Manifest::from_json(r#"{"id":"a.b"}"#).is_err());
    assert!(Manifest::from_json(&SAKURA.replace("\"storage\"", "\"root\"")).is_err());
}

#[test]
fn critical_permissions_are_flagged() {
    use serde_json::json;
    let mut v: serde_json::Value = serde_json::from_str(SAKURA).unwrap();
    v["permissions"] = json!(["sites", "net", "storage"]);
    v["hosts"] = json!(["<all_urls>"]);
    v["sites"] = json!([{ "match": ["<all_urls>"], "js": ["sites/youtube.css"] }]);
    let m: Manifest = serde_json::from_value(v).unwrap();
    m.validate(None).unwrap();
    let critical: Vec<Permission> = m
        .permission_notices()
        .into_iter()
        .filter(|n| n.critical)
        .map(|n| n.permission)
        .collect();
    assert_eq!(critical, [Permission::Sites, Permission::Net]);
}

#[test]
fn match_patterns() {
    let yt = MatchPattern::parse("*://*.youtube.com/*").unwrap();
    assert!(yt.matches("https://www.youtube.com/watch?v=1"));
    assert!(yt.matches("http://youtube.com/"));
    assert!(!yt.matches("https://notyoutube.com/"));
    assert!(!yt.matches("https://youtube.com.evil.io/"));
    assert!(!yt.matches("file:///C:/youtube.com/"));

    let anilist = MatchPattern::parse("https://anilist.co/anime/*").unwrap();
    assert!(anilist.matches("https://anilist.co/anime/1"));
    assert!(!anilist.matches("http://anilist.co/anime/1"));
    assert!(!anilist.matches("https://anilist.co/manga/1"));
    assert!(anilist.matches("https://user@anilist.co:443/anime/2"));

    assert!(MatchPattern::parse("<all_urls>")
        .unwrap()
        .matches("https://a.b/"));
    assert!(MatchPattern::parse("https://*evil.com/*").is_err());
    assert!(MatchPattern::parse("https://x.com").is_err());
}

#[test]
fn install_enable_inject_uninstall() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    write_mod(&src, SAKURA);

    let mut mods = ModManager::open(tmp.path().join("mods")).unwrap();
    let manifest = mods.install(&src).unwrap();
    assert_eq!(manifest.id, "pikaswelt.sakura-night");

    // Neu installierte Mods sind aus, bis der Nutzer zustimmt.
    assert!(mods.active(false).unwrap().is_empty());
    mods.set_enabled("pikaswelt.sakura-night", true).unwrap();

    let url = "https://www.youtube.com/watch?v=abc";
    let inj = mods.injections_for(url, false).unwrap();
    assert_eq!(inj.len(), 1);
    assert!(inj[0].css[0].ends_with("sites/youtube.css"));
    assert!(inj[0].css[0].exists());
    assert_eq!(inj[0].run_at, "document_idle");
    assert!(mods
        .injections_for("https://anilist.co/", false)
        .unwrap()
        .is_empty());

    // Inkognito nur mit Erlaubnis.
    assert!(mods.injections_for(url, true).unwrap().is_empty());
    mods.set_allow_in_private("pikaswelt.sakura-night", true)
        .unwrap();
    assert_eq!(mods.injections_for(url, true).unwrap().len(), 1);

    // Zustand übersteht einen Neustart.
    let mods2 = ModManager::open(tmp.path().join("mods")).unwrap();
    assert_eq!(mods2.active(true).unwrap().len(), 1);

    mods.uninstall("pikaswelt.sakura-night").unwrap();
    assert!(mods.list().unwrap().is_empty());
    assert!(matches!(
        mods.set_enabled("pikaswelt.sakura-night", true),
        Err(ModError::NotInstalled(_))
    ));
}

#[test]
fn install_rejects_missing_files_and_traversal_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    write_mod(&src, SAKURA);
    fs::remove_file(src.join("theme.css")).unwrap();

    let mut mods = ModManager::open(tmp.path().join("mods")).unwrap();
    assert!(matches!(mods.install(&src), Err(ModError::Invalid(_))));
    assert!(matches!(
        mods.uninstall("../src"),
        Err(ModError::NotInstalled(_))
    ));
    assert!(src.exists(), "fremde Ordner dürfen nie gelöscht werden");
}

#[test]
fn reinstall_updates_files_and_keeps_state() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    write_mod(&src, SAKURA);
    let mut mods = ModManager::open(tmp.path().join("mods")).unwrap();
    mods.install(&src).unwrap();
    mods.set_enabled("pikaswelt.sakura-night", true).unwrap();

    write_mod(&src, &SAKURA.replace("1.0.0", "1.1.0"));
    mods.install(&src).unwrap();
    let list = mods.list().unwrap();
    assert_eq!(list[0].manifest.version, "1.1.0");
    assert!(list[0].state.enabled);
}
