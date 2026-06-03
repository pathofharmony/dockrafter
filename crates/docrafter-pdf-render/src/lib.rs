//! Rasterize PDF pages to bitmaps in memory (no Poppler / `pdftoppm`).

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(clippy::multiple_crate_versions)] // zenpdf / hayro dependency tree

use docrafter_core::{Error, Result};
use zenpdf::{page_count, render_page, RenderBounds};

/// One rendered page (straight-alpha sRGB RGBA8).
#[derive(Debug, Clone)]
pub struct RenderedPage {
    /// Zero-based page index.
    pub index: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row-major RGBA8 pixels (`width * height * 4` bytes).
    pub rgba: Vec<u8>,
}

/// Number of pages in a PDF byte stream.
pub fn pdf_page_count(pdf_bytes: &[u8]) -> Result<u32> {
    page_count(pdf_bytes).map_err(map_render_err)
}

/// Render a single page at the given DPI.
pub fn render_page_rgba(pdf_bytes: &[u8], page_index: u32, dpi: f32) -> Result<RenderedPage> {
    let page =
        render_page(pdf_bytes, page_index, &RenderBounds::Dpi(dpi)).map_err(map_render_err)?;
    let width = page.buffer.width();
    let height = page.buffer.height();
    Ok(RenderedPage {
        index: page.index,
        width,
        height,
        rgba: page.buffer.copy_to_contiguous_bytes(),
    })
}

/// Render every page in order.
pub fn render_all_pages_rgba(pdf_bytes: &[u8], dpi: f32) -> Result<Vec<RenderedPage>> {
    let count = pdf_page_count(pdf_bytes)?;
    let mut pages = Vec::with_capacity(count as usize);
    for index in 0..count {
        pages.push(render_page_rgba(pdf_bytes, index, dpi)?);
    }
    Ok(pages)
}

fn map_render_err(e: zenpdf::PdfError) -> Error {
    Error::Pdf(format!("PDF render: {e}"))
}
