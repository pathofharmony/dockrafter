//! Open and inspect existing PDF files.

use std::path::Path;

use docrafter_core::{Error, Result};
use lopdf::Document;

use crate::annotations::{add_uri_link, list_uri_links, PdfLink, PdfRect};
use crate::bookmarks::{add_bookmark, rebuild_outline};
use crate::crypto::{is_encrypted, load_decrypt, load_mem_decrypt};
use crate::edit::{compress_streams, replace_text_all, replace_text_on_page};
use crate::encrypt_save::{encrypt_document, EncryptOptions};
use crate::extract::TextExtractMode;
use crate::extract::{extract_all_text, extract_text, extract_with_mode};
use crate::forms::{add_text_field, list_text_fields, PdfTextField};
use crate::merge::merge_documents;
use crate::metadata::PdfMetadata;
#[cfg(feature = "ocr")]
use crate::ocr::OcrOptions;
use crate::pages::{
    copy_pages, extract_pages, rotate_pages, split_pages, validate_page_numbers, Rotate,
};
use crate::watermark::{add_text_watermark, WatermarkOptions};

/// A loaded PDF suitable for inspection and merging.
pub struct PdfReader {
    doc: Document,
}

impl PdfReader {
    /// Load a PDF from the filesystem.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        Document::load(path)
            .map_err(|e| Error::Pdf(format!("failed to load {}: {e}", path.display())))
            .and_then(Self::from_document)
    }

    /// Load and decrypt if the PDF is password-protected.
    pub fn open_with_password(path: impl AsRef<Path>, password: &str) -> Result<Self> {
        let path = path.as_ref();
        load_decrypt(path, password).and_then(Self::from_document)
    }

    /// Parse PDF bytes from memory.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Document::load_mem(bytes)
            .map_err(|e| Error::Pdf(e.to_string()))
            .and_then(Self::from_document)
    }

    /// Parse bytes and decrypt when encrypted.
    pub fn from_bytes_with_password(bytes: &[u8], password: &str) -> Result<Self> {
        load_mem_decrypt(bytes, password).and_then(Self::from_document)
    }

    /// Whether this document is encrypted (may still be decrypted in memory).
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        is_encrypted(&self.doc)
    }

    /// Wrap an already parsed `lopdf` document.
    pub fn from_document(doc: Document) -> Result<Self> {
        if doc.get_pages().is_empty() {
            return Err(Error::Pdf("document has no pages".into()));
        }
        Ok(Self { doc })
    }

    /// Number of pages in the document.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.doc.get_pages().len()
    }

    /// Extract plain text from all pages (embedded fonts / ToUnicode).
    pub fn extract_text(&self) -> Result<String> {
        extract_all_text(&self.doc)
    }

    /// Extract text from specific 1-based page numbers.
    pub fn extract_text_pages(&self, page_numbers: &[u32]) -> Result<String> {
        extract_text(&self.doc, page_numbers)
    }

    /// Extract with mode: embedded only, in-process OCR, or auto.
    pub fn extract_text_mode(&self, mode: TextExtractMode) -> Result<String> {
        let pdf_bytes = self.to_bytes()?;
        extract_with_mode(&self.doc, &pdf_bytes, mode)
    }

    /// Convenience: OCR with default options (150 DPI).
    #[cfg(feature = "ocr")]
    pub fn extract_text_ocr(&self) -> Result<String> {
        self.extract_text_mode(TextExtractMode::Ocr(OcrOptions::default()))
    }

    /// Document Info metadata (`Title`, `Author`, …).
    #[must_use]
    pub fn metadata(&self) -> PdfMetadata {
        PdfMetadata::from_document(&self.doc)
    }

    /// Set Info metadata fields (replaces only provided keys).
    pub fn set_metadata(&mut self, metadata: &PdfMetadata) -> Result<()> {
        metadata.apply_to(&mut self.doc)
    }

    /// Keep only the given 1-based pages (pypdf `extract_pages`).
    pub fn extract_pages(&mut self, page_numbers: &[u32]) -> Result<()> {
        extract_pages(&mut self.doc, page_numbers)
    }

    /// New reader with a subset of pages (does not modify this reader).
    pub fn with_pages(&self, page_numbers: &[u32]) -> Result<Self> {
        let doc = copy_pages(&self.doc, page_numbers)?;
        Self::from_document(doc)
    }

    /// Remove 1-based pages.
    pub fn delete_pages(&mut self, page_numbers: &[u32]) -> Result<()> {
        validate_page_numbers(&self.doc, page_numbers)?;
        self.doc.delete_pages(page_numbers);
        let _ = self.doc.prune_objects();
        Ok(())
    }

    /// Split into one document per page.
    pub fn split(&self) -> Result<Vec<Self>> {
        split_pages(&self.doc)?
            .into_iter()
            .map(Self::from_document)
            .collect()
    }

    /// Rotate pages (90° steps). `None` = all pages.
    pub fn rotate(&mut self, page_numbers: Option<&[u32]>, rotation: Rotate) -> Result<()> {
        rotate_pages(&mut self.doc, page_numbers, rotation)
    }

    /// Add a navigation bookmark to a 1-based page. Call before `save` / `to_bytes`.
    pub fn add_bookmark(
        &mut self,
        title: impl Into<String>,
        page: u32,
        parent_id: Option<u32>,
    ) -> Result<u32> {
        add_bookmark(&mut self.doc, title, page, parent_id)
    }

    /// Overlay text watermark on pages (`None` = all).
    pub fn add_watermark(
        &mut self,
        page_numbers: Option<&[u32]>,
        options: &WatermarkOptions,
    ) -> Result<()> {
        add_text_watermark(&mut self.doc, page_numbers, options)
    }

    /// Add a clickable URI link on a page.
    pub fn add_link(&mut self, page: u32, rect: PdfRect, uri: impl Into<String>) -> Result<()> {
        add_uri_link(&mut self.doc, page, rect, uri)
    }

    /// URI links present in the document.
    pub fn links(&self) -> Result<Vec<PdfLink>> {
        list_uri_links(&self.doc)
    }

    /// Add an AcroForm text field widget.
    pub fn add_text_field(
        &mut self,
        page: u32,
        rect: PdfRect,
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<()> {
        add_text_field(&mut self.doc, page, rect, name, value)
    }

    /// Text field widgets in the document.
    pub fn text_fields(&self) -> Result<Vec<PdfTextField>> {
        list_text_fields(&self.doc)
    }

    /// Replace text in page content streams on one page.
    pub fn replace_text(&mut self, page: u32, from: &str, to: &str) -> Result<()> {
        replace_text_on_page(&mut self.doc, page, from, to)
    }

    /// Replace text on every page.
    pub fn replace_text_all(&mut self, from: &str, to: &str) -> Result<()> {
        replace_text_all(&mut self.doc, from, to)
    }

    /// Compress internal streams before saving.
    pub fn compress(&mut self) {
        compress_streams(&mut self.doc);
    }

    /// Encrypt with a user password before the next `save` / `to_bytes` (revision 2 / 40-bit RC4).
    pub fn encrypt(&mut self, options: &EncryptOptions) -> Result<()> {
        encrypt_document(&mut self.doc, options)
    }

    /// Append all pages from `other` after this document's pages.
    pub fn merge(&mut self, other: &PdfReader) -> Result<()> {
        let merged = merge_documents(vec![self.doc.clone(), other.doc.clone()])?;
        self.doc = merged;
        Ok(())
    }

    /// Serialize to PDF bytes (flushes bookmarks into `/Outlines` when present).
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut doc = self.doc.clone();
        rebuild_outline(&mut doc)?;
        let mut buf = Vec::new();
        doc.save_to(&mut buf)
            .map_err(|e| Error::Pdf(e.to_string()))?;
        Ok(buf)
    }

    /// Write PDF bytes to a path (flushes bookmarks when present).
    pub fn save(&mut self, path: impl AsRef<Path>) -> Result<()> {
        rebuild_outline(&mut self.doc)?;
        let path = path.as_ref();
        self.doc
            .save(path)
            .map(|_| ())
            .map_err(|e| Error::Pdf(format!("failed to save {}: {e}", path.display())))
    }

    /// Borrow the underlying `lopdf` document (advanced use).
    #[must_use]
    pub fn inner(&self) -> &Document {
        &self.doc
    }
}
