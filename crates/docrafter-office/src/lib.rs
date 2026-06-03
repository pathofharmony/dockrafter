//! Shared flow elements for office formats (DOCX, ODT / LibreOffice).

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod blocks;
mod document;
mod elements;

pub use blocks::OfficeBlock;
pub use document::OfficeDocument;
pub use elements::{Image, List, Paragraph, Table, TextRun};
