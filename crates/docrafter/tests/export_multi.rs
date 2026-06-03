//! Phase 1: one office model → PDF + DOCX + ODT.

use docrafter::export::{export_bytes, OutputFormat};
use docrafter::office::{OfficeDocument, Paragraph, Table};
use docrafter::prelude::Style;
use docrafter_testing::{assert_docx_structure, assert_odt_structure, assert_pdf_structure};

#[test]
fn export_same_content_to_pdf_docx_odt() {
    let mut doc = OfficeDocument::new();
    doc.push(Paragraph::new("Отчёт ").run("2026", Style::new().bold()));
    doc.push_table(
        Table::professional()
            .columns(["Name", "Hours"])
            .row(["Anna", "40"]),
    );

    let pdf = export_bytes(&doc, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&pdf, 1, &["Отчёт", "2026", "Name", "Anna"]);

    let docx = export_bytes(&doc, OutputFormat::Docx).unwrap();
    assert_docx_structure(&docx, &["Отчёт", "2026", "Name", "Anna"]);

    let odt = export_bytes(&doc, OutputFormat::Odt).unwrap();
    assert_odt_structure(&odt, &["Отчёт", "2026", "Name", "Anna"]);
}
