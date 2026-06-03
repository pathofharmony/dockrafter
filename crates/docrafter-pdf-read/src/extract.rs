//! Text extraction (pypdf-style) with optional OCR fallback.

use docrafter_core::{Error, Result};
use lopdf::Document;

#[cfg(feature = "ocr")]
use crate::ocr::{ocr_engine_available, ocr_pdf_bytes};

/// How to obtain text from a PDF.
#[derive(Debug, Clone, Default)]
pub enum TextExtractMode {
    /// Use embedded font / ToUnicode maps only (fast, pypdf-style).
    #[default]
    Embedded,
    /// Run OCR on rendered page images (requires feature `ocr`).
    #[cfg(feature = "ocr")]
    Ocr(crate::ocr::OcrOptions),
    /// Try embedded extraction; if too short, run OCR (requires feature `ocr`).
    #[cfg(feature = "ocr")]
    Auto(crate::ocr::OcrOptions),
}

/// Extract plain text from selected pages (1-based page numbers).
pub fn extract_text(doc: &Document, page_numbers: &[u32]) -> Result<String> {
    doc.extract_text(page_numbers)
        .map_err(|e| Error::Pdf(format!("text extraction failed: {e}")))
}

/// Extract text from all pages in order.
pub fn extract_all_text(doc: &Document) -> Result<String> {
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    extract_text(doc, &pages)
}

/// Extract using embedded fonts, OCR, or auto-detect.
///
/// `pdf_bytes` is required for OCR modes (works with [`PdfReader::from_bytes`](crate::PdfReader::from_bytes)).
#[cfg(feature = "ocr")]
pub fn extract_with_mode(
    doc: &Document,
    pdf_bytes: &[u8],
    mode: TextExtractMode,
) -> Result<String> {
    match mode {
        TextExtractMode::Embedded => extract_all_text(doc),
        TextExtractMode::Ocr(options) => ocr_pdf_bytes(pdf_bytes, &options),
        TextExtractMode::Auto(options) => {
            let embedded = extract_all_text(doc).unwrap_or_default();
            let trimmed = embedded.split_whitespace().collect::<String>();
            if trimmed.len() >= 20 {
                return Ok(embedded);
            }
            if !ocr_engine_available() {
                if embedded.is_empty() {
                    return Err(Error::Pdf(
                        "no embedded text and OCR models not installed (run ./scripts/fetch-ocr-models.sh)".into(),
                    ));
                }
                return Ok(embedded);
            }
            let ocr = ocr_pdf_bytes(pdf_bytes, &options)?;
            if ocr.len() > embedded.len() {
                Ok(ocr)
            } else {
                Ok(embedded)
            }
        }
    }
}

#[cfg(not(feature = "ocr"))]
pub fn extract_with_mode(
    doc: &Document,
    _pdf_bytes: &[u8],
    mode: TextExtractMode,
) -> Result<String> {
    match mode {
        TextExtractMode::Embedded => extract_all_text(doc),
    }
}
