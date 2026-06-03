//! Concatenate multiple PDF documents (lopdf merge example, simplified).

use std::collections::BTreeMap;

use docrafter_core::{Error, Result};
use lopdf::{Document, Object, ObjectId};

/// Merge multiple PDFs into one document (pages in order).
pub fn merge_documents(mut documents: Vec<Document>) -> Result<Document> {
    if documents.is_empty() {
        return Err(Error::Pdf("no documents to merge".into()));
    }
    if documents.len() == 1 {
        return Ok(documents.pop().expect("one document"));
    }

    let version = documents[0].version.clone();
    let mut max_id = 1;
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version(&version);

    for doc in &mut documents {
        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        for (_, object_id) in doc.get_pages() {
            let page = doc
                .get_object(object_id)
                .map_err(|e| Error::Pdf(e.to_string()))?
                .to_owned();
            documents_pages.insert(object_id, page);
        }
        documents_objects.extend(doc.objects.clone());
    }

    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    for (object_id, object) in documents_objects {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((
                    catalog_object.map(|(id, _)| id).unwrap_or(object_id),
                    object,
                ));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref old)) = pages_object {
                        if let Ok(old_dictionary) = old.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }
                    pages_object = Some((
                        pages_object.map(|(id, _)| id).unwrap_or(object_id),
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            "Page" | "Outlines" | "Outline" => {}
            _ => {
                document.objects.insert(object_id, object);
            }
        }
    }

    let Some((page_id, page_object)) = pages_object else {
        return Err(Error::Pdf("Pages root not found while merging".into()));
    };
    let Some((catalog_id, catalog_object)) = catalog_object else {
        return Err(Error::Pdf("Catalog root not found while merging".into()));
    };

    for (object_id, object) in &documents_pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", page_id);
            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = page_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .keys()
                .map(|&id| Object::Reference(id))
                .collect::<Vec<_>>(),
        );
        document
            .objects
            .insert(page_id, Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_object.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", page_id);
        dictionary.remove(b"Outlines");
        document
            .objects
            .insert(catalog_id, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();

    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use docrafter_core::Style;
    use docrafter_pdf_write::{PageBreak, Paragraph, PdfDocument};
    use docrafter_testing::assert_pdf_structure;

    fn one_page(label: &str) -> Vec<u8> {
        let mut doc = PdfDocument::new();
        doc.push(Paragraph::new(label).style(Style::new().font_size(12.0)));
        doc.to_bytes().unwrap()
    }

    fn two_page() -> Vec<u8> {
        let mut doc = PdfDocument::new();
        doc.push(Paragraph::new("Page one").style(Style::new().font_size(12.0)));
        doc.push(PageBreak);
        doc.push(Paragraph::new("Page two").style(Style::new().font_size(12.0)));
        doc.to_bytes().unwrap()
    }

    #[test]
    fn merge_concatenates_pages() {
        let a = Document::load_mem(&one_page("Doc A")).unwrap();
        let b = Document::load_mem(&two_page()).unwrap();
        let merged = merge_documents(vec![a, b]).unwrap();
        assert_eq!(merged.get_pages().len(), 3);

        let bytes = {
            let mut doc = merged;
            let mut buf = Vec::new();
            doc.save_to(&mut buf).unwrap();
            buf
        };
        let reader = crate::PdfReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 3);
        assert_pdf_structure(&bytes, 0, &["Doc A", "Page one", "Page two"]);
    }
}
