//! Das Manifest `mod.json` (siehe `docs/MODDING.md` §2) und seine Prüfung.

use std::collections::{BTreeMap, HashSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::matcher::MatchPattern;
use crate::ModError;

/// Aktuelle Version der Mod-API.
pub const API_VERSION: u32 = 0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub sounds: Option<Sounds>,
    #[serde(default)]
    pub wallpaper: Option<Wallpaper>,
    #[serde(default)]
    pub layout: Option<Value>,
    #[serde(default)]
    pub sites: Vec<SiteMod>,
    #[serde(default)]
    pub scripts: Option<Scripts>,
    #[serde(default)]
    pub widgets: Vec<Widget>,
    #[serde(default)]
    pub settings: Vec<Setting>,
    #[serde(default)]
    pub permissions: Vec<Permission>,
    #[serde(default)]
    pub hosts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sounds {
    pub pack: String,
    #[serde(default)]
    pub volume: Option<f32>,
    #[serde(default)]
    pub map: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallpaper {
    pub file: String,
    #[serde(default)]
    pub dim: Option<f32>,
    #[serde(default)]
    pub blur: Option<f32>,
    #[serde(default)]
    pub r#where: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteMod {
    pub r#match: Vec<String>,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub js: Vec<String>,
    #[serde(default)]
    pub run_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scripts {
    pub background: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub entry: String,
    pub area: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub r#type: String,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub options: Vec<Value>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

/// Berechtigungen aus `docs/MODDING.md` §4/§5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "storage")]
    Storage,
    #[serde(rename = "tabs")]
    Tabs,
    #[serde(rename = "bookmarks")]
    Bookmarks,
    #[serde(rename = "sites")]
    Sites,
    #[serde(rename = "net")]
    Net,
    #[serde(rename = "ui.theme")]
    UiTheme,
    #[serde(rename = "ui.sounds")]
    UiSounds,
    #[serde(rename = "ui.commands")]
    UiCommands,
    #[serde(rename = "ui.sidebar")]
    UiSidebar,
    #[serde(rename = "system.stats")]
    SystemStats,
    #[serde(rename = "vpn.status")]
    VpnStatus,
}

/// Eine Berechtigung so, wie sie im Installationsdialog erscheint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermissionNotice {
    pub permission: Permission,
    /// Rot markieren (`MODDING.md` §5).
    pub critical: bool,
}

const SETTING_TYPES: &[&str] = &["boolean", "number", "string", "color", "select", "file"];
const WIDGET_AREAS: &[&str] = &["hub", "sidebar"];

impl Manifest {
    pub fn from_json(json: &str) -> Result<Self, ModError> {
        serde_json::from_str(json).map_err(|e| ModError::Manifest(e.to_string()))
    }

    /// Prüft Pflichtfelder, Formate und Berechtigungen. Mit `root` wird zusätzlich geprüft,
    /// dass alle genannten Dateien im Mod-Ordner existieren.
    pub fn validate(&self, root: Option<&Path>) -> Result<(), ModError> {
        let fail = |msg: String| Err(ModError::Invalid(msg));

        if !valid_id(&self.id) {
            return fail(format!(
                "id \"{}\" muss die Form autor.mod-name haben (a-z, 0-9, -, .)",
                self.id
            ));
        }
        if self.name.trim().is_empty() || self.author.trim().is_empty() {
            return fail("name und author dürfen nicht leer sein".into());
        }
        if !valid_semver(&self.version) {
            return fail(format!(
                "version \"{}\" ist kein SemVer (1.2.3)",
                self.version
            ));
        }
        if self.api_version != API_VERSION {
            return fail(format!(
                "apiVersion {} wird nicht unterstützt (aktuell {API_VERSION})",
                self.api_version
            ));
        }

        let perms: HashSet<Permission> = self.permissions.iter().copied().collect();
        for site in &self.sites {
            if site.r#match.is_empty() {
                return fail("sites[].match darf nicht leer sein".into());
            }
            for pattern in &site.r#match {
                MatchPattern::parse(pattern)?;
            }
            if !site.js.is_empty() && !perms.contains(&Permission::Sites) {
                return fail("Seiten-JS braucht die Berechtigung \"sites\"".into());
            }
        }
        if perms.contains(&Permission::Net) && self.hosts.is_empty() {
            return fail("\"net\" braucht eine Liste \"hosts\"".into());
        }
        for host in &self.hosts {
            MatchPattern::parse(host)?;
        }

        let mut keys = HashSet::new();
        for s in &self.settings {
            if !SETTING_TYPES.contains(&s.r#type.as_str()) {
                return fail(format!(
                    "settings.{}: unbekannter Typ \"{}\"",
                    s.key, s.r#type
                ));
            }
            if s.r#type == "select" && s.options.is_empty() {
                return fail(format!("settings.{}: select braucht options", s.key));
            }
            if !keys.insert(s.key.as_str()) {
                return fail(format!("settings.{} ist doppelt", s.key));
            }
        }
        let mut widget_ids = HashSet::new();
        for w in &self.widgets {
            if !WIDGET_AREAS.contains(&w.area.as_str()) {
                return fail(format!("widgets.{}: area muss hub oder sidebar sein", w.id));
            }
            if !widget_ids.insert(w.id.as_str()) {
                return fail(format!("widgets.{} ist doppelt", w.id));
            }
        }

        for file in self.files() {
            if !safe_relative(file) {
                return fail(format!("Pfad \"{file}\" muss relativ im Mod-Ordner liegen"));
            }
            if let Some(root) = root {
                if !root.join(file).exists() {
                    return fail(format!("Datei \"{file}\" fehlt"));
                }
            }
        }
        Ok(())
    }

    /// Alle Dateien und Ordner, auf die das Manifest verweist.
    pub fn files(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        out.extend(self.icon.as_deref());
        out.extend(self.theme.as_deref());
        out.extend(self.sounds.as_ref().map(|s| s.pack.as_str()));
        out.extend(self.wallpaper.as_ref().map(|w| w.file.as_str()));
        out.extend(self.scripts.as_ref().map(|s| s.background.as_str()));
        for site in &self.sites {
            out.extend(site.css.iter().map(String::as_str));
            out.extend(site.js.iter().map(String::as_str));
        }
        out.extend(self.widgets.iter().map(|w| w.entry.as_str()));
        out
    }

    /// Berechtigungen für den Installationsdialog, kritische markiert.
    pub fn permission_notices(&self) -> Vec<PermissionNotice> {
        let all_urls = self
            .sites
            .iter()
            .flat_map(|s| &s.r#match)
            .any(|m| m == "<all_urls>" || m == "*://*/*");
        let open_net = self
            .hosts
            .iter()
            .any(|h| h == "<all_urls>" || h == "*://*/*");
        self.permissions
            .iter()
            .map(|&p| PermissionNotice {
                permission: p,
                critical: (p == Permission::Sites && all_urls)
                    || (p == Permission::Net && open_net),
            })
            .collect()
    }
}

fn valid_id(id: &str) -> bool {
    let ok_chars = id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.');
    let parts: Vec<&str> = id.split('.').collect();
    ok_chars && parts.len() >= 2 && parts.iter().all(|p| !p.is_empty())
}

fn valid_semver(v: &str) -> bool {
    let core = v.split(['-', '+']).next().unwrap_or_default();
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn safe_relative(path: &str) -> bool {
    let p = Path::new(path);
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && p.components().all(|c| matches!(c, Component::Normal(_)))
}
