//! Text extraction from docrafter-generated PDFs.

use docrafter_pdf_read::PdfReader;
use docrafter_pdf_write::{Paragraph, PdfDocument};

#[test]
fn extract_text_from_generated_hello() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!"));
    let bytes = doc.to_bytes().unwrap();

    let reader = PdfReader::from_bytes(&bytes).unwrap();
    let text = reader
        .extract_text()
        .expect("extract_text on generated PDF");
    assert!(
        text.contains("Hello") && text.contains("docrafter"),
        "expected text in extraction, got: {text:?}"
    );
}
