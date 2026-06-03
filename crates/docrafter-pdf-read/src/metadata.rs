//! Document information dictionary (`/Info`).

use docrafter_core::{Error, Result};
use lopdf::{dictionary, Document, Object};

/// Common PDF document metadata (Info dictionary).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PdfMetadata {
    /// Document title.
    pub title: Option<String>,
    /// Author.
    pub author: Option<String>,
    /// Subject.
    pub subject: Option<String>,
    /// Keywords.
    pub keywords: Option<String>,
    /// Creating application.
    pub creator: Option<String>,
    /// PDF producer.
    pub producer: Option<String>,
}

impl PdfMetadata {
    /// Read metadata from a loaded document.
    pub fn from_document(doc: &Document) -> Self {
        Self {
            title: read_info_field(doc, b"Title"),
            author: read_info_field(doc, b"Author"),
            subject: read_info_field(doc, b"Subject"),
            keywords: read_info_field(doc, b"Keywords"),
            creator: read_info_field(doc, b"Creator"),
            producer: read_info_field(doc, b"Producer"),
        }
    }

    /// Apply non-`None` fields to the document Info dictionary (creates `/Info` if missing).
    pub fn apply_to(&self, doc: &mut Document) -> Result<()> {
        let info_id = ensure_info_dict(doc)?;
        let dict = doc
            .get_dictionary_mut(info_id)
            .map_err(|e| Error::Pdf(e.to_string()))?;
        set_optional(dict, b"Title", &self.title);
        set_optional(dict, b"Author", &self.author);
        set_optional(dict, b"Subject", &self.subject);
        set_optional(dict, b"Keywords", &self.keywords);
        set_optional(dict, b"Creator", &self.creator);
        set_optional(dict, b"Producer", &self.producer);
        Ok(())
    }
}

fn read_info_field(doc: &Document, key: &[u8]) -> Option<String> {
    let dict = info_dict(doc)?;
    dict.get(key)
        .ok()
        .and_then(|obj| obj.as_string().ok())
        .map(|cow| cow.into_owned())
}

fn info_dict(doc: &Document) -> Option<&lopdf::Dictionary> {
    let info = doc.trailer.get(b"Info").ok()?;
    match info {
        Object::Dictionary(dict) => Some(dict),
        Object::Reference(id) => doc.get_dictionary(*id).ok(),
        _ => None,
    }
}

fn ensure_info_dict(doc: &mut Document) -> Result<lopdf::ObjectId> {
    if let Ok(info) = doc.trailer.get(b"Info") {
        if let Ok(id) = info.as_reference() {
            return Ok(id);
        }
        if info.as_dict().is_ok() {
            let id = doc.add_object(info.clone());
            doc.trailer.set("Info", Object::Reference(id));
            return Ok(id);
        }
    }
    let id = doc.add_object(dictionary! {});
    doc.trailer.set("Info", Object::Reference(id));
    Ok(id)
}

fn set_optional(dict: &mut lopdf::Dictionary, key: &[u8], value: &Option<String>) {
    if let Some(text) = value {
        dict.set(key, Object::string_literal(text.as_str()));
    }
}
