//! PDF AcroForm text field snapshot and roundtrip.

use docrafter::pdf::{PdfReader, PdfTextField};
use docrafter::prelude::*;
use docrafter_testing::assert_pdf_snapshot_file;
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

#[test]
fn pdf_text_field_snapshot() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Form base"));
    let base = doc.to_bytes().unwrap();

    let mut reader = PdfReader::from_bytes(&base).unwrap();
    reader
        .add_text_field(1, [72.0, 72.0, 280.0, 90.0], "email", "user@example.com")
        .unwrap();
    let bytes = reader.to_bytes().unwrap();

    assert_eq!(PdfReader::from_bytes(&bytes).unwrap().page_count(), 1);
    let fields = PdfReader::from_bytes(&bytes)
        .unwrap()
        .text_fields()
        .unwrap();
    assert_eq!(fields[0].name, "email");
    assert_eq!(fields[0].value, "user@example.com");
    assert_pdf_snapshot_file(&bytes, &snapshot_path("pdf_text_field.sha256"));
}

#[test]
fn pdf_text_field_roundtrip() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("X"));
    let mut reader = PdfReader::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    reader
        .add_text_field(1, [10.0, 20.0, 100.0, 36.0], "name", "Alice")
        .unwrap();
    let bytes = reader.to_bytes().unwrap();
    let loaded = PdfReader::from_bytes(&bytes).unwrap();
    let fields = loaded.text_fields().unwrap();
    assert_eq!(
        fields,
        vec![PdfTextField {
            page: 1,
            rect: [10.0, 20.0, 100.0, 36.0],
            name: "name".into(),
            value: "Alice".into(),
        }]
    );
}
