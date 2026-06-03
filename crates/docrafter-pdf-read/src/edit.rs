//! In-place PDF edits (replace text, compress streams).

use docrafter_core::{Error, Result};
use lopdf::Document;

use crate::pages::validate_page_numbers;
use crate::replace_cid::replace_text_on_page_cid;

/// Replace literal `from` with `to` in text operators on one page (pypdf-style).
///
/// Tries docrafter DejaVu CID encoding first, then falls back to `lopdf` encodings.
pub fn replace_text_on_page(doc: &mut Document, page: u32, from: &str, to: &str) -> Result<()> {
    validate_page_numbers(doc, &[page])?;
    if replace_text_on_page_cid(doc, page, from, to).is_ok() {
        return Ok(());
    }
    doc.replace_text(page, from, to)
        .map_err(|e| Error::Pdf(format!("replace_text on page {page}: {e}")))
}

/// Replace across all pages.
pub fn replace_text_all(doc: &mut Document, from: &str, to: &str) -> Result<()> {
    let pages: Vec<u32> = doc.get_pages().into_keys().collect();
    for page in pages {
        replace_text_on_page(doc, page, from, to)?;
    }
    Ok(())
}

/// Compress stream objects (smaller file size).
pub fn compress_streams(doc: &mut Document) {
    doc.compress();
}
