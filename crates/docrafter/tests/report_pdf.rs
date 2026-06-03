//! Phase 0.1 integration: report layout.

use docrafter::prelude::*;
use docrafter_testing::{assert_pdf_snapshot_file, assert_pdf_structure};
use std::path::PathBuf;

fn snapshot(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name)
}

#[test]
fn report_with_table() {
    let mut doc = PdfDocument::new();
    doc.push(
        Paragraph::new("Отчёт за май")
            .align(Alignment::Center)
            .style(Style::heading1().color_value(Color::from_hex("#1e40af").unwrap())),
    );
    doc.push(Spacer::new(Length::pt(12.0)));
    doc.push(
        Table::professional()
            .columns(["Сотрудник", "Проекты", "Часы"])
            .row(["Анна", "CRM", "142"]),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 1, &["Отчёт за май", "Сотрудник", "Анна"]);
    assert_pdf_snapshot_file(&bytes, &snapshot("report_table.sha256"));
}
