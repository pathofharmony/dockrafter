//! Format-agnostic office document builder (DOCX, ODT, PDF export).

use crate::blocks::OfficeBlock;
use crate::elements::{Image, List, Paragraph, Table};

/// Document body shared across Word, LibreOffice, and multi-format export.
#[derive(Debug, Clone, Default)]
pub struct OfficeDocument {
    blocks: Vec<OfficeBlock>,
}

impl OfficeDocument {
    /// Empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from existing blocks (e.g. after roundtrip).
    #[must_use]
    pub fn from_blocks(blocks: Vec<OfficeBlock>) -> Self {
        Self { blocks }
    }

    /// Append a paragraph.
    pub fn push(&mut self, paragraph: Paragraph) -> &mut Self {
        self.blocks.push(OfficeBlock::Paragraph(paragraph));
        self
    }

    /// Append a table.
    pub fn push_table(&mut self, table: Table) -> &mut Self {
        self.blocks.push(OfficeBlock::Table(table));
        self
    }

    /// Append an image.
    pub fn push_image(&mut self, image: Image) -> &mut Self {
        self.blocks.push(OfficeBlock::Image(image));
        self
    }

    /// Append a numbered list.
    pub fn push_list(&mut self, list: List) -> &mut Self {
        self.blocks.push(OfficeBlock::List(list));
        self
    }

    /// Body blocks in document order.
    #[must_use]
    pub fn blocks(&self) -> &[OfficeBlock] {
        &self.blocks
    }

    /// Whether the document has no content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Append all blocks from another document (python-docx / combine style).
    pub fn append(&mut self, other: &OfficeDocument) -> &mut Self {
        self.blocks.extend(other.blocks.iter().cloned());
        self
    }

    /// Consume into block vector (for backend serializers).
    #[must_use]
    pub fn into_blocks(self) -> Vec<OfficeBlock> {
        self.blocks
    }
}
