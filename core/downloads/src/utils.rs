use std::path::{Path, PathBuf};

/// Extracts and sanitizes a reasonable filename from a download URL.
pub fn filename_from_url(url_str: &str) -> String {
    let filename = if let Ok(parsed) = url::Url::parse(url_str) {
        let path = parsed.path();
        path.split('/')
            .rfind(|s| !s.is_empty())
            .map(|s| s.to_string())
    } else {
        None
    };

    let raw_name = filename.unwrap_or_else(|| "download.bin".to_string());
    sanitize_filename(&raw_name)
}

/// Sanitizes a filename by removing or replacing invalid filesystem characters.
pub fn sanitize_filename(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "download.bin".to_string();
    }

    // Strip URL encoding if any (e.g. %20 -> space)
    let decoded = urlencoding_decode(trimmed);

    let sanitized: String = decoded
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other if other.is_control() => '_',
            other => other,
        })
        .collect();

    let clean = sanitized.trim_matches(['.', ' ']).to_string();
    if clean.is_empty() {
        "download.bin".to_string()
    } else {
        clean
    }
}

/// Simple percent-decoding helper without heavy extra dependencies.
fn urlencoding_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let input_bytes = input.as_bytes();
    let mut i = 0;
    while i < input_bytes.len() {
        if input_bytes[i] == b'%'
            && i + 2 < input_bytes.len()
            && let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&input_bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            )
        {
            bytes.push(val);
            i += 3;
            continue;
        }
        bytes.push(input_bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Resolves a non-colliding file path in `dir`.
/// If `file.ext` exists, returns `file (1).ext`, `file (2).ext`, etc.
pub fn resolve_unique_path(dir: &Path, filename: &str) -> PathBuf {
    let initial_path = dir.join(filename);
    if !initial_path.exists() {
        return initial_path;
    }

    let p = Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("download");
    let ext = p.extension().and_then(|s| s.to_str());

    let mut counter = 1;
    loop {
        let new_name = match ext {
            Some(e) => format!("{stem} ({counter}).{e}"),
            None => format!("{stem} ({counter})"),
        };
        let candidate = dir.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_from_url() {
        assert_eq!(
            filename_from_url("https://example.com/downloads/setup.exe"),
            "setup.exe"
        );
        assert_eq!(
            filename_from_url("https://example.com/path/to/archive%20v1.zip?token=123"),
            "archive v1.zip"
        );
        assert_eq!(filename_from_url("https://example.com/"), "download.bin");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("bad:name*?.txt"), "bad_name__.txt");
        assert_eq!(sanitize_filename("..."), "download.bin");
    }

    #[test]
    fn test_resolve_unique_path() {
        let temp = tempfile::tempdir().unwrap();
        let path1 = temp.path().join("test.txt");
        std::fs::write(&path1, b"hello").unwrap();

        let resolved = resolve_unique_path(temp.path(), "test.txt");
        assert_eq!(resolved, temp.path().join("test (1).txt"));
    }
}
