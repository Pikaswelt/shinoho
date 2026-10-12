//! Liste der empfohlenen VPN-Anbieter (`providers.json`).
//!
//! Die Liste wird mit Shinoho ausgeliefert. Später kann eine neuere Fassung
//! nachgeladen werden, ohne dass ein neues Release nötig ist; darum wird sie
//! wie fremde Daten geprüft.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Die Fassung, die beim Bauen eingebettet wird.
const BUNDLED: &str = include_str!("../providers.json");

/// Höchste Schema-Version, die diese Shinoho-Version versteht.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetupKind {
    /// Nutzer lädt eine WireGuard-`.conf` beim Anbieter herunter und importiert sie.
    WireguardConfig,
    /// Nur über die eigene App des Anbieters nutzbar, nicht in Shinoho importierbar.
    AppOnly,
    /// Noch nicht geklärt, wie der Anbieter eingebunden wird.
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub website: String,
    pub recommended: bool,
    /// `None` = unbegrenzt.
    pub data_limit_gb_per_month: Option<u32>,
    /// `false`, wenn der Dienst nur verschlüsselt, aber den Standort nicht verbirgt.
    pub anonymizes: bool,
    pub setup: SetupKind,
    pub setup_hint: String,
    pub strengths: Vec<String>,
    pub limits: Vec<String>,
    /// Konditionen noch nicht bestätigt; die UI zeigt einen Hinweis an.
    pub needs_review: bool,
    /// Belege für die Angaben (nur `https://`).
    #[serde(default)]
    pub sources: Vec<String>,
}

impl Provider {
    /// Lässt sich eine Konfiguration dieses Anbieters in Shinoho importieren?
    pub fn importable(&self) -> bool {
        self.setup == SetupKind::WireguardConfig
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderList {
    pub schema_version: u32,
    pub last_reviewed: String,
    #[serde(default)]
    pub note: String,
    pub providers: Vec<Provider>,
}

impl ProviderList {
    /// Empfohlene Anbieter zuerst, danach alphabetisch.
    pub fn sorted_for_display(&self) -> Vec<&Provider> {
        let mut list: Vec<&Provider> = self.providers.iter().collect();
        list.sort_by(|a, b| {
            b.recommended
                .cmp(&a.recommended)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        list
    }

    /// Anbieter, deren Konfiguration Shinoho direkt importieren kann (für den Assistenten).
    pub fn importable(&self) -> impl Iterator<Item = &Provider> {
        self.providers.iter().filter(|p| p.importable())
    }

    pub fn get(&self, id: &str) -> Option<&Provider> {
        self.providers.iter().find(|p| p.id == id)
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("providers.json ist kein gültiges JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Schema-Version {0} wird nicht unterstützt (höchstens {SCHEMA_VERSION})")]
    UnsupportedSchema(u32),
    #[error("Anbieter-ID {0:?} ist doppelt")]
    DuplicateId(String),
    #[error("Anbieter-ID {0:?} ist ungültig (nur a-z, 0-9 und -)")]
    InvalidId(String),
    #[error("Anbieter {0:?}: Website muss mit https:// beginnen")]
    InsecureWebsite(String),
    #[error("Anbieter {0:?}: Quellen müssen mit https:// beginnen")]
    InsecureSource(String),
    #[error("Anbieter {0:?}: Name fehlt")]
    MissingName(String),
}

/// Liest und prüft eine Anbieterliste.
pub fn parse_providers(json: &str) -> Result<ProviderList, ProviderError> {
    let list: ProviderList = serde_json::from_str(json)?;
    if list.schema_version == 0 || list.schema_version > SCHEMA_VERSION {
        return Err(ProviderError::UnsupportedSchema(list.schema_version));
    }
    let mut seen = HashSet::new();
    for p in &list.providers {
        let id_ok = !p.id.is_empty()
            && p.id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !id_ok {
            return Err(ProviderError::InvalidId(p.id.clone()));
        }
        if !seen.insert(p.id.as_str()) {
            return Err(ProviderError::DuplicateId(p.id.clone()));
        }
        if p.name.trim().is_empty() {
            return Err(ProviderError::MissingName(p.id.clone()));
        }
        if !p.website.starts_with("https://") {
            return Err(ProviderError::InsecureWebsite(p.id.clone()));
        }
        if p.sources.iter().any(|s| !s.starts_with("https://")) {
            return Err(ProviderError::InsecureSource(p.id.clone()));
        }
    }
    Ok(list)
}

/// Die mit Shinoho ausgelieferte Anbieterliste.
pub fn load_bundled_providers() -> ProviderList {
    parse_providers(BUNDLED).expect("eingebettete providers.json ist gültig (siehe Tests)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_list_is_valid() {
        let list = parse_providers(BUNDLED).unwrap();
        assert!(!list.providers.is_empty());
        assert!(list.providers.iter().any(|p| p.recommended));
    }

    #[test]
    fn recommended_first() {
        let list = load_bundled_providers();
        let sorted = list.sorted_for_display();
        assert!(sorted[0].recommended);
        assert!(sorted
            .iter()
            .skip_while(|p| p.recommended)
            .all(|p| !p.recommended));
        assert_eq!(
            list.get("proton-free").unwrap().data_limit_gb_per_month,
            None
        );
    }

    #[test]
    fn bundled_list_has_sources_and_import_targets() {
        let list = load_bundled_providers();
        for p in &list.providers {
            assert!(!p.sources.is_empty(), "{} hat keine Quelle", p.id);
        }
        let ids: Vec<&str> = list.importable().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["proton-free"]);
        assert_eq!(
            list.get("windscribe-free").unwrap().setup,
            SetupKind::AppOnly
        );
    }

    fn one(id: &str, website: &str) -> String {
        format!(
            r#"{{"schemaVersion":1,"lastReviewed":"2026-10-12","providers":[{{
              "id":"{id}","name":"X","website":"{website}","recommended":false,
              "dataLimitGbPerMonth":5,"anonymizes":true,"setup":"unknown","setupHint":"",
              "strengths":[],"limits":[],"needsReview":true}}]}}"#
        )
    }

    #[test]
    fn rejects_bad_entries() {
        assert!(parse_providers(&one("ok-1", "https://a.example")).is_ok());
        assert!(matches!(
            parse_providers(&one("Böse", "https://a.example")),
            Err(ProviderError::InvalidId(_))
        ));
        assert!(matches!(
            parse_providers(&one("x", "http://a.example")),
            Err(ProviderError::InsecureWebsite(_))
        ));
        let bad_source = one("x", "https://a.example").replace(
            "\"needsReview\":true",
            "\"needsReview\":true,\"sources\":[\"http://a.example\"]",
        );
        assert!(matches!(
            parse_providers(&bad_source),
            Err(ProviderError::InsecureSource(_))
        ));
        assert!(matches!(
            parse_providers(&one("x", "javascript:alert(1)")),
            Err(ProviderError::InsecureWebsite(_))
        ));
        let dup = one("x", "https://a.example").replace(
            "]}",
            r#",{"id":"x","name":"Y","website":"https://b.example","recommended":false,
               "dataLimitGbPerMonth":null,"anonymizes":true,"setup":"unknown","setupHint":"",
               "strengths":[],"limits":[],"needsReview":true}]}"#,
        );
        assert!(matches!(
            parse_providers(&dup),
            Err(ProviderError::DuplicateId(_))
        ));
        let future =
            one("x", "https://a.example").replace("\"schemaVersion\":1", "\"schemaVersion\":2");
        assert!(matches!(
            parse_providers(&future),
            Err(ProviderError::UnsupportedSchema(2))
        ));
    }
}
