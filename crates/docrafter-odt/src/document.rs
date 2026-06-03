//! High-level ODT document builder.

use std::path::Path;

use docrafter_core::{Error, Result};
use docrafter_office::OfficeBlock;
use docrafter_office::{Image, List, Paragraph, Table};

use crate::package::{pack_odt, prepare_images};
use crate::read::{load_archive, parse_body};
use crate::write::build_content_xml;

/// OpenDocument Text document (LibreOffice, OpenOffice).
#[derive(Debug, Clone)]
pub struct OdtDocument {
    blocks: Vec<OfficeBlock>,
}

impl Default for OdtDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl OdtDocument {
    /// Empty document.
    #[must_use]
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Create from shared office blocks (multi-format export).
    #[must_use]
    pub fn from_blocks(blocks: Vec<OfficeBlock>) -> Self {
        Self { blocks }
    }

    /// Load `.odt` from disk.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::io(path, source))?;
        Self::from_bytes(&bytes)
    }

    /// Parse from memory.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let archive = load_archive(bytes)?;
        let blocks = parse_body(&archive)?;
        Ok(Self { blocks })
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

    /// Body blocks in order.
    #[must_use]
    pub fn blocks(&self) -> &[OfficeBlock] {
        &self.blocks
    }

    /// Append blocks from another document.
    pub fn append(&mut self, other: &OdtDocument) -> &mut Self {
        self.blocks.extend(other.blocks.iter().cloned());
        self
    }

    /// Plain-text lines for tests.
    #[must_use]
    pub fn paragraph_texts(&self) -> Vec<String> {
        self.blocks
            .iter()
            .flat_map(|block| match block {
                OfficeBlock::Paragraph(p) => vec![p.text()],
                OfficeBlock::Table(t) => {
                    let mut out = Vec::new();
                    if !t.columns.is_empty() {
                        out.push(t.columns.join(" | "));
                    }
                    for row in &t.rows {
                        out.push(row.join(" | "));
                    }
                    out
                }
                OfficeBlock::Image(_) => Vec::new(),
                OfficeBlock::List(list) => list.items().to_vec(),
            })
            .collect()
    }

    /// Serialize to `.odt` bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let images: Vec<Image> = self
            .blocks
            .iter()
            .filter_map(|b| match b {
                OfficeBlock::Image(img) => Some(img.clone()),
                _ => None,
            })
            .collect();
        let image_refs = prepare_images(&images);
        let xml = build_content_xml(&self.blocks, &image_refs);
        pack_odt(&xml, &image_refs)
    }

    /// Write to path.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.to_bytes()?;
        std::fs::write(path, &bytes).map_err(|source| Error::io(path, source))?;
        Ok(())
    }

    /// Number of blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_testing::assert_odt_structure;

    #[test]
    fn writes_hello_odt() {
        let mut doc = OdtDocument::new();
        doc.push(Paragraph::new("Hello, LibreOffice!"));
        let bytes = doc.to_bytes().unwrap();
        assert_odt_structure(&bytes, &["Hello, LibreOffice!"]);
    }
}
