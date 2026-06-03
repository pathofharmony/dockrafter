//! Monthly report from a template → PDF + DOCX + ODT.

use docrafter::prelude::*;

fn main() -> Result<()> {
    let rows = vec![
        vec!["Anna".into(), "CRM, Dashboard".into(), "142".into()],
        vec!["Igor".into(), "API".into(), "98".into()],
    ];

    let doc = ReportBuilder::new()
        .title("Report for {{month}} {{year}}")
        .paragraph("Generated on {{date}}")
        .table_professional(["Employee", "Projects", "Hours"], &rows)
        .build(
            &Context::new()
                .with("month", "May")
                .with("year", "2026")
                .with("date", "2026-06-03"),
        )?;

    export_bundle(&doc, "report")?;
    println!("Wrote report.pdf, report.docx, report.odt");
    Ok(())
}
