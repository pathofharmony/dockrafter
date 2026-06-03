//! Phase 0.2: read and merge PDFs.

use docrafter::prelude::*;
use docrafter_testing::assert_pdf_structure;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn pdf_reader_opens_generated_bytes() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Readable PDF").style(Style::new().font_size(12.0)));
    let bytes = doc.to_bytes().unwrap();

    let reader = PdfReader::from_bytes(&bytes).unwrap();
    assert_eq!(reader.page_count(), 1);
}

#[test]
fn merge_two_generated_pdfs() {
    let mut first = PdfDocument::new();
    first.push(Paragraph::new("First doc").style(Style::new().font_size(12.0)));
    let first_bytes = first.to_bytes().unwrap();

    let mut second = PdfDocument::new();
    second.push(Paragraph::new("Second doc").style(Style::new().font_size(12.0)));
    second.push(PageBreak);
    second.push(Paragraph::new("Second page").style(Style::new().font_size(12.0)));
    let second_bytes = second.to_bytes().unwrap();

    let mut merged = PdfReader::from_bytes(&first_bytes).unwrap();
    merged
        .merge(&PdfReader::from_bytes(&second_bytes).unwrap())
        .unwrap();

    assert_eq!(merged.page_count(), 3);
    let out = merged.to_bytes().unwrap();
    assert_eq!(PdfReader::from_bytes(&out).unwrap().page_count(), 3);
    assert_pdf_structure(&out, 0, &["First doc", "Second doc", "Second page"]);
}

#[test]
fn pdf_reader_from_path_fixture() {
    let path = fixture_dir().join("logo.png");
    assert!(path.exists(), "fixture logo.png must exist");

    let mut doc = PdfDocument::new();
    doc.push(Image::from_path(&path).unwrap().size(40.0, 40.0));
    let pdf_path = std::env::temp_dir().join("docrafter_merge_fixture.pdf");
    doc.save(&pdf_path).unwrap();

    let reader = PdfReader::open(&pdf_path).unwrap();
    assert_eq!(reader.page_count(), 1);
    let _ = std::fs::remove_file(pdf_path);
}
