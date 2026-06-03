//! ODT (OpenDocument ZIP) structural assertions.

use sha2::{Digest, Sha256};
use std::io::Cursor;

/// Structural facts for `.odt` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdtStructure {
    /// ODF mimetype at ZIP start.
    pub has_mimetype: bool,
    /// `content.xml` present.
    pub has_content_xml: bool,
    /// Text found in `content.xml`.
    pub contains_text: Vec<String>,
}

fn read_content_xml(bytes: &[u8]) -> Option<String> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name("content.xml").ok()?;
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

impl OdtStructure {
    /// Analyze `.odt` bytes.
    #[must_use]
    pub fn analyze(bytes: &[u8], expected_text: &[&str]) -> Self {
        let xml = read_content_xml(bytes);
        Self {
            has_mimetype: bytes.starts_with(b"PK")
                && read_mimetype(bytes).as_deref()
                    == Some(b"application/vnd.oasis.opendocument.text"),
            has_content_xml: xml.is_some(),
            contains_text: expected_text
                .iter()
                .filter(|s| xml.as_ref().is_some_and(|x| xml_contains_text(x, s)))
                .map(|s| (*s).to_string())
                .collect(),
        }
    }

    /// Validate structure.
    pub fn validate(&self, required_text: &[&str]) -> Result<(), String> {
        if !self.has_mimetype {
            return Err("missing or invalid ODF mimetype".into());
        }
        if !self.has_content_xml {
            return Err("missing content.xml".into());
        }
        for needle in required_text {
            if !self.contains_text.iter().any(|t| t == needle) {
                return Err(format!("missing expected text in content.xml: {needle}"));
            }
        }
        Ok(())
    }
}

fn read_mimetype(bytes: &[u8]) -> Option<Vec<u8>> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    let mut file = archive.by_name("mimetype").ok()?;
    let mut buf = Vec::new();
    std::io::Read::read_to_end(&mut file, &mut buf).ok()?;
    Some(buf)
}

/// Normalize `content.xml` for hashing.
#[must_use]
pub fn normalize_odt_bytes(bytes: &[u8]) -> Vec<u8> {
    read_content_xml(bytes).map_or_else(
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

/// SHA-256 fingerprint of normalized content (hex).
#[must_use]
pub fn odt_fingerprint(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(normalize_odt_bytes(bytes)))
}

/// Assert ODT structure.
///
/// # Panics
///
/// On validation failure.
pub fn assert_odt_structure(bytes: &[u8], required_text: &[&str]) {
    let structure = OdtStructure::analyze(bytes, required_text);
    if let Err(msg) = structure.validate(required_text) {
        panic!("ODT structure assertion failed: {msg}\nstructure: {structure:?}");
    }
}

/// Snapshot file compare / update.
///
/// # Panics
///
/// On mismatch.
pub fn assert_odt_snapshot_file(bytes: &[u8], fixture_path: &std::path::Path) {
    let fp = odt_fingerprint(bytes);
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
             Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_odt",
            fixture_path.display()
        );
    });
    assert_eq!(
        fp,
        expected.trim(),
        "ODT snapshot mismatch for {}.\n\
         Run: DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter --test hello_odt",
        fixture_path.display()
    );
}
