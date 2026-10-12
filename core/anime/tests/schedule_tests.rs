use shinoho_anime::schedule::*;
use shinoho_anime::TitleLanguage;

// 2026-10-12 00:00:00 UTC
const MONDAY: i64 = 1_791_763_200;
const HOUR: i64 = 3_600;

fn body() -> String {
    format!(
        r#"{{"data":{{"Page":{{"pageInfo":{{"hasNextPage":true}},"airingSchedules":[
  {{"episode":6,"airingAt":{a},"media":{{"id":1,"isAdult":false,"siteUrl":"https://anilist.co/anime/1",
    "title":{{"romaji":"Sousou no Frieren","english":"Frieren","native":null}},"coverImage":{{"large":"https://img/1.jpg"}}}}}},
  {{"episode":2,"airingAt":{b},"media":{{"id":5,"isAdult":false,"siteUrl":null,
    "title":{{"romaji":"Other Show","english":null,"native":null}},"coverImage":{{"large":null}}}}}},
  {{"episode":1,"airingAt":{c},"media":{{"id":9,"isAdult":true,"siteUrl":null,
    "title":{{"romaji":"Adult","english":null,"native":null}},"coverImage":{{"large":null}}}}}},
  {{"episode":12,"airingAt":{d},"media":{{"id":3,"isAdult":false,"siteUrl":null,
    "title":{{"romaji":"One Piece","english":null,"native":null}},"coverImage":{{"large":null}}}}}}
]}}}}}}"#,
        a = MONDAY + 20 * HOUR,
        b = MONDAY + 15 * HOUR,
        c = MONDAY + 16 * HOUR,
        d = MONDAY + 23 * HOUR,
    )
}

#[test]
fn request_has_time_window() {
    let req = schedule_request(MONDAY, MONDAY + 7 * 24 * HOUR, 0);
    assert_eq!(req.variables["from"], MONDAY);
    assert_eq!(req.variables["page"], 1);
    assert!(req.query.contains("airingSchedules"));
}

#[test]
fn parses_page_and_hides_adult_content() {
    let page = parse_schedule(&body(), TitleLanguage::English, &[1, 3]).unwrap();
    assert!(page.has_next_page);
    let ids: Vec<u64> = page.items.iter().map(|a| a.media_id).collect();
    assert_eq!(ids, [1, 5, 3]);
    assert_eq!(page.items[0].title, "Frieren");
    assert!(page.items[0].on_watchlist);
    assert!(!page.items[1].on_watchlist);
}

#[test]
fn groups_by_local_day_with_watchlist_first() {
    let page = parse_schedule(&body(), TitleLanguage::Romaji, &[1, 3]).unwrap();

    // UTC: alles am Montag; eigene Anime (1, 3) vor Show 5.
    let utc = group_by_day(page.items.clone(), 0);
    assert_eq!(utc.len(), 1);
    assert_eq!(utc[0].starts_at, MONDAY);
    let ids: Vec<u64> = utc[0].items.iter().map(|a| a.media_id).collect();
    assert_eq!(ids, [1, 3, 5]);

    // MESZ (UTC+2): 23:00 UTC ist schon Dienstag 01:00.
    let cest = group_by_day(page.items, 2 * HOUR);
    assert_eq!(cest.len(), 2);
    assert_eq!(cest[0].starts_at, MONDAY - 2 * HOUR);
    assert_eq!(cest[1].items[0].media_id, 3);
}
