//! High-level PDF document builder.

#![allow(unused_imports)]

use docrafter_core::{Error, PageSize, Result};
use docrafter_layout::LayoutMargins;
use std::path::Path;

use crate::header_footer::PageHeaderFooter;
use crate::render::PdfRenderer;

pub use crate::flow::{FlowItem, Image, List, PageBreak, Paragraph, Spacer, Table};

/// Flow-based PDF document.
#[derive(Debug)]
pub struct PdfDocument {
    page_size: PageSize,
    margins: LayoutMargins,
    header_footer: Option<PageHeaderFooter>,
    items: Vec<FlowItem>,
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfDocument {
    /// New A4 document.
    #[must_use]
    pub fn new() -> Self {
        Self::with_page_size(PageSize::a4())
    }

    /// New document with explicit page size.
    #[must_use]
    pub fn with_page_size(page_size: PageSize) -> Self {
        Self {
            page_size,
            margins: LayoutMargins::standard(),
            header_footer: None,
            items: Vec::new(),
        }
    }

    /// US Letter page size.
    #[must_use]
    pub fn letter() -> Self {
        Self::with_page_size(PageSize::letter())
    }

    /// Set content margins (points).
    #[must_use]
    pub fn with_margins(mut self, margins: LayoutMargins) -> Self {
        self.margins = margins;
        self
    }

    /// Running header/footer on every page (`{page}` / `{pages}` in footer).
    #[must_use]
    pub fn with_header_footer(mut self, hf: PageHeaderFooter) -> Self {
        self.header_footer = Some(hf);
        self
    }

    /// Append a flow element (paragraph, table, image, list, spacer, page break).
    pub fn push(&mut self, item: impl Into<FlowItem>) -> &mut Self {
        self.items.push(item.into());
        self
    }

    /// Append a numbered list.
    pub fn push_list(&mut self, list: List) -> &mut Self {
        self.items.push(FlowItem::List(list));
        self
    }

    /// Serialize to PDF bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut renderer = PdfRenderer::new(self.page_size).with_margins(self.margins);
        if let Some(hf) = &self.header_footer {
            renderer = renderer.with_header_footer(hf.clone());
        }
        for item in &self.items {
            renderer.push(item.clone());
        }
        renderer.finish()
    }

    /// Write PDF to a file path.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.to_bytes()?;
        std::fs::write(path, &bytes).map_err(|source| Error::io(path, source))?;
        Ok(())
    }

    /// Number of queued flow elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the document has no content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_core::{Alignment, Style, TableStyle};
    use docrafter_testing::assert_pdf_structure;

    #[test]
    fn empty_document_still_produces_page() {
        let doc = PdfDocument::new();
        let bytes = doc.to_bytes().unwrap();
        assert_pdf_structure(&bytes, 1, &[]);
    }

    #[test]
    fn push_paragraph_and_table() {
        let mut doc = PdfDocument::new();
        doc.push(
            Paragraph::new("Отчёт")
                .align(Alignment::Center)
                .style(Style::heading1()),
        );
        doc.push(
            Table::professional()
                .columns(["Сотрудник", "Часы"])
                .row(["Анна", "142"]),
        );
        let bytes = doc.to_bytes().unwrap();
        assert_pdf_structure(&bytes, 1, &["Отчёт", "Сотрудник", "Анна"]);
    }

    #[test]
    fn page_break_produces_two_pages() {
        let mut doc = PdfDocument::new();
        doc.push(Paragraph::new("First page"));
        doc.push(PageBreak);
        doc.push(Paragraph::new("Second page"));
        let bytes = doc.to_bytes().unwrap();
        assert_pdf_structure(&bytes, 2, &["First page", "Second page"]);
    }
}
