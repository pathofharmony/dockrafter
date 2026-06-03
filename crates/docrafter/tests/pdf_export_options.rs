//! PDF export layout options and office open.

use docrafter::export::{
    export_bytes_with_pdf_options, export_save, OutputFormat, PdfExportOptions,
};
use docrafter::office::{open, OfficeBlock, OfficeDocument, Paragraph};
use docrafter_core::PageSize;
use docrafter_layout::LayoutMargins;
use docrafter_testing::assert_pdf_structure;

#[test]
fn letter_export_with_wide_margins() {
    let mut doc = OfficeDocument::new();
    doc.push(Paragraph::new("Letter layout"));
    let opts = PdfExportOptions {
        page_size: PageSize::letter(),
        margins: LayoutMargins {
            left: 72.0,
            right: 72.0,
            top: 72.0,
            bottom: 72.0,
        },
        ..Default::default()
    };
    let pdf = export_bytes_with_pdf_options(&doc, OutputFormat::Pdf, &opts).unwrap();
    assert_pdf_structure(&pdf, 1, &["Letter"]);
}

#[test]
fn office_open_docx_roundtrip() {
    let mut doc = OfficeDocument::new();
    doc.push(Paragraph::new("Open test"));
    let path = std::env::temp_dir().join("docrafter_open_test.docx");
    export_save(&doc, &path, OutputFormat::Docx).unwrap();
    let loaded = open(&path).unwrap();
    match &loaded.blocks()[0] {
        OfficeBlock::Paragraph(p) => assert!(p.text().contains("Open")),
        _ => panic!("expected paragraph"),
    }
    let _ = std::fs::remove_file(path);
}
