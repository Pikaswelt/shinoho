use std::collections::HashSet;

use shinoho_achievements::*;
use shinoho_storage::Storage;

#[test]
fn first_visit_unlocks_once() {
    let s = Storage::open_in_memory().unwrap();
    let a = Achievements::new(&s);
    let fresh = a.record(Event::PageVisited, 1).unwrap();
    assert_eq!(
        fresh.iter().map(|d| d.id).collect::<Vec<_>>(),
        ["first_steps"]
    );
    assert!(a.record(Event::PageVisited, 1).unwrap().is_empty());
}

#[test]
fn counters_accumulate_until_goal() {
    let s = Storage::open_in_memory().unwrap();
    let a = Achievements::new(&s);
    assert!(a.record(Event::TabClosed, 99).unwrap().is_empty());
    let fresh = a.record(Event::TabClosed, 1).unwrap();
    assert_eq!(fresh[0].id, "tab_cleaner");

    let p = a.progress().unwrap();
    let cleaner = p.iter().find(|p| p.id == "tab_cleaner").unwrap();
    assert_eq!(cleaner.current, 100);
    assert!(cleaner.unlocked_at.is_some());
    let hoarder = p.iter().find(|p| p.id == "tab_hoarder").unwrap();
    assert_eq!(hoarder.current, 0);
    assert!(hoarder.unlocked_at.is_none());
}

#[test]
fn state_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile.db");
    {
        let s = Storage::open(&path).unwrap();
        Achievements::new(&s)
            .record(Event::ModInstalled, 1)
            .unwrap();
    }
    let s = Storage::open(&path).unwrap();
    let summary = Achievements::new(&s).summary().unwrap();
    assert_eq!(summary.xp, 50);
}

#[test]
fn levels_and_theme_rewards() {
    assert_eq!(xp_for_level(1), 0);
    assert_eq!(xp_for_level(2), 100);
    assert_eq!(xp_for_level(3), 300);
    assert_eq!(level_for_xp(0), 1);
    assert_eq!(level_for_xp(99), 1);
    assert_eq!(level_for_xp(100), 2);
    assert_eq!(level_for_xp(300), 3);

    let s = Storage::open_in_memory().unwrap();
    let a = Achievements::new(&s);
    a.record(Event::PageVisited, 1_000).unwrap(); // 10 + 100 XP
    let summary = a.summary().unwrap();
    assert_eq!(summary.xp, 110);
    assert_eq!(summary.level, 2);
    assert_eq!(summary.xp_to_next_level, 190);
    assert_eq!(summary.unlocked_themes, ["sakura-night"]);
}

#[test]
fn reset_clears_everything() {
    let s = Storage::open_in_memory().unwrap();
    let a = Achievements::new(&s);
    a.record(Event::IncognitoOpened, 1).unwrap();
    a.reset().unwrap();
    assert_eq!(a.summary().unwrap().xp, 0);
    assert_eq!(a.record(Event::IncognitoOpened, 1).unwrap()[0].id, "ninja");
}

#[test]
fn definition_ids_are_unique() {
    let ids: HashSet<_> = DEFINITIONS.iter().map(|d| d.id).collect();
    assert_eq!(ids.len(), DEFINITIONS.len());
}
