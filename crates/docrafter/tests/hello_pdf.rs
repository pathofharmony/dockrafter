//! Integration tests for Phase 0 deliverable.

use docrafter::prelude::*;
use docrafter_testing::{assert_pdf_snapshot_file, assert_pdf_structure};
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

#[test]
fn hello_pdf_integration() {
    let mut doc = PdfDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!").style(Style::new().font_size(14.0)));

    let bytes = doc.to_bytes().expect("serialize PDF");
    assert_pdf_structure(&bytes, 1, &["Hello, docrafter!"]);
    assert_pdf_snapshot_file(&bytes, &snapshot_path("hello_pdf.sha256"));
}

#[test]
fn russian_heading_pdf() {
    let mut doc = PdfDocument::new();
    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af").unwrap())),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 1, &["Отчёт за май"]);
    assert_pdf_snapshot_file(&bytes, &snapshot_path("russian_heading.sha256"));
}
