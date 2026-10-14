//! Musterabgleich (URL- & Domain-Matching) fuer Userstyles.

use url::Url;

/// Prueft, ob eine gegebene URL mit einem Muster uebereinstimmt.
pub fn matches_pattern(pattern: &str, url_str: &str) -> bool {
    let pattern = pattern.trim();
    let url_str = url_str.trim();

    if pattern.is_empty() || url_str.is_empty() {
        return false;
    }

    // 1. Spezieller Wildcard fuer alle URLs
    if pattern == "<all_urls>" || pattern == "*" {
        return url_str.starts_with("http://")
            || url_str.starts_with("https://")
            || url_str.starts_with("file://");
    }

    // URL parsen (falls moeglich)
    if let Ok(parsed_url) = Url::parse(url_str) {
        let actual_scheme = parsed_url.scheme();
        let actual_host = parsed_url.host_str();
        let actual_path = parsed_url.path();

        // 2. Standard WebExtension Match Pattern (<scheme>://<host>/<path>)
        if let Some((scheme_pat, rest)) = pattern.split_once("://") {
            if let Some((host_pat, path_pat)) = rest.split_once('/') {
                if match_scheme(scheme_pat, actual_scheme)
                    && actual_host.is_some_and(|h| match_host(host_pat, h))
                    && match_path(path_pat, actual_path)
                {
                    return true;
                }
            } else if match_scheme(scheme_pat, actual_scheme)
                && actual_host.is_some_and(|h| match_host(rest, h))
            {
                return true;
            }
        }

        // 3. Reines Domain-Matching (z.B. "youtube.com" oder "*.youtube.com")
        if !pattern.contains("://")
            && !pattern.contains('/')
            && actual_host.is_some_and(|h| match_host(pattern, h))
        {
            return true;
        }
    }

    // 4. Glob-artiger Fallback (z.B. "*example.com*")
    glob_match(pattern, url_str)
}

/// Prueft das URL-Schema.
fn match_scheme(pattern: &str, actual: &str) -> bool {
    if pattern == "*" {
        actual == "http" || actual == "https"
    } else {
        pattern.eq_ignore_ascii_case(actual)
    }
}

/// Prueft den Host / die Domain.
fn match_host(pattern: &str, actual_host: &str) -> bool {
    let pat = pattern.to_ascii_lowercase();
    let host = actual_host.to_ascii_lowercase();

    if pat == "*" {
        return true;
    }

    if let Some(suffix) = pat.strip_prefix("*.") {
        // "*.example.com" matcht sowohl "example.com" als auch "sub.example.com"
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else if let Some(suffix) = pat.strip_prefix('.') {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host == pat || host.ends_with(&format!(".{pat}"))
    }
}

/// Prueft den Pfadteil (Pfad beginnt ohne fuehrenden Slash).
fn match_path(pattern_path: &str, actual_path: &str) -> bool {
    // Normalisierung: Pfad beginnt meist mit /
    let act_path = actual_path.strip_prefix('/').unwrap_or(actual_path);
    let pat_path = pattern_path.strip_prefix('/').unwrap_or(pattern_path);

    if pat_path == "*" || pat_path.is_empty() {
        return true;
    }

    glob_match(pat_path, act_path)
}

/// Einfaches Wildcard/Glob-Matching mit '*' und '?'.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p_bytes = pattern.as_bytes();
    let t_bytes = text.as_bytes();

    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t_bytes.len() {
        if p_idx < p_bytes.len() && (p_bytes[p_idx] == b'?' || p_bytes[p_idx] == t_bytes[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p_bytes.len() && p_bytes[p_idx] == b'*' {
            star_idx = Some(p_idx);
            p_idx += 1;
            match_idx = t_idx;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_bytes.len() && p_bytes[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == p_bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webextension_match_patterns() {
        assert!(matches_pattern(
            "*://*.youtube.com/*",
            "https://www.youtube.com/watch?v=123"
        ));
        assert!(matches_pattern(
            "*://*.youtube.com/*",
            "http://youtube.com/"
        ));
        assert!(!matches_pattern(
            "*://*.youtube.com/*",
            "https://vimeo.com/123"
        ));

        assert!(matches_pattern(
            "https://github.com/*",
            "https://github.com/Pikaswelt/shinoho"
        ));
        assert!(!matches_pattern(
            "http://github.com/*",
            "https://github.com/Pikaswelt/shinoho"
        ));
    }

    #[test]
    fn test_domain_matching() {
        assert!(matches_pattern(
            "crunchyroll.com",
            "https://www.crunchyroll.com/de/series"
        ));
        assert!(matches_pattern(
            "*.crunchyroll.com",
            "https://beta.crunchyroll.com/"
        ));
        assert!(!matches_pattern(
            "crunchyroll.com",
            "https://other-site.com/"
        ));
    }

    #[test]
    fn test_all_urls() {
        assert!(matches_pattern("<all_urls>", "https://example.com/test"));
        assert!(matches_pattern("<all_urls>", "http://localhost:3000/"));
        assert!(!matches_pattern("<all_urls>", "shinoho://hub"));
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.css", "style.css"));
        assert!(glob_match("https://*/*", "https://example.org/index.html"));
        assert!(!glob_match("*.png", "style.css"));
    }
}
