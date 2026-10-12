use shinoho_storage::{BookmarkHtml, Storage};

const SAMPLE_NETSCAPE_HTML: &str = r#"<!DOCTYPE NETSCAPE-Bookmark-file-1>
<!-- This is an automatically generated file.
     It will be read and overwritten.
     DO NOT EDIT! -->
<META HTTP-EQUIV="Content-Type" CONTENT="text/html; charset=UTF-8">
<TITLE>Bookmarks</TITLE>
<H1>Bookmarks</H1>
<DL><p>
    <DT><H3 ADD_DATE="1600000000" LAST_MODIFIED="1600000000" PERSONAL_TOOLBAR_FOLDER="true">Bookmarks Bar</H3>
    <DL><p>
        <DT><A HREF="https://shinoho.org" ADD_DATE="1600000000" ICON="data:image/png;base64,iVBOR" TAGS="browser,fast">Shinoho &amp; Fast</A>
        <DT><H3 ADD_DATE="1600000001">Gaming &amp; Anime</H3>
        <DL><p>
            <DT><A HREF="https://store.steampowered.com" ADD_DATE="1600000002">Steam</A>
            <DT><A HREF="https://crunchyroll.com" ADD_DATE="1600000003">Crunchyroll</A>
        </DL><p>
        <DT><A HREF="https://github.com/Pikaswelt/shinoho" ADD_DATE="1600000004">GitHub Repo</A>
    </DL><p>
</DL><p>
"#;

#[test]
fn test_parse_netscape_html() {
    let items = BookmarkHtml::parse(SAMPLE_NETSCAPE_HTML);
    assert_eq!(items.len(), 1); // Bookmarks Bar folder

    let bar = &items[0];
    assert!(bar.is_folder);
    assert_eq!(bar.title, "Bookmarks Bar");
    assert_eq!(bar.children.len(), 3); // Shinoho, Gaming & Anime folder, GitHub Repo

    let b1 = &bar.children[0];
    assert!(!b1.is_folder);
    assert_eq!(b1.title, "Shinoho & Fast");
    assert_eq!(b1.url.as_deref(), Some("https://shinoho.org"));
    assert_eq!(b1.tags, vec!["browser", "fast"]);
    assert_eq!(
        b1.favicon_url.as_deref(),
        Some("data:image/png;base64,iVBOR")
    );

    let folder2 = &bar.children[1];
    assert!(folder2.is_folder);
    assert_eq!(folder2.title, "Gaming & Anime");
    assert_eq!(folder2.children.len(), 2);
    assert_eq!(folder2.children[0].title, "Steam");
    assert_eq!(folder2.children[1].title, "Crunchyroll");

    let b3 = &bar.children[2];
    assert!(!b3.is_folder);
    assert_eq!(b3.title, "GitHub Repo");
}

#[test]
fn test_import_and_export_roundtrip() {
    let storage = Storage::open_in_memory().expect("storage");

    let summary = storage
        .import_bookmarks_html(SAMPLE_NETSCAPE_HTML, Some("toolbar"))
        .expect("import");

    // We imported 2 folders ("Bookmarks Bar" and "Gaming & Anime") and 4 bookmarks
    assert_eq!(summary.folders_created, 2);
    assert_eq!(summary.bookmarks_created, 4);

    // Verify search works on imported bookmarks
    let search_results = storage.search_bookmarks("Anime").expect("search");
    assert_eq!(search_results.len(), 1);
    assert!(search_results[0].is_folder);

    let search_tag = storage.search_bookmarks("browser").expect("search tag");
    assert_eq!(search_tag.len(), 1);
    assert_eq!(search_tag[0].url.as_deref(), Some("https://shinoho.org"));

    // Export HTML back
    let exported_html = storage
        .export_bookmarks_html(Some("toolbar"))
        .expect("export");
    assert!(exported_html.contains("<!DOCTYPE NETSCAPE-Bookmark-file-1>"));
    assert!(exported_html.contains("Shinoho &amp; Fast"));
    assert!(exported_html.contains("https://shinoho.org"));
    assert!(exported_html.contains("Gaming &amp; Anime"));
    assert!(exported_html.contains("https://store.steampowered.com"));

    // Re-import the exported HTML into a clean storage
    let storage2 = Storage::open_in_memory().expect("storage2");
    let summary2 = storage2
        .import_bookmarks_html(&exported_html, Some("toolbar"))
        .expect("import2");
    assert_eq!(summary2.folders_created, summary.folders_created);
    assert_eq!(summary2.bookmarks_created, summary.bookmarks_created);
}
