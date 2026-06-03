//! Links, replace_text, compress.

use docrafter_pdf_read::PdfReader;
use docrafter_pdf_write::{Paragraph, PdfDocument};

#[test]
fn add_link_roundtrip() {
    let bytes = PdfDocument::new()
        .push(Paragraph::new("Click here"))
        .to_bytes()
        .unwrap();
    let mut reader = PdfReader::from_bytes(&bytes).unwrap();
    reader
        .add_link(1, [50.0, 700.0, 200.0, 750.0], "https://docrafter.example")
        .unwrap();
    let links = reader.links().unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].uri, "https://docrafter.example");
}

#[test]
fn replace_text_on_docrafter_pdf() {
    let bytes = PdfDocument::new()
        .push(Paragraph::new("Hello world"))
        .to_bytes()
        .unwrap();
    let mut reader = PdfReader::from_bytes(&bytes).unwrap();
    reader.replace_text(1, "world", "docrafter").unwrap();
    assert!(reader.extract_text().unwrap().contains("docrafter"));
}
