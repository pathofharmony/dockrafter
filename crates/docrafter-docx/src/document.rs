//! High-level DOCX document builder.

use std::path::Path;

use docrafter_core::{Error, Result};

use crate::comments::DocxComment;
use crate::package::{pack_docx, prepare_images};
use crate::read::{load_archive, parse_body, parse_comments};
use crate::write::build_document_xml;
use crate::DocxBlock;
use docrafter_office::{Image, List, Paragraph, Table};

/// Word document (create, read, save).
#[derive(Debug, Clone)]
pub struct DocxDocument {
    blocks: Vec<DocxBlock>,
    comments: Vec<DocxComment>,
}

impl Default for DocxDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl DocxDocument {
    /// Empty document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            comments: Vec::new(),
        }
    }

    /// Create from shared office blocks (multi-format export).
    #[must_use]
    pub fn from_blocks(blocks: Vec<DocxBlock>) -> Self {
        Self {
            blocks,
            comments: Vec::new(),
        }
    }

    /// Load an existing `.docx` from disk.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Error::io(path, source))?;
        Self::from_bytes(&bytes)
    }

    /// Parse a `.docx` from memory (paragraphs, tables, images, lists).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let archive = load_archive(bytes)?;
        let blocks = parse_body(&archive)?;
        let comments = parse_comments(&archive)?;
        Ok(Self { blocks, comments })
    }

    /// Append a paragraph.
    pub fn push(&mut self, paragraph: Paragraph) -> &mut Self {
        self.blocks.push(DocxBlock::Paragraph(paragraph));
        self
    }

    /// Append a table.
    pub fn push_table(&mut self, table: Table) -> &mut Self {
        self.blocks.push(DocxBlock::Table(table));
        self
    }

    /// Append an image.
    pub fn push_image(&mut self, image: Image) -> &mut Self {
        self.blocks.push(DocxBlock::Image(image));
        self
    }

    /// Append a numbered list.
    pub fn push_list(&mut self, list: List) -> &mut Self {
        self.blocks.push(DocxBlock::List(list));
        self
    }

    /// Add a review comment (attached to the first paragraph when saving).
    pub fn add_comment(&mut self, author: impl Into<String>, text: impl Into<String>) -> &mut Self {
        let id = self.comments.len() as u32;
        self.comments.push(DocxComment {
            id,
            author: author.into(),
            text: text.into(),
        });
        self
    }

    /// Review comments included in the package.
    #[must_use]
    pub fn comments(&self) -> &[DocxComment] {
        &self.comments
    }

    /// Body blocks in order.
    #[must_use]
    pub fn blocks(&self) -> &[DocxBlock] {
        &self.blocks
    }

    /// Append blocks from another document.
    pub fn append(&mut self, other: &DocxDocument) -> &mut Self {
        self.blocks.extend(other.blocks.iter().cloned());
        self
    }

    /// Plain text lines (paragraphs; tables expand rows; list items).
    #[must_use]
    pub fn paragraph_texts(&self) -> Vec<String> {
        self.blocks
            .iter()
            .flat_map(|block| match block {
                DocxBlock::Paragraph(p) => vec![p.text()],
                DocxBlock::Table(t) => {
                    let mut out = Vec::new();
                    if !t.columns.is_empty() {
                        out.push(t.columns.join(" | "));
                    }
                    for row in &t.rows {
                        out.push(row.join(" | "));
                    }
                    out
                }
                DocxBlock::Image(_) => Vec::new(),
                DocxBlock::List(list) => list.items().to_vec(),
            })
            .collect()
    }

    /// Serialize to `.docx` bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let has_numbering = self.blocks.iter().any(|b| matches!(b, DocxBlock::List(_)));
        let images: Vec<Image> = self
            .blocks
            .iter()
            .filter_map(|b| match b {
                DocxBlock::Image(img) => Some(img.clone()),
                _ => None,
            })
            .collect();
        let first_rel = if has_numbering { 3 } else { 2 };
        let image_refs = prepare_images(&images, first_rel);
        let xml = build_document_xml(&self.blocks, &image_refs, &self.comments);
        pack_docx(&xml, has_numbering, &image_refs, &self.comments)
    }

    /// Write `.docx` to a path.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let bytes = self.to_bytes()?;
        std::fs::write(path, &bytes).map_err(|source| Error::io(path, source))?;
        Ok(())
    }

    /// Number of top-level blocks.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the document has no blocks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_core::Style;
    use docrafter_testing::assert_docx_structure;

    #[test]
    fn writes_hello_docx() {
        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("Hello, docrafter!").style(Style::new().font_size(14.0)));
        let bytes = doc.to_bytes().unwrap();
        assert_docx_structure(&bytes, &["Hello, docrafter!"]);
    }

    #[test]
    fn open_roundtrip_preserves_runs() {
        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("Hello ").run("bold", Style::new().bold()));
        let bytes = doc.to_bytes().unwrap();
        let loaded = DocxDocument::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.paragraph_texts(), vec!["Hello bold".to_string()]);
    }

    #[test]
    fn comments_roundtrip() {
        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("Body text"));
        doc.add_comment("Reviewer", "Please check");
        doc.add_comment("Editor", "LGTM");
        let loaded = DocxDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
        assert_eq!(loaded.comments().len(), 2);
        assert_eq!(loaded.comments()[0].author, "Reviewer");
        assert_eq!(loaded.comments()[0].text, "Please check");
        assert_eq!(loaded.comments()[1].author, "Editor");
        assert_eq!(loaded.comments()[1].text, "LGTM");
    }

    #[test]
    fn comments_xml_in_package() {
        use std::io::{Cursor, Read};
        use zip::ZipArchive;

        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("Body"));
        doc.add_comment("Reviewer", "Please check");
        let bytes = doc.to_bytes().unwrap();
        let mut zip = ZipArchive::new(Cursor::new(bytes)).expect("docx zip");
        let mut comments = String::new();
        zip.by_name("word/comments.xml")
            .expect("comments part")
            .read_to_string(&mut comments)
            .unwrap();
        assert!(comments.contains("Please check"));
        assert!(comments.contains("Reviewer"));
    }

    #[test]
    fn strikethrough_roundtrip() {
        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("was ").run("old", Style::new().strikethrough()));
        let loaded = DocxDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
        let p = match &loaded.blocks()[0] {
            DocxBlock::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert!(p
            .runs()
            .iter()
            .any(|r| r.resolved_style().is_strikethrough()));
    }

    #[test]
    fn underline_roundtrip() {
        let mut doc = DocxDocument::new();
        doc.push(Paragraph::new("see ").run("link", Style::new().underline()));
        let bytes = doc.to_bytes().unwrap();
        let loaded = DocxDocument::from_bytes(&bytes).unwrap();
        let p = match &loaded.blocks()[0] {
            DocxBlock::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert!(p.runs().iter().any(|r| r.resolved_style().is_underline()));
    }

    #[test]
    fn table_roundtrip() {
        let mut doc = DocxDocument::new();
        doc.push_table(
            Table::professional()
                .columns(["Name", "Hours"])
                .row(["Ann", "40"]),
        );
        let loaded = DocxDocument::from_bytes(&doc.to_bytes().unwrap()).unwrap();
        match &loaded.blocks()[0] {
            DocxBlock::Table(t) => {
                assert_eq!(t.columns, vec!["Name", "Hours"]);
                assert_eq!(t.rows[0], vec!["Ann", "40"]);
            }
            _ => panic!("expected table"),
        }
    }
}
