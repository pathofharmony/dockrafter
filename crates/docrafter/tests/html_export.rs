//! Phase 1.8: HTML → office formats and PDF header/footer.

use docrafter::prelude::*;
use docrafter_testing::assert_pdf_structure;

#[test]
fn html_to_pdf_and_docx() {
    let html = "<h1>Report</h1><p>Line <b>one</b></p><ul><li>A</li><li>B</li></ul>";
    let doc = html_to_office(html).unwrap();
    assert_eq!(doc.blocks().len(), 3);

    let pdf = export_bytes(&doc, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&pdf, 1, &["Report", "Line", "one"]);

    let docx = export_bytes(&doc, OutputFormat::Docx).unwrap();
    assert!(docx.starts_with(b"PK"));
}

#[test]
fn pdf_header_footer_page_numbers() {
    let mut doc = PdfDocument::new().with_header_footer(
        PageHeaderFooter::new()
            .header("Confidential")
            .page_numbers(),
    );
    doc.push(Paragraph::new("Body"));
    let bytes = doc.to_bytes().unwrap();
    assert_pdf_structure(&bytes, 1, &["Confidential", "Page 1 of 1", "Body"]);
}

#[test]
fn html_text_align_exports_to_pdf() {
    let html = r#"<h1 style="text-align: center">Centered</h1>"#;
    let doc = html_to_office(html).unwrap();
    let pdf = export_bytes(&doc, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&pdf, 1, &["Centered"]);
}
