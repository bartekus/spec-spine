//! Deterministic content hashing.
//!
//! Ported from OAP `codebase-indexer/src/hash.rs`: SHA-256 over
//! `<repo-relative-POSIX-path>\0<normalized-content>` pieces, **sorted by path**
//! before hashing. Normalization (strip UTF-8 BOM, `\r\n`→`\n`, `\r`→`\n`) makes
//! the hash invariant across platforms and line-ending styles.

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

/// Normalize text for hashing: strip a leading UTF-8 BOM, then fold CRLF and CR
/// to LF.
pub fn normalize(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    text.replace("\r\n", "\n").replace('\r', "\n")
}

/// SHA-256 (lowercase hex) over `(path, raw_content)` pieces. The pieces are
/// sorted by path and each content is normalized before hashing, so the result
/// is a pure function of the input set regardless of iteration or platform.
pub fn content_hash(mut pieces: Vec<(String, String)>) -> String {
    pieces.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha256::new();
    for (path, content) in &pieces {
        hasher.update(path.as_bytes());
        hasher.update([0u8]);
        hasher.update(normalize(content).as_bytes());
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_folds_line_endings_and_bom() {
        assert_eq!(normalize("\u{feff}a\r\nb\rc"), "a\nb\nc");
    }

    #[test]
    fn hash_is_order_independent_and_stable() {
        let a = content_hash(vec![
            ("b.md".into(), "x".into()),
            ("a.md".into(), "y".into()),
        ]);
        let b = content_hash(vec![
            ("a.md".into(), "y".into()),
            ("b.md".into(), "x".into()),
        ]);
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn crlf_does_not_change_hash() {
        let unix = content_hash(vec![("a.md".into(), "line1\nline2\n".into())]);
        let win = content_hash(vec![("a.md".into(), "line1\r\nline2\r\n".into())]);
        assert_eq!(unix, win);
    }
}
