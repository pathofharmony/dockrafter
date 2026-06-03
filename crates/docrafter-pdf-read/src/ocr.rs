//! OCR text recognition for scanned PDFs (pure Rust, in-repo engine).

use docrafter_core::{Error, Result};
use docrafter_ocr::{engine_available, recognize_rgba, OcrEngine};
use docrafter_pdf_render::render_all_pages_rgba;

/// Options for in-process OCR.
#[derive(Debug, Clone)]
pub struct OcrOptions {
    /// Reserved for future multilingual models (ocrs default is Latin-focused).
    pub lang: String,
    /// Render resolution in DPI for rasterization.
    pub dpi: u32,
}

impl Default for OcrOptions {
    fn default() -> Self {
        Self {
            lang: "eng".into(),
            dpi: 150,
        }
    }
}

/// Whether bundled OCR models are installed (`./scripts/fetch-ocr-models.sh`).
#[must_use]
pub fn ocr_engine_available() -> bool {
    engine_available()
}

/// Back-compat alias for [`ocr_engine_available`].
#[must_use]
pub fn ocr_tools_available() -> bool {
    ocr_engine_available()
}

/// Recognize text from an RGBA8 page bitmap.
pub fn ocr_image_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<String> {
    let _engine = OcrEngine::open()?;
    recognize_rgba(width, height, rgba)
}

/// Full OCR pipeline: PDF bytes → rendered pages → recognized text.
pub fn ocr_pdf_bytes(pdf_bytes: &[u8], options: &OcrOptions) -> Result<String> {
    if !engine_available() {
        return Err(Error::Pdf(
            "OCR models missing: run ./scripts/fetch-ocr-models.sh".into(),
        ));
    }
    let _engine = OcrEngine::open()?;
    let dpi = options.dpi.max(72) as f32;
    let pages = render_all_pages_rgba(pdf_bytes, dpi)?;
    let mut text = String::new();
    for page in pages {
        let page_text = recognize_rgba(page.width, page.height, &page.rgba)?;
        if !text.is_empty() && !page_text.is_empty() {
            text.push('\n');
        }
        text.push_str(&page_text);
    }
    Ok(text)
}

/// OCR from a file path (convenience).
pub fn ocr_pdf_file(pdf_path: impl AsRef<std::path::Path>, options: &OcrOptions) -> Result<String> {
    let bytes = std::fs::read(pdf_path.as_ref())
        .map_err(|e| Error::Pdf(format!("read PDF for OCR: {e}")))?;
    ocr_pdf_bytes(&bytes, options)
}
