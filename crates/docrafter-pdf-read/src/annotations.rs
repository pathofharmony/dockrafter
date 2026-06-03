//! PDF link annotations (URI hotspots).

use docrafter_core::{Error, Result};
use lopdf::{dictionary, Document, Object};

use crate::pages::validate_page_numbers;

/// Rectangle `[left, bottom, right, top]` in PDF points.
pub type PdfRect = [f32; 4];

/// A URI link on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfLink {
    /// 1-based page number.
    pub page: u32,
    /// Clickable area.
    pub rect: PdfRect,
    /// Target URL.
    pub uri: String,
}

/// Add a URI link annotation on a page.
pub fn add_uri_link(
    doc: &mut Document,
    page: u32,
    rect: PdfRect,
    uri: impl Into<String>,
) -> Result<()> {
    validate_page_numbers(doc, &[page])?;
    let page_id = *doc
        .get_pages()
        .get(&page)
        .ok_or_else(|| Error::Pdf(format!("page {page} not found")))?;
    let uri = uri.into();
    let annot_id = doc.add_object(dictionary! {
        "Type" => "Annot",
        "Subtype" => "Link",
        "Rect" => vec![
            Object::Real(rect[0]),
            Object::Real(rect[1]),
            Object::Real(rect[2]),
            Object::Real(rect[3]),
        ],
        "Border" => vec![Object::Integer(0), Object::Integer(0), Object::Integer(0)],
        "A" => dictionary! {
            "S" => "URI",
            "URI" => Object::string_literal(uri.as_str()),
        },
    });
    append_annotation(doc, page_id, annot_id)
}

/// List URI links in the document.
pub fn list_uri_links(doc: &Document) -> Result<Vec<PdfLink>> {
    let mut links = Vec::new();
    for (page, page_id) in doc.get_pages() {
        let annots = match doc.get_dictionary(page_id).and_then(|d| d.get(b"Annots")) {
            Ok(Object::Array(arr)) => arr.clone(),
            Ok(Object::Reference(id)) => vec![Object::Reference(*id)],
            _ => continue,
        };
        for annot_ref in annots {
            let annot_id = annot_ref
                .as_reference()
                .map_err(|_| Error::Pdf("annotation is not a reference".into()))?;
            if let Some(link) = parse_link_annotation(doc, page, annot_id)? {
                links.push(link);
            }
        }
    }
    Ok(links)
}

pub(crate) fn append_annotation(
    doc: &mut Document,
    page_id: lopdf::ObjectId,
    annot_id: lopdf::ObjectId,
) -> Result<()> {
    let page = doc
        .get_dictionary_mut(page_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    match page.get_mut(b"Annots") {
        Ok(Object::Array(arr)) => arr.push(Object::Reference(annot_id)),
        Ok(Object::Reference(existing)) => {
            let prev = *existing;
            page.set(
                "Annots",
                vec![Object::Reference(prev), Object::Reference(annot_id)],
            );
        }
        Err(_) => page.set("Annots", vec![Object::Reference(annot_id)]),
        _ => return Err(Error::Pdf("unexpected Annots type on page".into())),
    }
    Ok(())
}

fn parse_link_annotation(
    doc: &Document,
    page: u32,
    annot_id: lopdf::ObjectId,
) -> Result<Option<PdfLink>> {
    let dict = doc
        .get_dictionary(annot_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    let subtype = dict
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).into_owned());
    if subtype.as_deref() != Some("Link") {
        return Ok(None);
    }
    let Some(rect) = dict.get(b"Rect").ok().and_then(|o| o.as_array().ok()) else {
        return Ok(None);
    };
    if rect.len() < 4 {
        return Ok(None);
    }
    let pdf_rect = [
        rect[0].as_f32().unwrap_or(0.0),
        rect[1].as_f32().unwrap_or(0.0),
        rect[2].as_f32().unwrap_or(0.0),
        rect[3].as_f32().unwrap_or(0.0),
    ];
    let action = dict.get(b"A").ok();
    let uri = match action {
        Some(Object::Dictionary(d)) => d
            .get(b"URI")
            .ok()
            .and_then(|u| u.as_string().ok())
            .map(|s| s.into_owned()),
        Some(Object::Reference(id)) => doc
            .get_dictionary(*id)
            .ok()
            .and_then(|d| d.get(b"URI").ok())
            .and_then(|u| u.as_string().ok())
            .map(|s| s.into_owned()),
        _ => None,
    };
    let Some(uri) = uri else {
        return Ok(None);
    };
    Ok(Some(PdfLink {
        page,
        rect: pdf_rect,
        uri,
    }))
}
