//! `{{name}}` placeholder replacement in text and office blocks.

use docrafter_office::{OfficeBlock, OfficeDocument, Paragraph, Table, TextRun};

use crate::context::Context;

/// Replace `{{key}}` placeholders using `ctx` (unknown keys are left unchanged).
#[must_use]
pub fn substitute(template: &str, ctx: &Context) -> String {
    let mut out = template.to_string();
    for (key, value) in ctx.iter() {
        let needle = format!("{{{{{key}}}}}");
        if out.contains(&needle) {
            out = out.replace(&needle, value);
        }
    }
    out
}

/// Apply substitution to every text run in an [`OfficeDocument`].
#[must_use]
pub fn apply_context(doc: &OfficeDocument, ctx: &Context) -> OfficeDocument {
    let blocks = doc
        .blocks()
        .iter()
        .map(|block| match block {
            OfficeBlock::Paragraph(p) => OfficeBlock::Paragraph(substitute_paragraph(p, ctx)),
            OfficeBlock::Table(t) => OfficeBlock::Table(substitute_table(t, ctx)),
            OfficeBlock::Image(img) => OfficeBlock::Image(img.clone()),
            OfficeBlock::List(list) => OfficeBlock::List(substitute_list(list, ctx)),
        })
        .collect();
    OfficeDocument::from_blocks(blocks)
}

fn substitute_paragraph(p: &Paragraph, ctx: &Context) -> Paragraph {
    let runs: Vec<TextRun> = p
        .runs()
        .iter()
        .map(|r| TextRun::new(substitute(r.text(), ctx)).style(r.resolved_style().clone()))
        .collect();
    Paragraph::from_runs(p.paragraph_style().clone(), runs)
}

fn substitute_table(t: &Table, ctx: &Context) -> Table {
    let mut table = Table::new().style(t.style.clone());
    if t.repeat_header_on_new_page {
        table = table.repeat_header_on_new_page(true);
    }
    if !t.column_widths.is_empty() {
        table = table.column_widths(t.column_widths.clone());
    }
    if !t.columns.is_empty() {
        let cols: Vec<String> = t.columns.iter().map(|c| substitute(c, ctx)).collect();
        table = table.columns(cols);
    }
    for row in &t.rows {
        let cells: Vec<String> = row.iter().map(|c| substitute(c, ctx)).collect();
        table = table.row(cells);
    }
    table
}

fn substitute_list(list: &docrafter_office::List, ctx: &Context) -> docrafter_office::List {
    let mut out = docrafter_office::List::new();
    for item in list.items() {
        out = out.item(substitute(item, ctx));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_unknown_keys() {
        let ctx = Context::new().with("a", "1");
        assert_eq!(substitute("{{a}} {{b}}", &ctx), "1 {{b}}");
    }
}
