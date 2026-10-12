use shinoho_storage::{CreateBookmarkParams, Storage};
use shinoho_sync::*;

const PASS: &str = "sakura-passwort";

fn seeded() -> Storage {
    let s = Storage::open_in_memory().unwrap();
    let folder = s
        .create_bookmark(CreateBookmarkParams {
            title: "Anime".into(),
            is_folder: true,
            ..Default::default()
        })
        .unwrap();
    s.create_bookmark(CreateBookmarkParams {
        parent_id: Some(folder.id.clone()),
        title: "AniList".into(),
        url: Some("https://anilist.co".into()),
        tags: vec!["anime".into()],
        ..Default::default()
    })
    .unwrap();
    s.set_setting("theme", "sakura").unwrap();
    s
}

#[test]
fn roundtrip_to_empty_profile() {
    let src = seeded();
    let file = export_encrypted(&src, PASS).unwrap();
    assert_eq!(&file[..4], MAGIC);

    let dst = Storage::open_in_memory().unwrap();
    let before = dst.list_all_bookmarks().unwrap().len();
    let report = import_encrypted(&dst, &file, PASS).unwrap();

    assert_eq!(report.bookmarks_orphaned, 0);
    assert_eq!(report.bookmarks_created, 2);
    assert_eq!(dst.list_all_bookmarks().unwrap().len(), before + 2);
    let hits = dst.search_bookmarks("AniList").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].tags, vec!["anime".to_string()]);
    assert_eq!(dst.get_setting("theme").unwrap().as_deref(), Some("sakura"));
}

#[test]
fn import_is_idempotent() {
    let src = seeded();
    let file = export_encrypted(&src, PASS).unwrap();
    let dst = Storage::open_in_memory().unwrap();
    import_encrypted(&dst, &file, PASS).unwrap();
    let count = dst.list_all_bookmarks().unwrap().len();

    let again = import_encrypted(&dst, &file, PASS).unwrap();
    assert_eq!(again.bookmarks_created, 0);
    assert_eq!(dst.list_all_bookmarks().unwrap().len(), count);
}

#[test]
fn newer_file_wins_older_file_is_skipped() {
    let src = seeded();
    let mut bundle = collect(&src).unwrap();
    let dst = Storage::open_in_memory().unwrap();
    apply(&dst, &bundle).unwrap();

    let item = bundle
        .bookmarks
        .iter_mut()
        .find(|b| b.title == "AniList")
        .unwrap();
    item.title = "AniList (neu)".into();
    item.updated_at += 60_000;
    let report = apply(&dst, &bundle).unwrap();
    assert_eq!(report.bookmarks_updated, 1);
    assert_eq!(dst.search_bookmarks("AniList (neu)").unwrap().len(), 1);

    let item = bundle
        .bookmarks
        .iter_mut()
        .find(|b| b.title == "AniList (neu)")
        .unwrap();
    item.title = "Veraltet".into();
    item.updated_at -= 3_600_000;
    let report = apply(&dst, &bundle).unwrap();
    assert_eq!(report.bookmarks_updated, 0);
    assert!(dst.search_bookmarks("Veraltet").unwrap().is_empty());
}

#[test]
fn children_before_parents_in_file_still_import() {
    let src = seeded();
    let mut bundle = collect(&src).unwrap();
    bundle.bookmarks.reverse();
    let dst = Storage::open_in_memory().unwrap();
    let report = apply(&dst, &bundle).unwrap();
    assert_eq!(report.bookmarks_orphaned, 0);
    assert_eq!(report.bookmarks_created, 2);
}

#[test]
fn wrong_passphrase_is_rejected() {
    let file = export_encrypted(&seeded(), PASS).unwrap();
    let err = decrypt(&file, "falsches-passwort").unwrap_err();
    assert!(matches!(err, SyncError::Decrypt));
}

#[test]
fn tampered_file_is_rejected() {
    let mut file = export_encrypted(&seeded(), PASS).unwrap();
    let last = file.len() - 1;
    file[last] ^= 0x01;
    assert!(matches!(decrypt(&file, PASS), Err(SyncError::Decrypt)));
}

#[test]
fn foreign_and_future_files_are_rejected() {
    assert!(matches!(
        decrypt(b"kein sync", PASS),
        Err(SyncError::NotASyncFile)
    ));
    let mut file = export_encrypted(&seeded(), PASS).unwrap();
    file[4] = 99;
    assert!(matches!(
        decrypt(&file, PASS),
        Err(SyncError::UnsupportedVersion(99))
    ));
}

#[test]
fn short_passphrase_is_rejected() {
    let err = export_encrypted(&seeded(), "kurz").unwrap_err();
    assert!(matches!(err, SyncError::PassphraseTooShort));
}

#[test]
fn same_input_gives_different_files() {
    let s = seeded();
    let a = export_encrypted(&s, PASS).unwrap();
    let b = export_encrypted(&s, PASS).unwrap();
    assert_ne!(a, b, "Salt und Nonce müssen zufällig sein");
}
