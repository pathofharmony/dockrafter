//! PDF manipulation API on the public facade.

use docrafter::pdf::{PdfMetadata, PdfReader, Rotate};
use docrafter::prelude::*;

#[test]
fn facade_extract_and_metadata() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Alpha"));
    doc.push(PageBreak);
    doc.push(Paragraph::new("Beta"));
    let bytes = doc.to_bytes().unwrap();

    let mut reader = PdfReader::from_bytes(&bytes).unwrap();
    reader
        .set_metadata(&PdfMetadata {
            title: Some("AB".into()),
            ..Default::default()
        })
        .unwrap();
    reader.extract_pages(&[2]).unwrap();
    assert_eq!(reader.page_count(), 1);
    assert!(reader.extract_text().unwrap().contains("Beta"));
    assert_eq!(reader.metadata().title.as_deref(), Some("AB"));
    reader.rotate(None, Rotate::Clockwise180).unwrap();
}
