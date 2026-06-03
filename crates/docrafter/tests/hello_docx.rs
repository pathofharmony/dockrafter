//! Phase 0.3 integration: Hello DOCX.

use docrafter::docx::{DocxDocument, Paragraph, Table};
use docrafter::prelude::{Alignment, Color, Style};
use docrafter_testing::{assert_docx_snapshot_file, assert_docx_structure};
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

#[test]
fn hello_docx_integration() {
    let mut doc = DocxDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!").style(Style::new().font_size(14.0)));

    let bytes = doc.to_bytes().expect("serialize DOCX");
    assert_docx_structure(&bytes, &["Hello, docrafter!"]);
    assert_docx_snapshot_file(&bytes, &snapshot_path("hello_docx.sha256"));
}

#[test]
fn russian_heading_docx() {
    let mut doc = DocxDocument::new();
    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af").unwrap())),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_docx_structure(&bytes, &["Отчёт за май"]);
    assert_docx_snapshot_file(&bytes, &snapshot_path("russian_heading_docx.sha256"));
}

#[test]
fn docx_open_roundtrip() {
    let mut doc = DocxDocument::new();
    doc.push(Paragraph::new("Roundtrip текст"));
    let bytes = doc.to_bytes().unwrap();

    let loaded = DocxDocument::from_bytes(&bytes).unwrap();
    assert_eq!(
        loaded.paragraph_texts(),
        vec!["Roundtrip текст".to_string()]
    );
}

#[test]
fn docx_table_with_header() {
    let mut doc = DocxDocument::new();
    doc.push_table(
        Table::professional()
            .columns(["Name", "Hours"])
            .row(["Anna", "40"]),
    );
    let bytes = doc.to_bytes().unwrap();
    assert_docx_structure(&bytes, &["Name", "Anna"]);
}
