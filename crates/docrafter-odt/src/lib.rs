//! ODT generation and reading (OpenDocument / LibreOffice).

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod document;
mod package;
mod read;
mod styles;
mod write;

pub use docrafter_office::OfficeBlock as OdtBlock;
pub use docrafter_office::{Image, List, Paragraph, Table, TextRun};
pub use document::OdtDocument;
