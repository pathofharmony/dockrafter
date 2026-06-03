//! Shared CLI helpers.

use std::path::Path;

use docrafter::open_document::OpenDocument;
use docrafter::pdf::{PdfReader, Rotate};
use docrafter::Result;

/// Open a PDF, optionally with password.
pub fn open_pdf(path: &Path, password: Option<&str>) -> Result<PdfReader> {
    let doc = OpenDocument::open_with_password(path, password)?;
    match doc {
        OpenDocument::Pdf(reader) => Ok(*reader),
        OpenDocument::Office(_) => Err(docrafter_core::Error::InvalidInput(
            "expected a PDF file".into(),
        )),
    }
}

/// Parse comma-separated 1-based page numbers (`1,3,5`).
pub fn parse_page_list(spec: &str) -> Result<Vec<u32>> {
    let mut pages = Vec::new();
    for part in spec.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let n: u32 = part
            .parse()
            .map_err(|_| docrafter_core::Error::InvalidInput(format!("invalid page: {part}")))?;
        if n == 0 {
            return Err(docrafter_core::Error::InvalidInput(
                "page numbers are 1-based".into(),
            ));
        }
        pages.push(n);
    }
    if pages.is_empty() {
        return Err(docrafter_core::Error::InvalidInput(
            "empty page list".into(),
        ));
    }
    Ok(pages)
}

/// Map degrees (90, 180, 270) to [`Rotate`].
pub fn parse_rotation(degrees: u16) -> Result<Rotate> {
    match degrees {
        90 => Ok(Rotate::Clockwise90),
        180 => Ok(Rotate::Clockwise180),
        270 => Ok(Rotate::Clockwise270),
        other => Err(docrafter_core::Error::InvalidInput(format!(
            "rotation must be 90, 180, or 270 (got {other})"
        ))),
    }
}
