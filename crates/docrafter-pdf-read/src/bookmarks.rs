//! PDF outline / bookmarks (navigation pane).

use docrafter_core::{Error, Result};
use lopdf::{Bookmark, Document, Object};

use crate::pages::validate_page_numbers;

/// Add a bookmark pointing at a 1-based page. Returns bookmark id for nested entries.
pub fn add_bookmark(
    doc: &mut Document,
    title: impl Into<String>,
    page: u32,
    parent_id: Option<u32>,
) -> Result<u32> {
    validate_page_numbers(doc, &[page])?;
    let page_id = *doc
        .get_pages()
        .get(&page)
        .ok_or_else(|| Error::Pdf(format!("page {page} not found")))?;
    let bookmark = Bookmark::new(title.into(), [0.0, 0.0, 0.0], 0, page_id);
    Ok(doc.add_bookmark(bookmark, parent_id))
}

/// Write accumulated bookmarks into the catalog `/Outlines` tree.
pub fn rebuild_outline(doc: &mut Document) -> Result<()> {
    doc.adjust_zero_pages();
    if let Some(outline_id) = doc.build_outline() {
        doc.catalog_mut()
            .map_err(|e| Error::Pdf(e.to_string()))?
            .set("Outlines", Object::Reference(outline_id));
    }
    Ok(())
}
