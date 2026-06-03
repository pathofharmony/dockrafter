//! Bookmarks, watermark, encryption helpers.

use docrafter_pdf_read::{PdfReader, WatermarkOptions};
use docrafter_pdf_write::{PageBreak, Paragraph, PdfDocument};

fn two_page_pdf() -> Vec<u8> {
    PdfDocument::new()
        .push(Paragraph::new("Alpha page"))
        .push(PageBreak)
        .push(Paragraph::new("Beta page"))
        .to_bytes()
        .unwrap()
}

#[test]
fn bookmarks_and_outline() {
    let mut reader = PdfReader::from_bytes(&two_page_pdf()).unwrap();
    reader.add_bookmark("Start", 1, None).unwrap();
    reader.add_bookmark("End", 2, None).unwrap();
    let bytes = reader.to_bytes().unwrap();
    assert!(bytes.windows(8).any(|w| w == b"Outlines"));
}

#[test]
fn watermark_on_generated_pdf() {
    let mut reader = PdfReader::from_bytes(&two_page_pdf()).unwrap();
    reader
        .add_watermark(
            None,
            &WatermarkOptions {
                text: "CONFIDENTIAL".into(),
                ..Default::default()
            },
        )
        .unwrap();
    let bytes = reader.to_bytes().unwrap();
    assert!(bytes.iter().copied().any(|b| b == b'C'));
    assert!(reader.extract_text().unwrap().contains("Alpha"));
}

#[test]
fn is_not_encrypted_by_default() {
    let reader = PdfReader::from_bytes(&two_page_pdf()).unwrap();
    assert!(!reader.is_encrypted());
}
