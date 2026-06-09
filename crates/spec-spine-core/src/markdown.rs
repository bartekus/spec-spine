//! Minimal markdown helpers for the compiler.
//!
//! Extracts ATX heading text from a spec body, skipping fenced code blocks.
//! These headings populate `SpecRecord.section_headings` (anchors for
//! section-granularity authority).

/// The trimmed text of each ATX heading (`#`..`######`) in `body`, in order,
/// ignoring headings inside fenced code blocks.
pub fn section_headings(body: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let mut in_fence = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(text) = atx_heading_text(trimmed) {
            headings.push(text.to_string());
        }
    }
    headings
}

/// If `line` is an ATX heading, return its trimmed text; else `None`.
fn atx_heading_text(line: &str) -> Option<&str> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    // A heading requires whitespace after the run of '#'.
    if !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }
    Some(rest.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_headings_skips_code_fences() {
        let body = "# Title\n\ntext\n\n## Sub\n\n```\n# not a heading\n```\n\n### Deep\n";
        assert_eq!(section_headings(body), vec!["Title", "Sub", "Deep"]);
    }

    #[test]
    fn requires_space_after_hashes_and_caps_at_six() {
        assert_eq!(
            section_headings("#nospace\n####### too-deep\n"),
            Vec::<String>::new()
        );
    }
}
