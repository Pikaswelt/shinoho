use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use adblock::engine::Engine;
use adblock::lists::{FilterSet, ParseOptions};
use adblock::request::Request;
use serde::{Deserialize, Serialize};

use crate::error::AdblockError;

pub const DEFAULT_FILTER_RULES: &[&str] = &[
    // Generic trackers & ad servers
    "||doubleclick.net^",
    "||google-analytics.com^",
    "||googlesyndication.com^",
    "||adservice.google.com^",
    "||adnxs.com^",
    "||advertising.com^",
    "||facebook.com/tr/*",
    "||scorecardresearch.com^",
    "||taboola.com^",
    "||outbrain.com^",
    "||criteo.com^",
    "||amazon-adsystem.com^",
    "||hotjar.com^",
    // Common cosmetic rules
    "##.ad-banner",
    "##.adsbox",
    "##[id^=\"google_ads_\"]",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdblockDecision {
    pub blocked: bool,
    pub matched_rule: Option<String>,
    pub redirect: Option<String>,
    pub exception: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdblockStats {
    pub total_checked: u64,
    pub total_blocked: u64,
    pub rules_count: usize,
    pub enabled: bool,
}

pub struct AdblockEngine {
    engine: RwLock<Engine>,
    rules: RwLock<Vec<String>>,
    enabled: RwLock<bool>,
    whitelist: RwLock<HashSet<String>>,
    checked_count: AtomicU64,
    blocked_count: AtomicU64,
}

impl AdblockEngine {
    pub fn new() -> Self {
        let engine = Engine::new_with_filter_set(FilterSet::new(true));
        Self {
            engine: RwLock::new(engine),
            rules: RwLock::new(Vec::new()),
            enabled: RwLock::new(true),
            whitelist: RwLock::new(HashSet::new()),
            checked_count: AtomicU64::new(0),
            blocked_count: AtomicU64::new(0),
        }
    }

    pub fn with_default_rules() -> Self {
        let engine = Self::new();
        let rules: Vec<String> = DEFAULT_FILTER_RULES.iter().map(|s| s.to_string()).collect();
        let _ = engine.add_filter_rules(&rules);
        engine
    }

    pub fn add_filter_rules(&self, rules: &[String]) -> Result<usize, AdblockError> {
        let mut all_rules = self.rules.write().map_err(|_| AdblockError::LockPoisoned)?;
        all_rules.extend(rules.iter().cloned());
        let count = rules.len();

        let mut filter_set = FilterSet::new(true);
        let full_text = all_rules.join("\n");
        filter_set.add_filter_list(full_text, ParseOptions::default());
        let new_engine = Engine::new_with_filter_set(filter_set);

        let mut engine_guard = self
            .engine
            .write()
            .map_err(|_| AdblockError::LockPoisoned)?;
        *engine_guard = new_engine;
        Ok(count)
    }

    pub fn add_filter_list(&self, content: &str) -> Result<usize, AdblockError> {
        let lines: Vec<String> = content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('!'))
            .collect();
        self.add_filter_rules(&lines)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.read().map(|g| *g).unwrap_or(true)
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut lock) = self.enabled.write() {
            *lock = enabled;
        }
    }

    pub fn whitelist_domain(&self, domain: &str) {
        if let Ok(mut lock) = self.whitelist.write() {
            lock.insert(domain.to_lowercase());
        }
    }

    pub fn unwhitelist_domain(&self, domain: &str) {
        if let Ok(mut lock) = self.whitelist.write() {
            lock.remove(&domain.to_lowercase());
        }
    }

    pub fn is_domain_whitelisted(&self, domain: &str) -> bool {
        if let Ok(lock) = self.whitelist.read() {
            let d = domain.to_lowercase();
            lock.contains(&d) || lock.iter().any(|w| d.ends_with(&format!(".{}", w)))
        } else {
            false
        }
    }

    pub fn check_network_request(
        &self,
        url: &str,
        source_url: &str,
        request_type: &str,
    ) -> AdblockDecision {
        if !self.is_enabled() {
            return AdblockDecision {
                blocked: false,
                matched_rule: None,
                redirect: None,
                exception: false,
            };
        }

        // Check if source domain is whitelisted
        if let Ok(source_parsed) = url::Url::parse(source_url) {
            if let Some(host) = source_parsed.host_str() {
                if self.is_domain_whitelisted(host) {
                    return AdblockDecision {
                        blocked: false,
                        matched_rule: None,
                        redirect: None,
                        exception: true,
                    };
                }
            }
        }

        self.checked_count.fetch_add(1, Ordering::Relaxed);

        let request = match Request::new(url, source_url, request_type, "GET") {
            Ok(req) => req,
            Err(_) => {
                return AdblockDecision {
                    blocked: false,
                    matched_rule: None,
                    redirect: None,
                    exception: false,
                };
            }
        };

        let engine = match self.engine.read() {
            Ok(e) => e,
            Err(_) => {
                return AdblockDecision {
                    blocked: false,
                    matched_rule: None,
                    redirect: None,
                    exception: false,
                };
            }
        };

        let blocker_result = engine.check_network_request(&request);
        let blocked = blocker_result.should_block();

        if blocked {
            self.blocked_count.fetch_add(1, Ordering::Relaxed);
        }

        AdblockDecision {
            blocked,
            matched_rule: blocker_result
                .filter
                .as_ref()
                .and_then(|f| f.raw_line.clone()),
            redirect: blocker_result.redirect,
            exception: blocker_result.exception.is_some(),
        }
    }

    pub fn hidden_elements_css(&self, url: &str) -> Option<String> {
        self.hidden_elements_css_with_classes(
            url,
            &["ad-banner", "adsbox"],
            &["google_ads_frame", "banner_ad"],
        )
    }

    pub fn hidden_elements_css_with_classes(
        &self,
        url: &str,
        classes: &[&str],
        ids: &[&str],
    ) -> Option<String> {
        if !self.is_enabled() {
            return None;
        }

        if let Ok(source_parsed) = url::Url::parse(url) {
            if let Some(host) = source_parsed.host_str() {
                if self.is_domain_whitelisted(host) {
                    return None;
                }
            }
        }

        let engine = self.engine.read().ok()?;
        let resources = engine.url_cosmetic_resources(url);
        let mut selectors: Vec<String> = resources.hide_selectors.into_iter().collect();

        if !resources.generichide {
            let generic = engine.hidden_class_id_selectors(
                classes.iter().copied(),
                ids.iter().copied(),
                &resources.exceptions,
            );
            selectors.extend(generic);
        }

        selectors.sort();
        selectors.dedup();

        if selectors.is_empty() {
            None
        } else {
            Some(format!(
                "{} {{ display: none !important; }}",
                selectors.join(", ")
            ))
        }
    }

    pub fn stats(&self) -> AdblockStats {
        let rules_count = self.rules.read().map(|r| r.len()).unwrap_or(0);
        AdblockStats {
            total_checked: self.checked_count.load(Ordering::Relaxed),
            total_blocked: self.blocked_count.load(Ordering::Relaxed),
            rules_count,
            enabled: self.is_enabled(),
        }
    }
}

impl Default for AdblockEngine {
    fn default() -> Self {
        Self::with_default_rules()
    }
}
