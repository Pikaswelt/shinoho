//! Tab-Schlaf (T-203): verbindet die Regeln aus [`plan`](crate::plan) mit den echten Tabs.
//!
//! Der [`TabSleeper`] bekommt regelmäßig die Tabs des Tab-Managers und die gemessenen
//! Ressourcen (T-202) und liefert nur die **Änderungen**: welche Tabs jetzt einschlafen und
//! welche wieder aufwachen sollen. Das Einfrieren selbst (WebView zerstören bzw. neu laden)
//! erledigt die WebView-Schicht in `apps/desktop`.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{plan, Action, Limits, TabSnapshot};

/// Was die WebView-Schicht über einen Tab weiß.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub id: String,
    pub active: bool,
    pub pinned: bool,
    /// Millisekunden seit 1970, wann der Tab zuletzt benutzt wurde.
    pub last_accessed_at: i64,
    /// Spielt gerade Ton ab (falls bekannt).
    #[serde(default)]
    pub audible: bool,
    /// Nutzt Kamera, Mikrofon oder Bildschirmfreigabe (falls bekannt).
    #[serde(default)]
    pub capturing: bool,
}

/// Änderungen für die WebView-Schicht.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SleepChanges {
    /// Diese Tabs einfrieren (WebView freigeben, Tab-Eintrag behalten).
    pub sleep: Vec<String>,
    /// Diese Tabs wieder aufwecken (WebView neu laden).
    pub wake: Vec<String>,
    /// Gedrosselte Hintergrund-Tabs (Timer/Animationen bremsen).
    pub throttle: Vec<String>,
}

impl SleepChanges {
    pub fn is_empty(&self) -> bool {
        self.sleep.is_empty() && self.wake.is_empty() && self.throttle.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct TabSleeper {
    asleep: HashSet<String>,
    throttled: HashSet<String>,
}

impl TabSleeper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_asleep(&self, id: &str) -> bool {
        self.asleep.contains(id)
    }

    pub fn asleep(&self) -> impl Iterator<Item = &str> {
        self.asleep.iter().map(String::as_str)
    }

    /// Ein Tab wurde geöffnet/angeklickt: sofort wecken, falls er schläft.
    pub fn touch(&mut self, id: &str) -> bool {
        self.asleep.remove(id)
    }

    /// Berechnet die Änderungen.
    ///
    /// `browser_ram_mb` ist der gemessene RAM aller WebViews zusammen. Weil sich RAM nicht
    /// genau einem Tab zuordnen lässt, wird er gleichmäßig auf die wachen Tabs verteilt.
    pub fn update(
        &mut self,
        limits: &Limits,
        tabs: &[TabState],
        browser_ram_mb: u32,
        now_ms: i64,
    ) -> SleepChanges {
        let mut changes = SleepChanges::default();
        let ids: HashSet<&str> = tabs.iter().map(|t| t.id.as_str()).collect();
        self.asleep.retain(|id| ids.contains(id.as_str()));
        self.throttled.retain(|id| ids.contains(id.as_str()));

        // Aktive Tabs wachen immer auf.
        for t in tabs.iter().filter(|t| t.active) {
            if self.asleep.remove(&t.id) {
                changes.wake.push(t.id.clone());
            }
        }

        let awake = tabs.iter().filter(|t| !self.asleep.contains(&t.id)).count();
        let per_tab = if awake == 0 {
            0
        } else {
            browser_ram_mb / awake as u32
        };

        let index: HashMap<u64, &TabState> = tabs
            .iter()
            .enumerate()
            .map(|(i, t)| (i as u64, t))
            .collect();
        let snapshots: Vec<TabSnapshot> =
            tabs.iter()
                .enumerate()
                .map(|(i, t)| TabSnapshot {
                    id: i as u64,
                    ram_mb: per_tab,
                    cpu_percent: 0,
                    idle: Duration::from_millis(
                        now_ms.saturating_sub(t.last_accessed_at).max(0) as u64
                    ),
                    focused: t.active,
                    pinned: t.pinned,
                    audible: t.audible,
                    capturing: t.capturing,
                    asleep: self.asleep.contains(&t.id),
                })
                .collect();

        let mut throttled_now = HashSet::new();
        for action in plan(limits, &snapshots) {
            match action {
                Action::Sleep(i) => {
                    let id = index[&i].id.clone();
                    if self.asleep.insert(id.clone()) {
                        changes.sleep.push(id);
                    }
                }
                Action::Throttle(i) => {
                    throttled_now.insert(index[&i].id.clone());
                }
            }
        }
        changes.throttle = throttled_now.iter().cloned().collect();
        changes.throttle.sort();
        self.throttled = throttled_now;
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Preset;

    const MIN: i64 = 60_000;
    const NOW: i64 = 1_000 * MIN;

    fn tab(id: &str, idle_min: i64) -> TabState {
        TabState {
            id: id.into(),
            active: false,
            pinned: false,
            last_accessed_at: NOW - idle_min * MIN,
            audible: false,
            capturing: false,
        }
    }

    #[test]
    fn idle_tabs_sleep_once_and_wake_when_active() {
        let limits = Preset::Balanced.limits(16_384); // Schlaf nach 30 min
        let mut s = TabSleeper::new();
        let mut a = tab("a", 0);
        a.active = true;
        let tabs = vec![a.clone(), tab("b", 45), tab("c", 5)];

        let first = s.update(&limits, &tabs, 600, NOW);
        assert_eq!(first.sleep, vec!["b"]);
        assert!(s.is_asleep("b"));

        // Zweiter Durchlauf: nichts Neues zum Einschlafen.
        assert!(s.update(&limits, &tabs, 400, NOW).sleep.is_empty());

        // Nutzer klickt b an → b wird aktiv und wacht auf.
        let mut tabs2 = tabs.clone();
        tabs2[0].active = false;
        tabs2[1].active = true;
        tabs2[1].last_accessed_at = NOW;
        let woke = s.update(&limits, &tabs2, 400, NOW);
        assert_eq!(woke.wake, vec!["b"]);
        assert!(!s.is_asleep("b"));
    }

    #[test]
    fn max_power_never_sleeps() {
        let limits = Preset::MaxPower.limits(16_384);
        let mut s = TabSleeper::new();
        let changes = s.update(&limits, &[tab("a", 999), tab("b", 999)], 90_000, NOW);
        assert!(changes.is_empty());
    }

    #[test]
    fn ram_pressure_puts_more_tabs_to_sleep() {
        // 1 GB Gerät im Gaming-Modus: Budget 256 MB, 4 Tabs à 200 MB.
        let limits = Preset::Gaming.limits(1_024);
        let mut s = TabSleeper::new();
        let mut a = tab("a", 0);
        a.active = true;
        let tabs = vec![a, tab("b", 1), tab("c", 1), tab("d", 1)];
        let changes = s.update(&limits, &tabs, 800, NOW);
        assert_eq!(changes.sleep.len(), 3, "nur der aktive Tab bleibt wach");
        assert!(!s.is_asleep("a"));
    }

    #[test]
    fn closed_tabs_are_forgotten_and_audio_protected() {
        let limits = Preset::BatterySaver.limits(16_384);
        let mut s = TabSleeper::new();
        let mut music = tab("music", 60);
        music.audible = true;
        let changes = s.update(&limits, &[music.clone(), tab("old", 60)], 100, NOW);
        assert_eq!(changes.sleep, vec!["old"]);
        // "old" wird geschlossen
        s.update(&limits, &[music], 100, NOW);
        assert!(!s.is_asleep("old"));
        assert!(!s.is_asleep("music"));
    }
}
