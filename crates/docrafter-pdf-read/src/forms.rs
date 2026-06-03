//! AcroForm text fields (basic widgets).

use docrafter_core::{Error, Result};
use lopdf::{dictionary, Document, Object};

use crate::annotations::{append_annotation, PdfRect};
use crate::pages::validate_page_numbers;

/// A single-line PDF text field widget.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfTextField {
    /// 1-based page number.
    pub page: u32,
    /// Widget rectangle in PDF points.
    pub rect: PdfRect,
    /// Field name (`/T`).
    pub name: String,
    /// Current value (`/V`).
    pub value: String,
}

/// Add a text field widget on a page (AcroForm `/Tx`).
pub fn add_text_field(
    doc: &mut Document,
    page: u32,
    rect: PdfRect,
    name: impl Into<String>,
    value: impl Into<String>,
) -> Result<()> {
    validate_page_numbers(doc, &[page])?;
    let page_id = *doc
        .get_pages()
        .get(&page)
        .ok_or_else(|| Error::Pdf(format!("page {page} not found")))?;
    let name = name.into();
    let value = value.into();
    let annot_id = doc.add_object(dictionary! {
        "Type" => "Annot",
        "Subtype" => "Widget",
        "FT" => "Tx",
        "Rect" => vec![
            Object::Real(rect[0]),
            Object::Real(rect[1]),
            Object::Real(rect[2]),
            Object::Real(rect[3]),
        ],
        "T" => Object::string_literal(name.as_str()),
        "V" => Object::string_literal(value.as_str()),
        "F" => Object::Integer(4),
    });
    append_annotation(doc, page_id, annot_id)?;
    ensure_acroform(doc, annot_id)
}

/// List text field widgets in the document.
pub fn list_text_fields(doc: &Document) -> Result<Vec<PdfTextField>> {
    let mut fields = Vec::new();
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
            if let Some(field) = parse_text_field(doc, page, annot_id)? {
                fields.push(field);
            }
        }
    }
    Ok(fields)
}

fn ensure_acroform(doc: &mut Document, field_id: lopdf::ObjectId) -> Result<()> {
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok());
    let Some(catalog_id) = catalog_id else {
        return Ok(());
    };
    let acroform_id = match doc
        .get_dictionary(catalog_id)
        .and_then(|d| d.get(b"AcroForm"))
    {
        Ok(Object::Reference(id)) => *id,
        Ok(Object::Dictionary(_)) => catalog_id,
        _ => {
            let form_id = doc.add_object(dictionary! {
                "Fields" => vec![Object::Reference(field_id)],
            });
            doc.get_dictionary_mut(catalog_id)
                .map_err(|e| Error::Pdf(e.to_string()))?
                .set("AcroForm", Object::Reference(form_id));
            return Ok(());
        }
    };
    let form = doc
        .get_dictionary_mut(acroform_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    match form.get_mut(b"Fields") {
        Ok(Object::Array(arr)) => arr.push(Object::Reference(field_id)),
        Err(_) => form.set("Fields", vec![Object::Reference(field_id)]),
        _ => return Err(Error::Pdf("unexpected AcroForm Fields".into())),
    }
    Ok(())
}

fn parse_text_field(
    doc: &Document,
    page: u32,
    annot_id: lopdf::ObjectId,
) -> Result<Option<PdfTextField>> {
    let dict = doc
        .get_dictionary(annot_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    let subtype = dict
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).into_owned());
    if subtype.as_deref() != Some("Widget") {
        return Ok(None);
    }
    let ft = dict
        .get(b"FT")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).into_owned());
    if ft.as_deref() != Some("Tx") {
        return Ok(None);
    }
    let rect = parse_rect(dict.get(b"Rect").ok())?;
    let name = dict
        .get(b"T")
        .ok()
        .and_then(|o| o.as_string().ok())
        .map(|s| s.into_owned())
        .unwrap_or_default();
    let value = dict
        .get(b"V")
        .ok()
        .and_then(|o| o.as_string().ok())
        .map(|s| s.into_owned())
        .unwrap_or_default();
    Ok(Some(PdfTextField {
        page,
        rect,
        name,
        value,
    }))
}

fn parse_rect(obj: Option<&Object>) -> Result<PdfRect> {
    let Some(arr) = obj.and_then(|o| o.as_array().ok()) else {
        return Err(Error::Pdf("field missing Rect".into()));
    };
    if arr.len() < 4 {
        return Err(Error::Pdf("field Rect too short".into()));
    }
    Ok([
        arr[0].as_float().unwrap_or(0.0),
        arr[1].as_float().unwrap_or(0.0),
        arr[2].as_float().unwrap_or(0.0),
        arr[3].as_float().unwrap_or(0.0),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_pdf_write::{Paragraph, PdfDocument};
    use lopdf::Document;

    #[test]
    fn add_and_list_text_field() {
        let mut pdf = PdfDocument::new();
        pdf.push(Paragraph::new("x"));
        let bytes = pdf.to_bytes().unwrap();
        let mut doc = Document::load_mem(&bytes).unwrap();
        add_text_field(&mut doc, 1, [1.0, 2.0, 3.0, 4.0], "f1", "v1").unwrap();
        let fields = list_text_fields(&doc).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "f1");
        assert_eq!(fields[0].value, "v1");
        assert_eq!(fields[0].rect, [1.0, 2.0, 3.0, 4.0]);

        let mut saved = Vec::new();
        doc.save_to(&mut saved).unwrap();
        let reloaded = Document::load_mem(&saved).unwrap();
        let again = list_text_fields(&reloaded).unwrap();
        assert_eq!(again[0].rect, [1.0, 2.0, 3.0, 4.0]);
    }
}
