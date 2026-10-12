//! Gaming-Modus (T-205): erkennt, ob gerade ein Spiel läuft, damit Shinoho
//! automatisch auf das Preset [`Preset::Gaming`](crate::Preset::Gaming) umschalten kann.
//!
//! Erkannt wird über die Prozessliste, nicht über Bildschirminhalte:
//! - Spiele aus Steam-, Epic-, GOG-, Riot-, Ubisoft-, EA- und Battle.net-Bibliotheken
//!   (anhand des Installationspfads, gilt auch für Proton/Wine unter Linux),
//! - bekannte Spiele über ihren Prozessnamen,
//! - Minecraft (Java) über die Kommandozeile,
//! - eigene Einträge des Nutzers; eigene Ausnahmen gehen immer vor.

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Das Nötigste über einen laufenden Prozess.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInfo {
    pub name: String,
    /// Voller Pfad zur ausführbaren Datei, falls lesbar.
    pub exe: String,
    pub cmd: Vec<String>,
}

/// Ordner, in denen Spielebibliotheken ihre Spiele installieren (klein, mit `/`).
const LIBRARY_DIRS: &[&str] = &[
    "/steamapps/common/",
    "/epic games/",
    "/gog galaxy/games/",
    "/gog games/",
    "/riot games/",
    "/ubisoft game launcher/games/",
    "/ea games/",
    "/battle.net/",
    "/xboxgames/",
    "/heroic/",
    "/lutris/",
];

/// Launcher und Hilfsprogramme in diesen Ordnern, die selbst kein Spiel sind.
const NOT_GAMES: &[&str] = &[
    "steam",
    "steamwebhelper",
    "steamservice",
    "epicgameslauncher",
    "epicwebhelper",
    "unrealcefsubprocess",
    "crashreportclient",
    "riotclientservices",
    "riotclientux",
    "riotclientcrashhandler",
    "galaxyclient",
    "galaxyclient helper",
    "upc",
    "uplaywebcore",
    "eadesktop",
    "battle.net",
    "agent",
    "blizzard error",
    "easyanticheat",
    "easyanticheat_eos",
    "beservice",
    "vgc",
];

/// Bekannte Spiele, die oft außerhalb der Bibliotheksordner liegen.
const KNOWN_GAMES: &[&str] = &[
    "cs2",
    "csgo",
    "valorant-win64-shipping",
    "league of legends",
    "fortniteclient-win64-shipping",
    "gta5",
    "gta5_enhanced",
    "rocketleague",
    "overwatch",
    "r5apex",
    "dota2",
    "eldenring",
    "genshinimpact",
    "starrail",
    "zenlesszonezero",
    "wutheringwaves",
    "robloxplayerbeta",
    "osu!",
    "minecraft.windows",
    "factorio",
    "terraria",
    "stardew valley",
    "hollow_knight",
    "cyberpunk2077",
    "witcher3",
    "bg3",
    "bg3_dx11",
];

fn normalize(name: &str) -> String {
    let lower = name.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// Einstellungen des Nutzers für die Erkennung.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRules {
    /// Zusätzliche Prozessnamen, die als Spiel zählen (ohne `.exe`, egal ob groß/klein).
    #[serde(default)]
    pub extra_games: Vec<String>,
    /// Prozessnamen, die nie als Spiel zählen (z. B. ein Launcher, der im Hintergrund läuft).
    #[serde(default)]
    pub ignored: Vec<String>,
}

impl GameRules {
    /// Gibt einen Anzeigenamen zurück, wenn `p` ein Spiel ist.
    pub fn match_process(&self, p: &ProcessInfo) -> Option<String> {
        let name = normalize(&p.name);
        if name.is_empty() || self.ignored.iter().any(|i| normalize(i) == name) {
            return None;
        }
        if self.extra_games.iter().any(|g| normalize(g) == name) {
            return Some(p.name.clone());
        }
        if NOT_GAMES.contains(&name.as_str()) {
            return None;
        }
        if KNOWN_GAMES.contains(&name.as_str()) {
            return Some(p.name.clone());
        }
        let exe = p.exe.replace('\\', "/").to_lowercase();
        if LIBRARY_DIRS.iter().any(|dir| exe.contains(dir)) {
            // Proton/Wine: das Spiel steckt in der Kommandozeile, nicht in `exe`.
            return Some(p.name.clone());
        }
        let in_library_cmd = p.cmd.iter().any(|arg| {
            let a = arg.replace('\\', "/").to_lowercase();
            a.ends_with(".exe") && LIBRARY_DIRS.iter().any(|dir| a.contains(dir))
        });
        if in_library_cmd {
            return Some(p.name.clone());
        }
        let minecraft =
            name.starts_with("java") && p.cmd.iter().any(|a| a.contains("net.minecraft"));
        if minecraft {
            return Some("Minecraft".to_string());
        }
        None
    }

    /// Das erste laufende Spiel in `processes`, falls eins läuft.
    pub fn detect(&self, processes: &[ProcessInfo]) -> Option<String> {
        processes.iter().find_map(|p| self.match_process(p))
    }
}

/// Änderung des Gaming-Zustands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamingEvent {
    Started(String),
    Stopped,
}

/// Beobachtet die Prozessliste. `poll()` regelmäßig aufrufen (z. B. alle 5 Sekunden).
pub struct GameWatcher {
    system: System,
    rules: GameRules,
    current: Option<String>,
}

impl GameWatcher {
    pub fn new(rules: GameRules) -> Self {
        Self {
            system: System::new(),
            rules,
            current: None,
        }
    }

    pub fn set_rules(&mut self, rules: GameRules) {
        self.rules = rules;
    }

    /// Das gerade erkannte Spiel.
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// Liest die Prozessliste neu und meldet, ob ein Spiel gestartet oder beendet wurde.
    pub fn poll(&mut self) -> Option<GamingEvent> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet),
        );
        let processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .values()
            .map(|p| ProcessInfo {
                name: p.name().to_string_lossy().into_owned(),
                exe: p
                    .exe()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                cmd: p
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect(),
            })
            .collect();
        self.update(self.rules.detect(&processes))
    }

    fn update(&mut self, found: Option<String>) -> Option<GamingEvent> {
        match (&self.current, found) {
            (None, Some(game)) => {
                self.current = Some(game.clone());
                Some(GamingEvent::Started(game))
            }
            (Some(_), None) => {
                self.current = None;
                Some(GamingEvent::Stopped)
            }
            (Some(old), Some(game)) if *old != game => {
                self.current = Some(game.clone());
                Some(GamingEvent::Started(game))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(name: &str, exe: &str) -> ProcessInfo {
        ProcessInfo {
            name: name.into(),
            exe: exe.into(),
            cmd: vec![exe.into()],
        }
    }

    #[test]
    fn steam_games_on_windows_and_linux() {
        let r = GameRules::default();
        let win = proc(
            "eldenring.exe",
            r"C:\Program Files (x86)\Steam\steamapps\common\ELDEN RING\Game\eldenring.exe",
        );
        assert_eq!(r.match_process(&win).as_deref(), Some("eldenring.exe"));
        let linux = proc(
            "hl2_linux",
            "/home/pika/.local/share/Steam/steamapps/common/Half-Life 2/hl2_linux",
        );
        assert!(r.match_process(&linux).is_some());
    }

    #[test]
    fn proton_games_via_command_line() {
        let p = ProcessInfo {
            name: "wine64-preloader".into(),
            exe: "/home/pika/.steam/steam/steamapps/common/Proton 9.0/files/bin/wine64-preloader"
                .into(),
            cmd: vec![
                "Z:\\home\\pika\\.steam\\steam\\steamapps\\common\\Hades\\x64\\Hades.exe".into(),
            ],
        };
        assert!(GameRules::default().match_process(&p).is_some());
    }

    #[test]
    fn launchers_are_not_games() {
        let r = GameRules::default();
        for p in [
            proc("steam.exe", r"C:\Program Files (x86)\Steam\steam.exe"),
            proc(
                "steamwebhelper",
                "/home/pika/.local/share/Steam/ubuntu12_64/steamwebhelper",
            ),
            proc(
                "EpicGamesLauncher.exe",
                r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe",
            ),
            proc(
                "RiotClientServices.exe",
                r"C:\Riot Games\Riot Client\RiotClientServices.exe",
            ),
            proc("firefox", "/usr/lib/firefox/firefox"),
            proc("shinoho", "/usr/bin/shinoho"),
        ] {
            assert_eq!(r.match_process(&p), None, "{} ist kein Spiel", p.name);
        }
    }

    #[test]
    fn known_names_and_minecraft() {
        let r = GameRules::default();
        assert!(r.match_process(&proc("cs2", "/opt/cs2/cs2")).is_some());
        let mc = ProcessInfo {
            name: "javaw.exe".into(),
            exe: r"C:\Program Files\Java\bin\javaw.exe".into(),
            cmd: vec!["javaw".into(), "net.minecraft.client.main.Main".into()],
        };
        assert_eq!(r.match_process(&mc).as_deref(), Some("Minecraft"));
        let other_java = ProcessInfo {
            cmd: vec!["java".into(), "-jar".into(), "server.jar".into()],
            ..mc
        };
        assert_eq!(r.match_process(&other_java), None);
    }

    #[test]
    fn user_rules_win() {
        let r = GameRules {
            extra_games: vec!["MyIndieGame.exe".into()],
            ignored: vec!["cs2".into()],
        };
        assert!(r
            .match_process(&proc("myindiegame", "/x/myindiegame"))
            .is_some());
        assert_eq!(r.match_process(&proc("cs2", "/opt/cs2/cs2")), None);
    }

    #[test]
    fn watcher_emits_only_changes() {
        let mut w = GameWatcher::new(GameRules::default());
        assert_eq!(w.update(None), None);
        assert_eq!(
            w.update(Some("cs2".into())),
            Some(GamingEvent::Started("cs2".into()))
        );
        assert_eq!(w.current(), Some("cs2"));
        assert_eq!(w.update(Some("cs2".into())), None);
        assert_eq!(
            w.update(Some("dota2".into())),
            Some(GamingEvent::Started("dota2".into()))
        );
        assert_eq!(w.update(None), Some(GamingEvent::Stopped));
        assert_eq!(w.current(), None);
    }

    #[test]
    fn real_process_list_can_be_read() {
        // Läuft gegen das echte System: darf kein Spiel finden und nicht abstürzen.
        let mut w = GameWatcher::new(GameRules::default());
        let _ = w.poll();
    }

    #[test]
    fn rules_roundtrip_as_json() {
        let r: GameRules = serde_json::from_str(r#"{"extraGames":["x"]}"#).unwrap();
        assert_eq!(r.extra_games, vec!["x"]);
        assert!(r.ignored.is_empty());
    }
}
