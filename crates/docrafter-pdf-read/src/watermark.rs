//! Text watermark overlay on existing pages.

use docrafter_core::{Error, Result};
use lopdf::Document;

use crate::pages::validate_page_numbers;

/// Options for a diagonal text watermark.
#[derive(Debug, Clone)]
pub struct WatermarkOptions {
    /// Watermark text.
    pub text: String,
    /// Font size in points.
    pub font_size: f32,
    /// Gray level `0.0` (black) .. `1.0` (white).
    pub gray: f32,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        Self {
            text: "DRAFT".into(),
            font_size: 48.0,
            gray: 0.75,
        }
    }
}

/// Draw a text watermark on selected pages (`None` = all).
pub fn add_text_watermark(
    doc: &mut Document,
    page_numbers: Option<&[u32]>,
    options: &WatermarkOptions,
) -> Result<()> {
    if options.text.is_empty() {
        return Err(Error::Pdf("watermark text is empty".into()));
    }
    if let Some(nums) = page_numbers {
        validate_page_numbers(doc, nums)?;
    }
    for (num, page_id) in doc.get_pages() {
        if page_numbers.is_some_and(|nums| !nums.contains(&num)) {
            continue;
        }
        let (width, height) = page_media_box(doc, page_id)?;
        let font_key = first_font_name(doc, page_id)?;
        let content = build_watermark_stream(
            &options.text,
            options.font_size,
            options.gray,
            width,
            height,
            &font_key,
        );
        doc.add_page_contents(page_id, content)
            .map_err(|e| Error::Pdf(format!("watermark page {num}: {e}")))?;
    }
    Ok(())
}

fn page_media_box(doc: &Document, page_id: lopdf::ObjectId) -> Result<(f32, f32)> {
    let page = doc
        .get_dictionary(page_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    let box_array = page
        .get(b"MediaBox")
        .or_else(|_| page.get(b"CropBox"))
        .map_err(|e| Error::Pdf(e.to_string()))?;
    let arr = box_array
        .as_array()
        .map_err(|_| Error::Pdf("MediaBox is not an array".into()))?;
    if arr.len() < 4 {
        return Err(Error::Pdf("invalid MediaBox".into()));
    }
    let x0 = arr[0].as_f32().unwrap_or(0.0);
    let y0 = arr[1].as_f32().unwrap_or(0.0);
    let x1 = arr[2].as_f32().unwrap_or(612.0);
    let y1 = arr[3].as_f32().unwrap_or(792.0);
    Ok((x1 - x0, y1 - y0))
}

fn first_font_name(doc: &Document, page_id: lopdf::ObjectId) -> Result<String> {
    let fonts = doc
        .get_page_fonts(page_id)
        .map_err(|e| Error::Pdf(e.to_string()))?;
    let key = fonts
        .keys()
        .next()
        .ok_or_else(|| Error::Pdf("page has no fonts for watermark".into()))?;
    Ok(String::from_utf8_lossy(key).into_owned())
}

fn pdf_escape_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn build_watermark_stream(
    text: &str,
    font_size: f32,
    gray: f32,
    width: f32,
    height: f32,
    font_key: &str,
) -> Vec<u8> {
    let escaped = pdf_escape_text(text);
    let cx = width * 0.35;
    let cy = height * 0.5;
    format!(
        "q\n{gray} g\nBT\n/{font_key} {font_size} Tf\n0.7071 0.7071 -0.7071 0.7071 {cx} {cy} Tm\n({escaped}) Tj\nET\nQ\n",
        gray = gray,
        font_key = font_key,
        font_size = font_size,
        cx = cx,
        cy = cy,
        escaped = escaped,
    )
    .into_bytes()
}
