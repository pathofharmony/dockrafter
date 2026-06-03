//! PDF multi-run paragraphs (reportlab / python-docx parity).

use docrafter::pdf::{Paragraph, PdfDocument};
use docrafter::prelude::Style;
use docrafter_testing::assert_pdf_structure;

#[test]
fn pdf_preserves_mixed_bold_runs_in_one_paragraph() {
    let mut doc = PdfDocument::new();
    doc.push(
        Paragraph::new("Hello ")
            .run("bold", Style::new().bold())
            .run(" world", Style::new()),
    );

    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 1, &["Hello", "bold", "world"]);
}

#[test]
fn pdf_export_multi_run_from_office_model() {
    use docrafter::export::{export_bytes, OutputFormat};
    use docrafter::office::{OfficeDocument, Paragraph};

    let mut doc = OfficeDocument::new();
    doc.push(
        Paragraph::new("A ").run(
            "B",
            Style::new()
                .bold()
                .color_value(docrafter_core::Color::rgb(200, 0, 0)),
        ),
    );
    let bytes = export_bytes(&doc, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&bytes, 1, &["A", "B"]);
}
