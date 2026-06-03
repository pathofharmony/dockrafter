//! Rasterize docrafter-generated PDFs from bytes.

use docrafter_pdf_render::{pdf_page_count, render_page_rgba};
use docrafter_pdf_write::{Paragraph, PdfDocument};

#[test]
fn render_page_from_bytes() {
    let bytes = PdfDocument::new()
        .push(Paragraph::new("Render me"))
        .to_bytes()
        .unwrap();
    assert_eq!(pdf_page_count(&bytes).unwrap(), 1);
    let page = render_page_rgba(&bytes, 0, 150.0).unwrap();
    assert!(page.width > 0);
    assert!(page.height > 0);
    assert_eq!(page.rgba.len(), (page.width * page.height * 4) as usize);
}
