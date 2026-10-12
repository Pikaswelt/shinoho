use shinoho_feeds::*;

// 2026-10-12 12:00:00 UTC
const NOW: i64 = 1_791_806_400_000;

const SAMPLE: &str = r#"[
  {"id":3810,"title":"Fireside Feelings (Steam) Giveaway","worth":"$7.99",
   "thumbnail":"https://x/t1.jpg","image":"https://x/i1.jpg",
   "open_giveaway_url":"https://www.gamerpower.com/open/fireside",
   "platforms":"PC, Steam","end_date":"2026-10-12 23:59:00","status":"Active"},
  {"id":3800,"title":"Hogwarts Legacy (Epic Games) Giveaway","worth":"$49.99",
   "thumbnail":"https://x/t2.jpg","image":"",
   "open_giveaway_url":"https://www.gamerpower.com/open/hogwarts",
   "platforms":"PC, Epic Games Store","end_date":"2026-10-16 15:00:00","status":"Active"},
  {"id":3790,"title":"Old Game (GOG) Giveaway","worth":"N/A",
   "thumbnail":"","image":"https://x/i3.jpg",
   "open_giveaway_url":"https://www.gamerpower.com/open/old",
   "platforms":"PC, GOG","end_date":"2026-10-01 00:00:00","status":"Active"},
  {"id":3780,"title":"Forever Free (Prime Gaming) Giveaway","worth":"",
   "thumbnail":"","image":"https://x/i4.jpg",
   "open_giveaway_url":"https://www.gamerpower.com/open/forever",
   "platforms":"PC, Amazon Prime","end_date":"N/A","status":"Active"},
  {"id":3770,"title":"Gone (Steam) Giveaway","worth":"$1",
   "thumbnail":"","image":"https://x/i5.jpg",
   "open_giveaway_url":"https://www.gamerpower.com/open/gone",
   "platforms":"PC, Steam","end_date":"2026-12-01 00:00:00","status":"Expired"}
]"#;

#[test]
fn parses_and_drops_expired_or_inactive() {
    let games = parse_gamerpower(SAMPLE, NOW).unwrap();
    let ids: Vec<u64> = games.iter().map(|g| g.id).collect();
    assert_eq!(ids, [3810, 3800, 3780]);

    let fireside = &games[0];
    assert_eq!(fireside.title, "Fireside Feelings");
    assert_eq!(fireside.store, Store::Steam);
    assert_eq!(fireside.worth.as_deref(), Some("$7.99"));
    assert_eq!(fireside.image, "https://x/i1.jpg");
    assert!(fireside.ends_at.is_some());

    assert_eq!(games[1].store, Store::Epic);
    assert_eq!(games[1].image, "https://x/t2.jpg", "Fallback auf thumbnail");
    assert_eq!(games[2].store, Store::Prime);
    assert_eq!(games[2].worth, None);
    assert_eq!(games[2].ends_at, None, "N/A = unbefristet");
}

#[test]
fn empty_result_object_is_no_error() {
    let json = r#"{"status":0,"status_message":"No active giveaways available at the moment."}"#;
    assert!(parse_gamerpower(json, NOW).unwrap().is_empty());
}

#[test]
fn garbage_is_an_error() {
    assert!(parse_gamerpower("<html>", NOW).is_err());
}

#[test]
fn select_sorts_by_deadline_and_filters_stores() {
    let games = parse_gamerpower(SAMPLE, NOW).unwrap();
    let all = select(games.clone(), &[], 10);
    assert_eq!(
        all.iter().map(|g| g.id).collect::<Vec<_>>(),
        [3810, 3800, 3780]
    );

    let epic_only = select(games.clone(), &[Store::Epic], 10);
    assert_eq!(epic_only.len(), 1);
    assert_eq!(epic_only[0].id, 3800);

    assert_eq!(select(games, &[], 1).len(), 1);
}

#[test]
fn cache_goes_stale_and_hides_expired_games() {
    let cache = FeedCache::new(parse_gamerpower(SAMPLE, NOW).unwrap(), NOW);
    assert!(!cache.is_stale(NOW + 60_000));
    assert!(cache.is_stale(NOW + CACHE_TTL_MS));

    let next_day = NOW + 24 * 60 * 60 * 1000;
    let ids: Vec<u64> = cache.current(next_day).iter().map(|g| g.id).collect();
    assert_eq!(ids, [3800, 3780]);

    let json = serde_json::to_string(&cache).unwrap();
    assert_eq!(serde_json::from_str::<FeedCache>(&json).unwrap(), cache);
}
