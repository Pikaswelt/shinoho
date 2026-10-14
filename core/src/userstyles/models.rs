//! Datenmodelle fuer Seiten-CSS (Userstyles) und Injektion.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Gibt den aktuellen Unix-Zeitstempel in Millisekunden zurueck.
pub(crate) fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Zeitpunkt der CSS-Injektion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StyleRunAt {
    /// Injektion sobald das Dokument beginnt zu laden
    DocumentStart,
    /// Injektion nach dem Laden des DOMs (Standard)
    #[default]
    DocumentEnd,
    /// Injektion im Ruhezustand (document_idle)
    DocumentIdle,
}

/// Eine einzelne Userstyle-Regel fuer Webseiten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStyleRule {
    /// Eindeutige Kennung (z.B. "userstyle-1" oder "pikaswelt.dark-youtube")
    pub id: String,
    /// Anzeigename des Userstyles
    pub name: String,
    /// Liste von Match-Patterns (z.B. ["*://*.youtube.com/*", "crunchyroll.com"])
    pub match_patterns: Vec<String>,
    /// Optionale Liste von Exclude-Patterns
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    /// Der eigentliche CSS-Inhalt
    pub css: String,
    /// Ob die Regel aktiv ist
    pub enabled: bool,
    /// Injektionszeitpunkt
    pub run_at: StyleRunAt,
    /// Zugehoerige Mod-ID (falls aus Mod geladen)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_id: Option<String>,
    /// Prioritaet (hoehere Prioritaet wird spaeter injiziert / ueberschreibt andere)
    pub priority: i32,
    /// Erstellungszeitpunkt in Millisekunden
    pub created_at: i64,
    /// Letzte Aktualisierung in Millisekunden
    pub updated_at: i64,
}

impl UserStyleRule {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        match_patterns: Vec<String>,
        css: impl Into<String>,
    ) -> Self {
        let now = current_time_millis();
        Self {
            id: id.into(),
            name: name.into(),
            match_patterns,
            exclude_patterns: Vec::new(),
            css: css.into(),
            enabled: true,
            run_at: StyleRunAt::DocumentEnd,
            mod_id: None,
            priority: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Request zum Erstellen oder Registrieren eines Userstyles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserStyleRequest {
    pub id: Option<String>,
    pub name: String,
    pub match_patterns: Vec<String>,
    pub exclude_patterns: Option<Vec<String>>,
    pub css: String,
    pub enabled: Option<bool>,
    pub run_at: Option<StyleRunAt>,
    pub mod_id: Option<String>,
    pub priority: Option<i32>,
}

/// Request zum Aktualisieren eines bestehenden Userstyles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserStyleRequest {
    pub id: String,
    pub name: Option<String>,
    pub match_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub css: Option<String>,
    pub enabled: Option<bool>,
    pub run_at: Option<StyleRunAt>,
    pub priority: Option<i32>,
}

/// Request zum Aktivieren/Deaktivieren eines Userstyles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleUserStyleRequest {
    pub id: String,
    pub enabled: bool,
}

/// Request zum Loeschen eines Userstyles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveUserStyleRequest {
    pub id: String,
}

/// Request zum Abfragen der passenden Styles fuer eine URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStylesForUrlRequest {
    pub url: String,
}

/// Rueckgabe der passenden Styles fuer eine URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteStylesResponse {
    pub url: String,
    pub matched_rules: Vec<UserStyleRule>,
    pub combined_css: String,
    pub injection_script: Option<String>,
}

/// Request zum Injizieren von Styles in einen Tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectStylesRequest {
    pub tab_id: String,
    pub url: Option<String>,
}

/// Ergebnis einer Injektion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectionResult {
    pub tab_id: String,
    pub url: String,
    pub injected_rules_count: usize,
    pub matched_rule_ids: Vec<String>,
    pub css_bytes: usize,
    pub success: bool,
}
