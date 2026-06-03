//! Page selection, rotation, and splitting (pypdf-style).

use docrafter_core::{Error, Result};
use lopdf::{Document, Object};

/// Clockwise page rotation in 90° steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotate {
    /// 90° clockwise.
    Clockwise90,
    /// 180°.
    Clockwise180,
    /// 270° clockwise (90° counter-clockwise).
    Clockwise270,
}

impl Rotate {
    fn degrees(self) -> i64 {
        match self {
            Self::Clockwise90 => 90,
            Self::Clockwise180 => 180,
            Self::Clockwise270 => 270,
        }
    }
}

/// Validate 1-based page numbers against `doc`.
pub fn validate_page_numbers(doc: &Document, page_numbers: &[u32]) -> Result<()> {
    let pages = doc.get_pages();
    for &n in page_numbers {
        if !pages.contains_key(&n) {
            return Err(Error::Pdf(format!(
                "invalid page number {n} (document has {} page(s))",
                pages.len()
            )));
        }
    }
    Ok(())
}

/// Keep only the given 1-based pages (mutates `doc`).
pub fn extract_pages(doc: &mut Document, keep: &[u32]) -> Result<()> {
    if keep.is_empty() {
        return Err(Error::Pdf("extract_pages: empty page list".into()));
    }
    validate_page_numbers(doc, keep)?;
    let all: Vec<u32> = doc.get_pages().into_keys().collect();
    let to_delete: Vec<u32> = all.into_iter().filter(|p| !keep.contains(p)).collect();
    doc.delete_pages(&to_delete);
    let _ = doc.prune_objects();
    Ok(())
}

/// Build a new document containing only `keep` pages.
pub fn copy_pages(doc: &Document, keep: &[u32]) -> Result<Document> {
    let mut copy = doc.clone();
    extract_pages(&mut copy, keep)?;
    Ok(copy)
}

/// One PDF per page (1-based order).
pub fn split_pages(doc: &Document) -> Result<Vec<Document>> {
    let nums: Vec<u32> = doc.get_pages().into_keys().collect();
    nums.into_iter().map(|n| copy_pages(doc, &[n])).collect()
}

/// Rotate pages by 90° steps. `page_numbers == None` means all pages.
pub fn rotate_pages(
    doc: &mut Document,
    page_numbers: Option<&[u32]>,
    rotation: Rotate,
) -> Result<()> {
    if let Some(nums) = page_numbers {
        validate_page_numbers(doc, nums)?;
    }
    let delta = rotation.degrees();
    for (num, page_id) in doc.get_pages() {
        if page_numbers.is_some_and(|nums| !nums.contains(&num)) {
            continue;
        }
        let page_dict = doc
            .get_object_mut(page_id)
            .map_err(|e| Error::Pdf(e.to_string()))?
            .as_dict_mut()
            .map_err(|_| Error::Pdf(format!("page {num} is not a dictionary")))?;
        let current = page_dict
            .get(b"Rotate")
            .and_then(|obj| obj.as_i64())
            .unwrap_or(0);
        page_dict.set("Rotate", Object::Integer((current + delta) % 360));
    }
    Ok(())
}
