//! Phase 1.5: page extract, split, rotate, metadata.

use docrafter_pdf_read::{PdfMetadata, PdfReader, Rotate};
use docrafter_pdf_write::{PageBreak, Paragraph, PdfDocument};

fn three_page_pdf() -> Vec<u8> {
    PdfDocument::new()
        .push(Paragraph::new("Page one"))
        .push(PageBreak)
        .push(Paragraph::new("Page two"))
        .push(PageBreak)
        .push(Paragraph::new("Page three"))
        .to_bytes()
        .unwrap()
}

#[test]
fn extract_pages_keeps_subset() {
    let mut reader = PdfReader::from_bytes(&three_page_pdf()).unwrap();
    reader.extract_pages(&[2]).unwrap();
    assert_eq!(reader.page_count(), 1);
    assert!(reader.extract_text().unwrap().contains("two"));
}

#[test]
fn with_pages_does_not_modify_original() {
    let reader = PdfReader::from_bytes(&three_page_pdf()).unwrap();
    let sub = reader.with_pages(&[1, 3]).unwrap();
    assert_eq!(reader.page_count(), 3);
    assert_eq!(sub.page_count(), 2);
}

#[test]
fn split_produces_one_pdf_per_page() {
    let parts = PdfReader::from_bytes(&three_page_pdf())
        .unwrap()
        .split()
        .unwrap();
    assert_eq!(parts.len(), 3);
    assert!(parts[0].extract_text().unwrap().contains("one"));
    assert!(parts[2].extract_text().unwrap().contains("three"));
}

#[test]
fn rotate_all_pages() {
    let mut reader = PdfReader::from_bytes(&three_page_pdf()).unwrap();
    reader.rotate(None, Rotate::Clockwise90).unwrap();
    let bytes = reader.to_bytes().unwrap();
    let reloaded = PdfReader::from_bytes(&bytes).unwrap();
    assert_eq!(reloaded.page_count(), 3);
}

#[test]
fn metadata_roundtrip() {
    let mut reader = PdfReader::from_bytes(&three_page_pdf()).unwrap();
    reader
        .set_metadata(&PdfMetadata {
            title: Some("Test doc".into()),
            author: Some("docrafter".into()),
            ..Default::default()
        })
        .unwrap();
    let meta = reader.metadata();
    assert_eq!(meta.title.as_deref(), Some("Test doc"));
    assert_eq!(meta.author.as_deref(), Some("docrafter"));

    let bytes = reader.to_bytes().unwrap();
    let again = PdfReader::from_bytes(&bytes).unwrap().metadata();
    assert_eq!(again.title, meta.title);
    assert_eq!(again.author, meta.author);
}
