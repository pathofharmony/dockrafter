//! Phase 0.4: Hello ODT (LibreOffice / OpenDocument).

use docrafter::odt::{OdtDocument, Paragraph};
use docrafter::prelude::{Alignment, Color, Style};
use docrafter_testing::{assert_odt_snapshot_file, assert_odt_structure};
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

#[test]
fn hello_odt_integration() {
    let mut doc = OdtDocument::new();
    doc.push(Paragraph::new("Hello, docrafter!").style(Style::new().font_size(14.0)));

    let bytes = doc.to_bytes().expect("serialize ODT");
    assert_odt_structure(&bytes, &["Hello, docrafter!"]);
    assert_odt_snapshot_file(&bytes, &snapshot_path("hello_odt.sha256"));
}

#[test]
fn russian_heading_odt() {
    let mut doc = OdtDocument::new();
    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af").unwrap())),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_odt_structure(&bytes, &["Отчёт за май"]);
    assert_odt_snapshot_file(&bytes, &snapshot_path("russian_heading_odt.sha256"));
}
