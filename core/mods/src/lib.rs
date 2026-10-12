//! Mod-Loader für Shinoho (`docs/MODDING.md`).
//!
//! Installiert Mods aus einem Ordner in das Mod-Verzeichnis des Profils, merkt sich, welche
//! aktiv sind, und liefert der App, was sie laden muss: Themes, Soundpacks, Widgets und die
//! Seiten-CSS/-JS für eine URL. Ausgeführt werden Mod-Skripte nicht hier, sondern in der
//! Sandbox der Oberfläche (Worker bzw. iframe).

pub mod manifest;
pub mod matcher;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use manifest::{Manifest, Permission, PermissionNotice, API_VERSION};
pub use matcher::MatchPattern;

const MANIFEST_FILE: &str = "mod.json";
const STATE_FILE: &str = "mods-state.json";

#[derive(Debug, Error)]
pub enum ModError {
    #[error("mod.json ist kein gültiges JSON: {0}")]
    Manifest(String),
    #[error("Mod ungültig: {0}")]
    Invalid(String),
    #[error("Mod \"{0}\" ist nicht installiert")]
    NotInstalled(String),
    #[error("Dateifehler: {0}")]
    Io(#[from] std::io::Error),
}

/// Einstellungen einer installierten Mod.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModState {
    pub enabled: bool,
    /// Darf auch im Inkognito-Fenster laufen (`MODDING.md` §5).
    pub allow_in_private: bool,
}

/// Eine installierte Mod.
#[derive(Debug, Clone, PartialEq)]
pub struct InstalledMod {
    pub manifest: Manifest,
    pub dir: PathBuf,
    pub state: ModState,
}

impl InstalledMod {
    /// Absoluter Pfad einer Datei aus dem Manifest.
    pub fn path(&self, relative: &str) -> PathBuf {
        self.dir.join(relative)
    }
}

/// Was die App für eine Seite einfügen soll.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SiteInjection {
    pub mod_id: String,
    pub css: Vec<PathBuf>,
    pub js: Vec<PathBuf>,
    pub run_at: String,
}

pub struct ModManager {
    root: PathBuf,
    state: BTreeMap<String, ModState>,
}

impl ModManager {
    /// Öffnet das Mod-Verzeichnis (wird bei Bedarf angelegt).
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ModError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let state = match fs::read_to_string(root.join(STATE_FILE)) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => BTreeMap::new(),
        };
        Ok(Self { root, state })
    }

    /// Liest und prüft eine Mod in einem Ordner, ohne sie zu installieren.
    pub fn inspect(dir: &Path) -> Result<Manifest, ModError> {
        let json = fs::read_to_string(dir.join(MANIFEST_FILE))
            .map_err(|_| ModError::Invalid("mod.json fehlt".into()))?;
        let manifest = Manifest::from_json(&json)?;
        manifest.validate(Some(dir))?;
        Ok(manifest)
    }

    /// Installiert (oder aktualisiert) eine Mod aus `source`. Neue Mods starten deaktiviert,
    /// bis der Nutzer den Berechtigungen zugestimmt hat ([`ModManager::set_enabled`]).
    pub fn install(&mut self, source: &Path) -> Result<Manifest, ModError> {
        let manifest = Self::inspect(source)?;
        let target = self.root.join(&manifest.id);
        if target.exists() {
            fs::remove_dir_all(&target)?;
        }
        copy_dir(source, &target)?;
        self.state.entry(manifest.id.clone()).or_default();
        self.save()?;
        Ok(manifest)
    }

    pub fn uninstall(&mut self, id: &str) -> Result<(), ModError> {
        let dir = self.dir_of(id)?;
        fs::remove_dir_all(dir)?;
        self.state.remove(id);
        self.save()
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> Result<(), ModError> {
        self.dir_of(id)?;
        self.state.entry(id.to_string()).or_default().enabled = enabled;
        self.save()
    }

    pub fn set_allow_in_private(&mut self, id: &str, allow: bool) -> Result<(), ModError> {
        self.dir_of(id)?;
        self.state
            .entry(id.to_string())
            .or_default()
            .allow_in_private = allow;
        self.save()
    }

    /// Alle installierten Mods, nach ID sortiert. Kaputte Ordner werden übersprungen.
    pub fn list(&self) -> Result<Vec<InstalledMod>, ModError> {
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let dir = entry?.path();
            if !dir.is_dir() {
                continue;
            }
            if let Ok(manifest) = Self::inspect(&dir) {
                let state = self.state.get(&manifest.id).cloned().unwrap_or_default();
                out.push(InstalledMod {
                    manifest,
                    dir,
                    state,
                });
            }
        }
        out.sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
        Ok(out)
    }

    /// Aktive Mods für ein normales oder ein Inkognito-Fenster.
    pub fn active(&self, private_window: bool) -> Result<Vec<InstalledMod>, ModError> {
        Ok(self
            .list()?
            .into_iter()
            .filter(|m| m.state.enabled && (!private_window || m.state.allow_in_private))
            .collect())
    }

    /// Seiten-CSS/-JS aller aktiven Mods, die zu `url` passen.
    pub fn injections_for(
        &self,
        url: &str,
        private_window: bool,
    ) -> Result<Vec<SiteInjection>, ModError> {
        let mut out = Vec::new();
        for m in self.active(private_window)? {
            for site in &m.manifest.sites {
                let hit = site
                    .r#match
                    .iter()
                    .filter_map(|p| MatchPattern::parse(p).ok())
                    .any(|p| p.matches(url));
                if hit {
                    out.push(SiteInjection {
                        mod_id: m.manifest.id.clone(),
                        css: site.css.iter().map(|f| m.path(f)).collect(),
                        js: site.js.iter().map(|f| m.path(f)).collect(),
                        run_at: site
                            .run_at
                            .clone()
                            .unwrap_or_else(|| "document_idle".to_string()),
                    });
                }
            }
        }
        Ok(out)
    }

    fn dir_of(&self, id: &str) -> Result<PathBuf, ModError> {
        let dir = self.root.join(id);
        if id.contains(['/', '\\']) || id.starts_with('.') || !dir.join(MANIFEST_FILE).exists() {
            return Err(ModError::NotInstalled(id.to_string()));
        }
        Ok(dir)
    }

    fn save(&self) -> Result<(), ModError> {
        let json = serde_json::to_string_pretty(&self.state)
            .map_err(|e| ModError::Manifest(e.to_string()))?;
        fs::write(self.root.join(STATE_FILE), json)?;
        Ok(())
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), ModError> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        }
        // Symlinks werden bewusst nicht übernommen.
    }
    Ok(())
}
