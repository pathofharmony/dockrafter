//! Low-level PDF generation for docrafter.

#![deny(missing_docs)]

mod document;
mod flow;
mod header_footer;
mod render;

pub use document::PdfDocument;
pub use flow::{FlowItem, Image, List, PageBreak, Paragraph, Spacer, Table, TextRun};
pub use header_footer::{expand_page_template, PageHeaderFooter};
pub use render::PdfRenderer;
