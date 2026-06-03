//! Phase 1.6: append / merge office documents.

use docrafter::docx::DocxDocument;
use docrafter::export::{export_bytes, OutputFormat};
use docrafter::odt::OdtDocument;
use docrafter::office::{OfficeDocument, Paragraph};
use docrafter_testing::{assert_docx_structure, assert_odt_structure, assert_pdf_structure};

#[test]
fn office_document_append() {
    let mut a = OfficeDocument::new();
    a.push(Paragraph::new("Part A"));
    let mut b = OfficeDocument::new();
    b.push(Paragraph::new("Part B"));
    a.append(&b);
    let pdf = export_bytes(&a, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&pdf, 1, &["Part A", "Part B"]);
}

#[test]
fn docx_append_roundtrip() {
    let mut first = DocxDocument::new();
    first.push(Paragraph::new("First"));
    let mut second = DocxDocument::new();
    second.push(Paragraph::new("Second"));
    first.append(&second);
    let bytes = first.to_bytes().unwrap();
    assert_docx_structure(&bytes, &["First", "Second"]);
    let loaded = DocxDocument::from_bytes(&bytes).unwrap();
    assert!(loaded
        .paragraph_texts()
        .iter()
        .any(|t| t.contains("Second")));
}

#[test]
fn odt_append() {
    let mut doc = OdtDocument::new();
    doc.push(Paragraph::new("One"));
    let mut extra = OdtDocument::new();
    extra.push(Paragraph::new("Two"));
    doc.append(&extra);
    let bytes = doc.to_bytes().unwrap();
    assert_odt_structure(&bytes, &["One", "Two"]);
}
