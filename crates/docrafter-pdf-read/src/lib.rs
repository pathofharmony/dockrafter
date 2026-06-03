//! PDF reading and merging (pypdf-style operations).

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(clippy::multiple_crate_versions)] // zenpdf / ocrs via pdf-render and docrafter-ocr

mod annotations;
mod bookmarks;
mod crypto;
mod edit;
mod encrypt_save;
mod extract;
mod forms;
mod merge;
mod metadata;
#[cfg(feature = "ocr")]
mod ocr;
mod pages;
mod reader;
mod replace_cid;
mod watermark;

pub use annotations::{add_uri_link, list_uri_links, PdfLink, PdfRect};
pub use bookmarks::{add_bookmark, rebuild_outline};
pub use crypto::{is_encrypted, load_decrypt, load_mem_decrypt};
pub use edit::{compress_streams, replace_text_all, replace_text_on_page};
pub use encrypt_save::{encrypt_document, EncryptOptions};
pub use extract::{extract_all_text, extract_text, extract_with_mode, TextExtractMode};
pub use forms::{add_text_field, list_text_fields, PdfTextField};
pub use merge::merge_documents;
pub use metadata::PdfMetadata;
#[cfg(feature = "ocr")]
pub use ocr::{
    ocr_engine_available, ocr_image_rgba, ocr_pdf_bytes, ocr_pdf_file, ocr_tools_available,
    OcrOptions,
};
pub use pages::{
    copy_pages, extract_pages, rotate_pages, split_pages, validate_page_numbers, Rotate,
};
pub use reader::PdfReader;
pub use watermark::{add_text_watermark, WatermarkOptions};
