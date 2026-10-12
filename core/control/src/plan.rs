use std::cmp::Reverse;
use std::collections::HashSet;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::Limits;

/// Messwerte eines Tabs zu einem Zeitpunkt (geliefert von T-202).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabSnapshot {
    pub id: u64,
    pub ram_mb: u32,
    /// Prozent eines CPU-Kerns.
    pub cpu_percent: u32,
    /// Zeit seit der letzten Nutzung.
    pub idle: Duration,
    /// Der Tab, den man gerade sieht.
    pub focused: bool,
    pub pinned: bool,
    /// Spielt gerade Ton ab (Musik, Video).
    pub audible: bool,
    /// Nutzt Kamera, Mikrofon oder Bildschirmfreigabe.
    pub capturing: bool,
    pub asleep: bool,
}

impl TabSnapshot {
    /// Diese Tabs werden nie schlafen gelegt oder gedrosselt.
    fn protected(&self) -> bool {
        self.focused || self.audible || self.capturing
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "tabId", rename_all = "kebab-case")]
pub enum Action {
    /// Tab einfrieren und Speicher freigeben (lädt beim Anklicken neu).
    Sleep(u64),
    /// Tab läuft weiter, aber mit gedrosselten Timern und Animationen.
    Throttle(u64),
}

/// Entscheidet, welche Tabs schlafen gelegt bzw. gedrosselt werden.
///
/// Reihenfolge:
/// 1. Tabs, die länger als `sleep_after` ungenutzt sind (außer angepinnte).
/// 2. Ist das RAM-Limit trotzdem überschritten: weitere Tabs, angepinnte zuletzt,
///    sonst die am längsten ungenutzten und dann die größten zuerst.
/// 3. Drosseln: alle Hintergrund-Tabs, wenn `throttle_background` an ist; sonst nur
///    die CPU-hungrigsten, bis das CPU-Limit eingehalten wird.
///
/// Fokussierte, tönende und aufnehmende Tabs werden nie angefasst.
pub fn plan(limits: &Limits, tabs: &[TabSnapshot]) -> Vec<Action> {
    let mut actions = Vec::new();
    let mut sleeping: HashSet<u64> = HashSet::new();

    let candidates: Vec<&TabSnapshot> = tabs
        .iter()
        .filter(|t| !t.asleep && !t.protected())
        .collect();

    if let Some(after) = limits.sleep_after {
        for t in candidates.iter().filter(|t| !t.pinned && t.idle >= after) {
            sleeping.insert(t.id);
            actions.push(Action::Sleep(t.id));
        }
    }

    if let Some(limit) = limits.ram_limit_mb {
        let mut used: u64 = tabs
            .iter()
            .filter(|t| !t.asleep && !sleeping.contains(&t.id))
            .map(|t| u64::from(t.ram_mb))
            .sum();
        let mut rest: Vec<&TabSnapshot> = candidates
            .iter()
            .copied()
            .filter(|t| !sleeping.contains(&t.id))
            .collect();
        rest.sort_by_key(|t| (t.pinned, Reverse(t.idle), Reverse(t.ram_mb), t.id));
        for t in rest {
            if used <= u64::from(limit) {
                break;
            }
            used -= u64::from(t.ram_mb);
            sleeping.insert(t.id);
            actions.push(Action::Sleep(t.id));
        }
    }

    let mut background: Vec<&TabSnapshot> = candidates
        .into_iter()
        .filter(|t| !sleeping.contains(&t.id))
        .collect();

    if limits.throttle_background {
        actions.extend(background.iter().map(|t| Action::Throttle(t.id)));
    } else if let Some(limit) = limits.cpu_limit_percent {
        let mut used: u64 = tabs
            .iter()
            .filter(|t| !t.asleep && !sleeping.contains(&t.id))
            .map(|t| u64::from(t.cpu_percent))
            .sum();
        background.sort_by_key(|t| (Reverse(t.cpu_percent), t.id));
        for t in background {
            if used <= u64::from(limit) || t.cpu_percent == 0 {
                break;
            }
            used -= u64::from(t.cpu_percent);
            actions.push(Action::Throttle(t.id));
        }
    }

    actions
}

/// Die `n` gierigsten wachen Tabs für den Hot-Tabs-Killer, größte zuerst.
/// 1 % CPU zählt dabei so viel wie 10 MB RAM.
pub fn hot_tabs(tabs: &[TabSnapshot], n: usize) -> Vec<&TabSnapshot> {
    let mut awake: Vec<&TabSnapshot> = tabs.iter().filter(|t| !t.asleep).collect();
    awake.sort_by_key(|t| {
        (
            Reverse(u64::from(t.ram_mb) + 10 * u64::from(t.cpu_percent)),
            t.id,
        )
    });
    awake.truncate(n);
    awake
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Preset;

    const MIN: Duration = Duration::from_secs(60);

    fn tab(id: u64, ram_mb: u32, idle_min: u32) -> TabSnapshot {
        TabSnapshot {
            id,
            ram_mb,
            cpu_percent: 0,
            idle: MIN * idle_min,
            focused: false,
            pinned: false,
            audible: false,
            capturing: false,
            asleep: false,
        }
    }

    fn no_limits() -> Limits {
        Preset::MaxPower.limits(16_384)
    }

    fn sleeps(actions: &[Action]) -> Vec<u64> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Sleep(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    fn throttles(actions: &[Action]) -> Vec<u64> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Throttle(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn max_power_does_nothing() {
        let tabs = vec![tab(1, 4000, 999), tab(2, 4000, 999)];
        assert!(plan(&no_limits(), &tabs).is_empty());
    }

    #[test]
    fn idle_tabs_go_to_sleep() {
        let limits = Limits {
            sleep_after: Some(MIN * 30),
            ..no_limits()
        };
        let mut pinned = tab(3, 100, 120);
        pinned.pinned = true;
        let tabs = vec![tab(1, 100, 31), tab(2, 100, 5), pinned];
        assert_eq!(sleeps(&plan(&limits, &tabs)), vec![1]);
    }

    #[test]
    fn protected_tabs_are_never_touched() {
        let limits = Preset::Gaming.limits(1_024);
        let mut focused = tab(1, 5000, 999);
        focused.focused = true;
        let mut music = tab(2, 5000, 999);
        music.audible = true;
        let mut call = tab(3, 5000, 999);
        call.capturing = true;
        assert!(plan(&limits, &[focused, music, call]).is_empty());
    }

    #[test]
    fn ram_limit_sleeps_oldest_then_biggest_pinned_last() {
        let limits = Limits {
            ram_limit_mb: Some(1000),
            ..no_limits()
        };
        let mut focused = tab(1, 400, 0);
        focused.focused = true;
        let mut pinned = tab(2, 900, 60);
        pinned.pinned = true;
        let tabs = vec![
            focused,
            pinned,
            tab(3, 300, 10),
            tab(4, 500, 10),
            tab(5, 200, 20),
        ];
        // 2300 MB belegt: 5 (am ältesten) → 2100, 4 (größer als 3) → 1600, 3 → 1300,
        // erst dann der angepinnte Tab 2 → 400.
        assert_eq!(sleeps(&plan(&limits, &tabs)), vec![5, 4, 3, 2]);
    }

    #[test]
    fn ram_limit_stops_when_enough_is_free() {
        let limits = Limits {
            ram_limit_mb: Some(1000),
            ..no_limits()
        };
        let tabs = vec![tab(1, 600, 1), tab(2, 600, 2)];
        assert_eq!(sleeps(&plan(&limits, &tabs)), vec![2]);
    }

    #[test]
    fn already_sleeping_tabs_do_not_count() {
        let limits = Limits {
            ram_limit_mb: Some(1000),
            ..no_limits()
        };
        let mut asleep = tab(1, 5000, 99);
        asleep.asleep = true;
        let tabs = vec![asleep, tab(2, 500, 1)];
        assert!(plan(&limits, &tabs).is_empty());
    }

    #[test]
    fn throttle_all_background_tabs() {
        let limits = Limits {
            throttle_background: true,
            sleep_after: Some(MIN * 30),
            ..no_limits()
        };
        let mut focused = tab(1, 100, 0);
        focused.focused = true;
        let tabs = vec![focused, tab(2, 100, 1), tab(3, 100, 40)];
        let actions = plan(&limits, &tabs);
        assert_eq!(sleeps(&actions), vec![3]);
        assert_eq!(throttles(&actions), vec![2]);
    }

    #[test]
    fn cpu_limit_throttles_hungriest_first() {
        let limits = Limits {
            cpu_limit_percent: Some(50),
            ..no_limits()
        };
        let mut focused = tab(1, 100, 0);
        focused.focused = true;
        focused.cpu_percent = 30;
        let mut a = tab(2, 100, 1);
        a.cpu_percent = 10;
        let mut b = tab(3, 100, 1);
        b.cpu_percent = 40;
        let mut c = tab(4, 100, 1);
        c.cpu_percent = 20;
        // 100 % → 3 drosseln (60 %) → 4 drosseln (40 %) → fertig
        assert_eq!(throttles(&plan(&limits, &[focused, a, b, c])), vec![3, 4]);
    }

    #[test]
    fn hot_tabs_ranks_by_ram_and_cpu() {
        let mut busy = tab(1, 100, 0);
        busy.cpu_percent = 80; // 100 + 800
        let big = tab(2, 700, 0);
        let mut asleep = tab(3, 9000, 0);
        asleep.asleep = true;
        let small = tab(4, 50, 0);
        let tabs = vec![busy, big, asleep, small];
        let ids: Vec<u64> = hot_tabs(&tabs, 2).iter().map(|t| t.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn action_json_shape() {
        assert_eq!(
            serde_json::to_string(&Action::Sleep(7)).unwrap(),
            r#"{"action":"sleep","tabId":7}"#
        );
    }
}
