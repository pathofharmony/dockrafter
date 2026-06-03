//! Fluent report builder → [`OfficeDocument`].

use docrafter_core::{Alignment, Result, Style};
use docrafter_office::{OfficeDocument, Paragraph, Table};

use crate::context::Context;
use crate::substitute::apply_context;

/// Describes one table in a report.
#[derive(Debug, Clone)]
struct TableSpec {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    professional: bool,
}

/// Build a styled office report from template strings and tabular data.
#[derive(Debug, Clone, Default)]
pub struct ReportBuilder {
    title: Option<String>,
    title_style: Style,
    paragraphs: Vec<String>,
    tables: Vec<TableSpec>,
}

impl ReportBuilder {
    /// Empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Centered title (supports `{{vars}}`).
    #[must_use]
    pub fn title(mut self, text: impl Into<String>) -> Self {
        self.title = Some(text.into());
        self.title_style = Style::heading1();
        self
    }

    /// Title with custom paragraph style.
    #[must_use]
    pub fn title_styled(mut self, text: impl Into<String>, style: Style) -> Self {
        self.title = Some(text.into());
        self.title_style = style;
        self
    }

    /// Body paragraph template.
    #[must_use]
    pub fn paragraph(mut self, text: impl Into<String>) -> Self {
        self.paragraphs.push(text.into());
        self
    }

    /// Data table: column headers + row cell values (all support `{{vars}}`).
    #[must_use]
    pub fn table(
        mut self,
        columns: impl IntoIterator<Item = impl Into<String>>,
        rows: &[Vec<String>],
    ) -> Self {
        self.tables.push(TableSpec {
            columns: columns.into_iter().map(Into::into).collect(),
            rows: rows.to_vec(),
            professional: false,
        });
        self
    }

    /// Table with professional header styling (gray header row).
    #[must_use]
    pub fn table_professional(
        mut self,
        columns: impl IntoIterator<Item = impl Into<String>>,
        rows: &[Vec<String>],
    ) -> Self {
        self.tables.push(TableSpec {
            columns: columns.into_iter().map(Into::into).collect(),
            rows: rows.to_vec(),
            professional: true,
        });
        self
    }

    /// Render templates with `ctx` into an [`OfficeDocument`].
    pub fn build(&self, ctx: &Context) -> Result<OfficeDocument> {
        let mut doc = OfficeDocument::new();

        if let Some(title) = &self.title {
            let style = self.title_style.clone().align(Alignment::Center);
            doc.push(Paragraph::new(title).style(style));
        }

        for p in &self.paragraphs {
            doc.push(Paragraph::new(p));
        }

        for spec in &self.tables {
            let mut table = if spec.professional {
                Table::professional()
            } else {
                Table::new()
            };
            table = table.columns(spec.columns.clone());
            for row in &spec.rows {
                table = table.row(row.clone());
            }
            doc.push_table(table);
        }

        Ok(apply_context(&doc, ctx))
    }
}
