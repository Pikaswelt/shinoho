//! URL-Muster wie bei Browser-Erweiterungen: `*://*.youtube.com/*`, `https://anilist.co/*`,
//! `<all_urls>`.

use crate::ModError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchPattern {
    AllUrls,
    Url {
        /// `None` = `http` oder `https`.
        scheme: Option<String>,
        /// `None` = jeder Host.
        host: Option<String>,
        /// `*.example.com` passt auch auf Subdomains.
        subdomains: bool,
        path: String,
    },
}

impl MatchPattern {
    pub fn parse(pattern: &str) -> Result<Self, ModError> {
        if pattern == "<all_urls>" {
            return Ok(MatchPattern::AllUrls);
        }
        let bad = || ModError::Invalid(format!("ungültiges URL-Muster \"{pattern}\""));
        let (scheme, rest) = pattern.split_once("://").ok_or_else(bad)?;
        let scheme = match scheme {
            "*" => None,
            "http" | "https" => Some(scheme.to_string()),
            _ => return Err(bad()),
        };
        let (host, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => return Err(bad()),
        };
        let (host, subdomains) = if host == "*" {
            (None, false)
        } else if let Some(base) = host.strip_prefix("*.") {
            (Some(base.to_ascii_lowercase()), true)
        } else if host.is_empty() || host.contains('*') {
            return Err(bad());
        } else {
            (Some(host.to_ascii_lowercase()), false)
        };
        Ok(MatchPattern::Url {
            scheme,
            host,
            subdomains,
            path: path.to_string(),
        })
    }

    pub fn matches(&self, url: &str) -> bool {
        let Some((url_scheme, rest)) = url.split_once("://") else {
            return false;
        };
        if url_scheme != "http" && url_scheme != "https" {
            return false;
        }
        let MatchPattern::Url {
            scheme,
            host,
            subdomains,
            path,
        } = self
        else {
            return true;
        };
        if scheme.as_deref().is_some_and(|s| s != url_scheme) {
            return false;
        }
        let (authority, url_path) = match rest.find(['/', '?', '#']) {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        let url_host = authority
            .rsplit_once('@')
            .map_or(authority, |(_, h)| h)
            .split(':')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let host_ok = match host {
            None => true,
            Some(h) if *subdomains => url_host == *h || url_host.ends_with(&format!(".{h}")),
            Some(h) => url_host == *h,
        };
        host_ok && glob(path, url_path)
    }
}

/// `*` passt auf beliebig viele Zeichen.
fn glob(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !text.starts_with(first) || text.len() < first.len() + last.len() || !text.ends_with(last) {
        return false;
    }
    let mut rest = &text[first.len()..text.len() - last.len()];
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }
    true
}
