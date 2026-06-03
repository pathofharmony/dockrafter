//! Template substitution and report builder.

use docrafter_office::OfficeDocument;
use docrafter_template::{apply_context, substitute, Context, ReportBuilder};

#[test]
fn substitute_replaces_placeholders() {
    let ctx = Context::new().with("name", "Anna");
    assert_eq!(substitute("Hello {{name}}!", &ctx), "Hello Anna!");
    assert_eq!(substitute("{{missing}}", &ctx), "{{missing}}");
}

#[test]
fn apply_context_to_paragraph_and_table() {
    let mut doc = OfficeDocument::new();
    doc.push(docrafter_office::Paragraph::new("Report {{period}}"));
    doc.push_table(
        docrafter_office::Table::professional()
            .columns(["{{col}}", "Hours"])
            .row(["{{who}}", "40"]),
    );
    let ctx = Context::new()
        .with("period", "May 2026")
        .with("col", "Name")
        .with("who", "Anna");
    let out = apply_context(&doc, &ctx);
    let blocks = out.blocks();
    assert!(blocks[0]
        .as_paragraph()
        .unwrap()
        .text()
        .contains("May 2026"));
    let table = blocks[1].as_table().unwrap();
    assert_eq!(table.columns[0], "Name");
    assert_eq!(table.rows[0][0], "Anna");
}

#[test]
fn report_builder_monthly() {
    let rows = vec![
        vec!["Anna".into(), "142".into()],
        vec!["Igor".into(), "98".into()],
    ];
    let doc = ReportBuilder::new()
        .title("Report for {{month}}")
        .paragraph("Generated on {{date}}")
        .table_professional(["Employee", "Hours"], &rows)
        .build(
            &Context::new()
                .with("month", "May")
                .with("date", "2026-05-01"),
        )
        .unwrap();
    let text: String = doc
        .blocks()
        .iter()
        .filter_map(|b| match b {
            docrafter_office::OfficeBlock::Paragraph(p) => Some(p.text()),
            _ => None,
        })
        .collect();
    assert!(text.contains("May"));
    assert!(text.contains("2026-05-01"));
}

// Helpers for tests — expose via office if needed
trait BlockExt {
    fn as_paragraph(&self) -> Option<&docrafter_office::Paragraph>;
    fn as_table(&self) -> Option<&docrafter_office::Table>;
}

impl BlockExt for docrafter_office::OfficeBlock {
    fn as_paragraph(&self) -> Option<&docrafter_office::Paragraph> {
        match self {
            docrafter_office::OfficeBlock::Paragraph(p) => Some(p),
            _ => None,
        }
    }
    fn as_table(&self) -> Option<&docrafter_office::Table> {
        match self {
            docrafter_office::OfficeBlock::Table(t) => Some(t),
            _ => None,
        }
    }
}
