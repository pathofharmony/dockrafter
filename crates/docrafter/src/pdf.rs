//! PDF read and write API.

#[cfg(feature = "ocr")]
pub use docrafter_ocr::{models_dir, OcrEngine};
pub use docrafter_pdf_read::{
    add_bookmark, add_text_field, add_text_watermark, add_uri_link, compress_streams, copy_pages,
    encrypt_document, extract_pages, is_encrypted, list_text_fields, list_uri_links, load_decrypt,
    load_mem_decrypt, merge_documents, rebuild_outline, replace_text_all, replace_text_on_page,
    rotate_pages, split_pages, validate_page_numbers, EncryptOptions, PdfLink, PdfMetadata,
    PdfReader, PdfRect, PdfTextField, Rotate, TextExtractMode, WatermarkOptions,
};
#[cfg(feature = "ocr")]
pub use docrafter_pdf_read::{
    ocr_engine_available, ocr_pdf_bytes, ocr_tools_available, OcrOptions,
};
#[cfg(feature = "ocr")]
pub use docrafter_pdf_render::{
    pdf_page_count, render_all_pages_rgba, render_page_rgba, RenderedPage,
};
pub use docrafter_pdf_write::{
    expand_page_template, FlowItem, Image, List, PageBreak, PageHeaderFooter, Paragraph,
    PdfDocument, Spacer, Table, TextRun,
};
