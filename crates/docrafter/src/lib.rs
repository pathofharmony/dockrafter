//! **docrafter** — unified document library for PDF, DOCX, and ODT.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use docrafter::prelude::*;
//!
//! # fn main() -> docrafter::Result<()> {
//! let mut doc = PdfDocument::new();
//! doc.push(Paragraph::new("Hello, docrafter!"));
//! doc.save("hello.pdf")?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(clippy::multiple_crate_versions)] // OCR + PDF render dependency trees

pub mod docx;
pub mod export;
pub mod html;
pub mod odt;
pub mod office;
pub mod open_document;
pub mod pdf;
pub mod prelude;
pub mod template;

pub use open_document::OpenDocument;

pub use export::{
    export_bundle, export_bytes, export_bytes_with_pdf_options, export_save, export_save_auto,
    pdf_from_office, pdf_from_office_with_options, OutputFormat, PdfExportOptions,
};

pub use docrafter_core::{
    self, Alignment, Color, Error, Length, PageSize, Result, Style, TableStyle,
};
