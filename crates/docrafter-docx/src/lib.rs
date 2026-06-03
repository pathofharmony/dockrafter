//! DOCX generation and reading (WordprocessingML / OOXML).

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod comments;
mod document;
mod numbering;
mod package;
mod read;
mod styles;
mod write;

pub use comments::DocxComment;
pub use docrafter_office::OfficeBlock as DocxBlock;
pub use docrafter_office::{Image, List, Paragraph, Table, TextRun};
pub use document::DocxDocument;
