//! Manager fuer Userstyles und Injektionsskripte (T-303).

use crate::error::{CoreError, CoreResult};
use crate::userstyles::matcher::matches_pattern;
use crate::userstyles::models::{
    AddUserStyleRequest, InjectionResult, SiteStylesResponse, StyleRunAt, UpdateUserStyleRequest,
    UserStyleRule, current_time_millis,
};
use std::collections::HashMap;

/// Verwaltet alle registrierten Seiten-CSS-Regeln.
#[derive(Debug, Default)]
pub struct UserStylesManager {
    rules: HashMap<String, UserStyleRule>,
    id_counter: u64,
}

impl UserStylesManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fuegt einen neuen Userstyle hinzu oder registriert ihn.
    pub fn add_rule(&mut self, req: AddUserStyleRequest) -> UserStyleRule {
        let id = req.id.unwrap_or_else(|| {
            self.id_counter += 1;
            format!("userstyle-{}", self.id_counter)
        });

        let now = current_time_millis();
        let rule = UserStyleRule {
            id: id.clone(),
            name: req.name,
            match_patterns: req.match_patterns,
            exclude_patterns: req.exclude_patterns.unwrap_or_default(),
            css: req.css,
            enabled: req.enabled.unwrap_or(true),
            run_at: req.run_at.unwrap_or(StyleRunAt::DocumentEnd),
            mod_id: req.mod_id,
            priority: req.priority.unwrap_or(0),
            created_at: now,
            updated_at: now,
        };

        self.rules.insert(id, rule.clone());
        rule
    }

    /// Aktualisiert eine bestehende Userstyle-Regel.
    pub fn update_rule(&mut self, req: UpdateUserStyleRequest) -> CoreResult<UserStyleRule> {
        let rule = self.rules.get_mut(&req.id).ok_or_else(|| {
            CoreError::InvalidOperation(format!("Userstyle {} nicht gefunden", req.id))
        })?;

        if let Some(name) = req.name {
            rule.name = name;
        }
        if let Some(patterns) = req.match_patterns {
            rule.match_patterns = patterns;
        }
        if let Some(excludes) = req.exclude_patterns {
            rule.exclude_patterns = excludes;
        }
        if let Some(css) = req.css {
            rule.css = css;
        }
        if let Some(enabled) = req.enabled {
            rule.enabled = enabled;
        }
        if let Some(run_at) = req.run_at {
            rule.run_at = run_at;
        }
        if let Some(priority) = req.priority {
            rule.priority = priority;
        }

        rule.updated_at = current_time_millis();
        Ok(rule.clone())
    }

    /// Aktiviert oder deaktiviert eine Userstyle-Regel.
    pub fn toggle_rule(&mut self, id: &str, enabled: bool) -> CoreResult<UserStyleRule> {
        let rule = self
            .rules
            .get_mut(id)
            .ok_or_else(|| CoreError::InvalidOperation(format!("Userstyle {id} nicht gefunden")))?;

        rule.enabled = enabled;
        rule.updated_at = current_time_millis();
        Ok(rule.clone())
    }

    /// Entfernt eine Userstyle-Regel.
    pub fn remove_rule(&mut self, id: &str) -> bool {
        self.rules.remove(id).is_some()
    }

    /// Gibt alle registrierten Userstyles zurueck.
    pub fn get_rules(&self) -> Vec<UserStyleRule> {
        let mut list: Vec<UserStyleRule> = self.rules.values().cloned().collect();
        list.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| a.id.cmp(&b.id)));
        list
    }

    /// Gibt eine bestimmte Userstyle-Regel anhand ihrer ID zurueck.
    pub fn get_rule(&self, id: &str) -> Option<UserStyleRule> {
        self.rules.get(id).cloned()
    }

    /// Findet alle aktiven Regeln, die fuer die gegebene URL zutreffen, sortiert nach Prioritaet.
    pub fn matching_rules_for_url(&self, url: &str) -> Vec<UserStyleRule> {
        let mut matched: Vec<UserStyleRule> = self
            .rules
            .values()
            .filter(|rule| {
                if !rule.enabled {
                    return false;
                }

                // Exclude-Patterns haben Vorrang
                let is_excluded = rule
                    .exclude_patterns
                    .iter()
                    .any(|pat| matches_pattern(pat, url));
                if is_excluded {
                    return false;
                }

                // Mindestens ein Match-Pattern muss passen
                rule.match_patterns
                    .iter()
                    .any(|pat| matches_pattern(pat, url))
            })
            .cloned()
            .collect();

        // Nach Prioritaet aufsteigend sortieren (hoehere Prioritaet wird spaeter injiziert)
        matched.sort_by_key(|a| a.priority);
        matched
    }

    /// Kombiniert den CSS-Code aller passenden Regeln fuer eine URL.
    pub fn get_combined_css_for_url(&self, url: &str) -> String {
        let matched = self.matching_rules_for_url(url);
        if matched.is_empty() {
            return String::new();
        }

        let mut combined = String::new();
        for rule in &matched {
            combined.push_str(&format!(
                "/* Shinoho UserStyle: {} ({}) */\n",
                rule.name, rule.id
            ));
            combined.push_str(&rule.css);
            combined.push_str("\n\n");
        }
        combined
    }

    /// Erzeugt ein sicheres JavaScript-Snippet zur Injektion des Seiten-CSS.
    pub fn generate_injection_script(&self, url: &str) -> Option<String> {
        let matched = self.matching_rules_for_url(url);
        if matched.is_empty() {
            return None;
        }

        let combined_css = self.get_combined_css_for_url(url);
        let rule_ids: Vec<String> = matched.iter().map(|r| r.id.clone()).collect();
        let rule_ids_str = rule_ids.join(",");

        // CSS sicher JSON-kodieren, um Syntaxfehler im JS zu verhindern
        let css_json = serde_json::to_string(&combined_css).unwrap_or_else(|_| "\"\"".to_string());

        let script = format!(
            r#"(function() {{
  const styleId = 'shinoho-userstyles-root';
  let el = document.getElementById(styleId);
  if (!el) {{
    el = document.createElement('style');
    el.id = styleId;
    el.setAttribute('data-shinoho-userstyles', '{rule_ids_str}');
    (document.head || document.documentElement).appendChild(el);
  }} else {{
    el.setAttribute('data-shinoho-userstyles', '{rule_ids_str}');
  }}
  el.textContent = {css_json};
}})();"#
        );

        Some(script)
    }

    /// Abfrage-Helfer fuer Tauri-Commands.
    pub fn query_styles_for_url(&self, url: &str) -> SiteStylesResponse {
        let matched_rules = self.matching_rules_for_url(url);
        let combined_css = self.get_combined_css_for_url(url);
        let injection_script = self.generate_injection_script(url);

        SiteStylesResponse {
            url: url.to_string(),
            matched_rules,
            combined_css,
            injection_script,
        }
    }

    /// Erstellt das Ergebnis einer Injektion fuer einen Tab.
    pub fn evaluate_injection(&self, tab_id: &str, url: &str) -> InjectionResult {
        let matched = self.matching_rules_for_url(url);
        let matched_rule_ids: Vec<String> = matched.iter().map(|r| r.id.clone()).collect();
        let css = self.get_combined_css_for_url(url);

        InjectionResult {
            tab_id: tab_id.to_string(),
            url: url.to_string(),
            injected_rules_count: matched.len(),
            matched_rule_ids,
            css_bytes: css.len(),
            success: true,
        }
    }
}
