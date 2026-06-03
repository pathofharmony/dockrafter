//! PDF text extraction (pypdf + OCR API).

use docrafter::pdf::{Paragraph, PdfDocument, PdfReader};
use docrafter_pdf_read::TextExtractMode;

#[test]
fn extract_text_from_generated_pdf() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Hello extract"));
    let bytes = doc.to_bytes().unwrap();

    let reader = PdfReader::from_bytes(&bytes).unwrap();
    let text = reader.extract_text().unwrap();
    assert!(text.contains("Hello"));
    assert!(text.contains("extract"));
}

#[test]
fn extract_text_mode_embedded_default() {
    let bytes = PdfDocument::new()
        .push(Paragraph::new("Mode test"))
        .to_bytes()
        .unwrap();
    let reader = PdfReader::from_bytes(&bytes).unwrap();
    let text = reader.extract_text_mode(TextExtractMode::Embedded).unwrap();
    assert!(text.contains("Mode"));
}
