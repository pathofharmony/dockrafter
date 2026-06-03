//! Format-agnostic office elements (shared by DOCX and ODT).

use std::path::Path;

use docrafter_core::{Error, Result};
use docrafter_docx::DocxDocument;
use docrafter_odt::OdtDocument;

pub use docrafter_office::{Image, List, OfficeBlock, OfficeDocument, Paragraph, Table, TextRun};

/// Load `.docx` or `.odt` into a shared [`OfficeDocument`].
pub fn open(path: impl AsRef<Path>) -> Result<OfficeDocument> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| Error::InvalidInput(format!("no extension: {}", path.display())))?;
    match ext.to_ascii_lowercase().as_str() {
        "docx" => {
            let doc = DocxDocument::open(path)?;
            Ok(OfficeDocument::from_blocks(doc.blocks().to_vec()))
        }
        "odt" => {
            let doc = OdtDocument::open(path)?;
            Ok(OfficeDocument::from_blocks(doc.blocks().to_vec()))
        }
        other => Err(Error::InvalidInput(format!(
            "office::open does not support .{other} (use docx or odt; PDF → pdf::PdfReader)"
        ))),
    }
}
