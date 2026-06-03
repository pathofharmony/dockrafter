//! Phase 1.4: template reports → multi-format export.

use docrafter::export::{export_bytes, OutputFormat};
use docrafter::prelude::*;
use docrafter_testing::{assert_docx_structure, assert_odt_structure, assert_pdf_structure};

#[test]
fn report_builder_exports_all_formats() {
    let doc = ReportBuilder::new()
        .title("Q{{quarter}} report")
        .table_professional(["Metric", "Value"], &[vec!["Revenue".into(), "100".into()]])
        .build(&Context::new().with("quarter", "2"))
        .unwrap();

    let pdf = export_bytes(&doc, OutputFormat::Pdf).unwrap();
    assert_pdf_structure(&pdf, 1, &["Q2", "Revenue"]);

    let docx = export_bytes(&doc, OutputFormat::Docx).unwrap();
    assert_docx_structure(&docx, &["Q2 report", "Revenue"]);

    let odt = export_bytes(&doc, OutputFormat::Odt).unwrap();
    assert_odt_structure(&odt, &["Q2 report", "Revenue"]);
}
