//! PDF normalization and snapshot helpers.

use sha2::{Digest, Sha256};
use std::fmt;
use std::path::Path;

/// Structural facts extracted from raw PDF bytes (no full parser).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfStructure {
    /// File begins with `%PDF-`.
    pub has_header: bool,
    /// Number of `/Type /Page` markers (heuristic page count).
    pub page_markers: usize,
    /// Whether EOF marker is present.
    pub has_eof: bool,
    /// Substrings that must appear in the document body.
    pub contains_text: Vec<String>,
}

fn bytes_contain_text(haystack: &[u8], needle: &str) -> bool {
    if haystack
        .windows(needle.len())
        .any(|window| window == needle.as_bytes())
    {
        return true;
    }
    // pdf-writer encodes non-ASCII `Str` as uppercase hex inside `<...>`.
    let hex_upper: Vec<u8> = needle
        .bytes()
        .flat_map(|b| format!("{b:02X}").into_bytes())
        .collect();
    if haystack.windows(hex_upper.len()).any(|w| w == hex_upper) {
        return true;
    }
    // Identity-H CID streams and ToUnicode cmaps use UTF-16BE code units.
    let utf16be: Vec<u8> = needle
        .encode_utf16()
        .flat_map(|u| u.to_be_bytes())
        .collect();
    if haystack.windows(utf16be.len()).any(|w| w == utf16be) {
        return true;
    }
    // Embedded fonts: `/ToUnicode` maps glyph IDs to per-codepoint UTF-16 hex (`<0048> <0065>`).
    needle.chars().all(|ch| {
        let unit = format!("{:04X}", ch.encode_utf16(&mut [0; 2])[0]);
        haystack.windows(unit.len()).any(|w| w == unit.as_bytes())
    })
}

impl PdfStructure {
    /// Analyze raw PDF bytes.
    #[must_use]
    pub fn analyze(bytes: &[u8], expected_text: &[&str]) -> Self {
        let text = String::from_utf8_lossy(bytes);
        Self {
            has_header: bytes.starts_with(b"%PDF-"),
            page_markers: text.matches("/Type /Page").count(),
            has_eof: bytes.windows(5).any(|w| w == b"%%EOF"),
            contains_text: expected_text
                .iter()
                .filter(|s| bytes_contain_text(bytes, s))
                .map(|s| (*s).to_string())
                .collect(),
        }
    }

    /// Validate minimum structural requirements for Phase 0 PDFs.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message when header, EOF, page count, or text checks fail.
    pub fn validate(&self, expected_pages: usize, required_text: &[&str]) -> Result<(), String> {
        if !self.has_header {
            return Err("missing %PDF- header".into());
        }
        if !self.has_eof {
            return Err("missing %%EOF trailer".into());
        }
        if self.page_markers < expected_pages {
            return Err(format!(
                "expected at least {expected_pages} page markers, found {}",
                self.page_markers
            ));
        }
        for needle in required_text {
            if !self.contains_text.iter().any(|t| t == needle) {
                return Err(format!("missing expected text: {needle}"));
            }
        }
        Ok(())
    }
}

/// Normalize PDF bytes for stable hashing (strip comments with dates, collapse `\r\n`).
#[must_use]
pub fn normalize_pdf_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with('%') && trimmed.contains("pdf-writer") {
            continue;
        }
        out.extend_from_slice(trimmed.as_bytes());
        out.push(b'\n');
    }
    out
}

/// SHA-256 fingerprint of normalized PDF content (hex-encoded).
#[must_use]
pub fn pdf_fingerprint(bytes: &[u8]) -> String {
    let normalized = normalize_pdf_bytes(bytes);
    let digest = Sha256::digest(&normalized);
    hex::encode(digest)
}

/// Assert that PDF bytes meet structural requirements.
///
/// # Panics
///
/// If [`PdfStructure::validate`] fails.
pub fn assert_pdf_structure(bytes: &[u8], expected_pages: usize, required_text: &[&str]) {
    let structure = PdfStructure::analyze(bytes, required_text);
    if let Err(msg) = structure.validate(expected_pages, required_text) {
        panic!("PDF structure assertion failed: {msg}\nstructure: {structure:?}");
    }
}

/// Compare normalized fingerprint against an expected hex digest.
///
/// # Panics
///
/// If the fingerprint does not match `expected_fingerprint`.
pub fn assert_pdf_snapshot(bytes: &[u8], expected_fingerprint: &str) {
    let fp = pdf_fingerprint(bytes);
    assert_eq!(
        fp, expected_fingerprint,
        "PDF snapshot mismatch.\nexpected: {expected_fingerprint}\nactual:   {fp}\n\
         Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_pdf"
    );
}

/// Load fingerprint from `fixture_path` (relative to workspace root or absolute).
///
/// When `DOCRAFTER_UPDATE_SNAPSHOTS=1` is set, rewrites the fixture with the current fingerprint.
///
/// # Panics
///
/// If the fixture is missing (without update mode) or the fingerprint mismatches.
pub fn assert_pdf_snapshot_file(bytes: &[u8], fixture_path: &Path) {
    let fp = pdf_fingerprint(bytes);
    if std::env::var("DOCRAFTER_UPDATE_SNAPSHOTS").as_deref() == Ok("1") {
        if let Some(parent) = fixture_path.parent() {
            std::fs::create_dir_all(parent).expect("create snapshot dirs");
        }
        std::fs::write(fixture_path, format!("{fp}\n")).expect("write snapshot fixture");
        return;
    }
    let expected = std::fs::read_to_string(fixture_path).unwrap_or_else(|e| {
        panic!(
            "missing snapshot fixture {}: {e}\n\
             Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_pdf",
            fixture_path.display()
        );
    });
    assert_eq!(
        fp,
        expected.trim(),
        "PDF snapshot mismatch for {}.\n\
         Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_pdf",
        fixture_path.display()
    );
}

impl fmt::Display for PdfStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PdfStructure {{ header: {}, pages: {}, eof: {}, text: {:?} }}",
            self.has_header, self.page_markers, self.has_eof, self.contains_text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_is_deterministic() {
        let input = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n%%EOF\n";
        assert_eq!(normalize_pdf_bytes(input), normalize_pdf_bytes(input));
    }

    #[test]
    fn fingerprint_changes_when_content_changes() {
        let a = pdf_fingerprint(b"%PDF-1.7\nHello\n%%EOF\n");
        let b = pdf_fingerprint(b"%PDF-1.7\nWorld\n%%EOF\n");
        assert_ne!(a, b);
    }

    #[test]
    fn structure_detects_missing_header() {
        let s = PdfStructure::analyze(b"not a pdf", &[]);
        assert!(s.validate(0, &[]).is_err());
    }

    #[test]
    fn detects_hex_encoded_unicode() {
        // "При" in UTF-8 hex as emitted by pdf-writer for non-ASCII strings.
        let pdf = b"%PDF\n<D09FD180D0B8D0B9>\n%%EOF";
        let s = PdfStructure::analyze(pdf, &["При"]);
        assert_eq!(s.contains_text, vec!["При".to_string()]);
    }

    #[test]
    fn detects_to_unicode_per_codepoint_hex() {
        let pdf = b"%PDF\n<0048> <0048>\n<0065> <0065>\n%%EOF";
        let s = PdfStructure::analyze(pdf, &["He"]);
        assert_eq!(s.contains_text, vec!["He".to_string()]);
    }
}
