//! Open PDF or office files through one entry point.

use std::path::Path;

use docrafter_core::{Error, Result};
use docrafter_office::OfficeDocument;
use docrafter_pdf_read::PdfReader;

use crate::office;

/// A loaded document, either PDF or office (DOCX/ODT).
pub enum OpenDocument {
    /// PDF for read/merge/manipulation.
    Pdf(Box<PdfReader>),
    /// Shared office model for export to PDF/DOCX/ODT.
    Office(OfficeDocument),
}

impl OpenDocument {
    /// Open by file extension: `.pdf`, `.docx`, `.odt`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_path(path.as_ref())
    }

    /// Open with optional password (PDF only; ignored for office).
    pub fn open_with_password(path: impl AsRef<Path>, password: Option<&str>) -> Result<Self> {
        let path = path.as_ref();
        let ext = extension_lower(path)?;
        match ext.as_str() {
            "pdf" => {
                let reader = match password {
                    Some(pw) => PdfReader::open_with_password(path, pw)?,
                    None => PdfReader::open(path)?,
                };
                Ok(Self::Pdf(Box::new(reader)))
            }
            "docx" | "odt" => {
                if password.is_some() {
                    return Err(Error::InvalidInput(
                        "password is only supported for PDF files".into(),
                    ));
                }
                Ok(Self::Office(office::open(path)?))
            }
            other => unsupported_ext(path, other),
        }
    }

    fn open_path(path: &Path) -> Result<Self> {
        Self::open_with_password(path, None)
    }

    /// Path extension in lowercase, or error.
    pub fn extension_lower(path: &Path) -> Result<String> {
        extension_lower(path)
    }
}

fn extension_lower(path: &Path) -> Result<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .ok_or_else(|| Error::InvalidInput(format!("no extension: {}", path.display())))
}

fn unsupported_ext(path: &Path, ext: &str) -> Result<OpenDocument> {
    Err(Error::InvalidInput(format!(
        "unsupported extension .{ext} for {} (use pdf, docx, or odt)",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{export_save, OutputFormat};
    use crate::office::{OfficeDocument, Paragraph};
    use docrafter_pdf_write::{Paragraph as PdfParagraph, PdfDocument};

    #[test]
    fn opens_generated_pdf_and_docx() {
        let dir = std::env::temp_dir().join("docrafter_open_document");
        let _ = std::fs::create_dir_all(&dir);

        let pdf_path = dir.join("t.pdf");
        let mut pdf = PdfDocument::new();
        pdf.push(PdfParagraph::new("Open API"));
        pdf.save(&pdf_path).unwrap();

        match OpenDocument::open(&pdf_path).unwrap() {
            OpenDocument::Pdf(r) => assert_eq!(r.as_ref().page_count(), 1),
            _ => panic!("expected pdf"),
        }

        let docx_path = dir.join("t.docx");
        let mut doc = OfficeDocument::new();
        doc.push(Paragraph::new("Office"));
        export_save(&doc, &docx_path, OutputFormat::Docx).unwrap();

        match OpenDocument::open(&docx_path).unwrap() {
            OpenDocument::Office(o) => assert!(!o.is_empty()),
            _ => panic!("expected office"),
        }

        let _ = std::fs::remove_dir_all(dir);
    }
}
