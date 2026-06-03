//! DOCX (OOXML ZIP) structural assertions.

use sha2::{Digest, Sha256};
use std::io::Cursor;

/// Structural facts extracted from a `.docx` byte buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxStructure {
    /// ZIP magic `PK`.
    pub is_zip: bool,
    /// Whether `word/document.xml` exists in the archive.
    pub has_document_xml: bool,
    /// Substrings found in `word/document.xml` (after UTF-8 decode).
    pub contains_text: Vec<String>,
}

fn read_document_xml(bytes: &[u8]) -> Option<String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name("word/document.xml").ok()?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml).ok()?;
    Some(xml)
}

fn xml_contains_text(xml: &str, needle: &str) -> bool {
    if xml.contains(needle) {
        return true;
    }
    let escaped = quick_xml::escape::escape(needle);
    xml.contains(escaped.as_ref())
}

impl DocxStructure {
    /// Analyze raw `.docx` bytes.
    #[must_use]
    pub fn analyze(bytes: &[u8], expected_text: &[&str]) -> Self {
        let xml = read_document_xml(bytes);
        let has_document_xml = xml.is_some();
        let contains_text = expected_text
            .iter()
            .filter(|s| xml.as_ref().is_some_and(|x| xml_contains_text(x, s)))
            .map(|s| (*s).to_string())
            .collect();
        Self {
            is_zip: bytes.starts_with(b"PK"),
            has_document_xml,
            contains_text,
        }
    }

    /// Validate minimum structural requirements.
    pub fn validate(&self, required_text: &[&str]) -> Result<(), String> {
        if !self.is_zip {
            return Err("missing ZIP header (PK)".into());
        }
        if !self.has_document_xml {
            return Err("missing word/document.xml".into());
        }
        for needle in required_text {
            if !self.contains_text.iter().any(|t| t == needle) {
                return Err(format!("missing expected text in document.xml: {needle}"));
            }
        }
        Ok(())
    }
}

/// Normalize `word/document.xml` for stable hashing.
#[must_use]
pub fn normalize_docx_bytes(bytes: &[u8]) -> Vec<u8> {
    read_document_xml(bytes).map_or_else(
        || bytes.to_vec(),
        |xml| {
            xml.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes()
        },
    )
}

/// SHA-256 fingerprint of normalized document body (hex).
#[must_use]
pub fn docx_fingerprint(bytes: &[u8]) -> String {
    let normalized = normalize_docx_bytes(bytes);
    hex::encode(Sha256::digest(&normalized))
}

/// Assert DOCX bytes meet structural requirements.
///
/// # Panics
///
/// If validation fails.
pub fn assert_docx_structure(bytes: &[u8], required_text: &[&str]) {
    let structure = DocxStructure::analyze(bytes, required_text);
    if let Err(msg) = structure.validate(required_text) {
        panic!("DOCX structure assertion failed: {msg}\nstructure: {structure:?}");
    }
}

/// Compare fingerprint against a fixture file.
///
/// When `DOCRAFTER_UPDATE_SNAPSHOTS=1`, rewrites the fixture.
///
/// # Panics
///
/// On mismatch or missing fixture (without update mode).
pub fn assert_docx_snapshot_file(bytes: &[u8], fixture_path: &std::path::Path) {
    let fp = docx_fingerprint(bytes);
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
             Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_docx",
            fixture_path.display()
        );
    });
    assert_eq!(
        fp,
        expected.trim(),
        "DOCX snapshot mismatch for {}.\n\
         Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_docx",
        fixture_path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic_for_same_xml() {
        let a = docx_fingerprint(b"PK\x03\x04not really");
        let b = docx_fingerprint(b"PK\x03\x04not really");
        assert_eq!(a, b);
    }
}
