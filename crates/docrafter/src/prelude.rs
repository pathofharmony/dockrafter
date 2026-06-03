//! Ergonomic imports for typical applications.

pub use crate::docx::{
    DocxBlock, DocxComment, DocxDocument, Image as DocxImage, List as DocxList,
    Paragraph as DocxParagraph, Table as DocxTable,
};
pub use crate::export::{
    export_bundle, export_bytes, export_save, export_save_auto, OutputFormat, PdfExportOptions,
};
pub use crate::html::{from_html, html_to_office};
pub use crate::odt::{
    Image as OdtImage, List as OdtList, OdtBlock, OdtDocument, Paragraph as OdtParagraph,
    Table as OdtTable,
};
pub use crate::office::{List, OfficeBlock, OfficeDocument, TextRun};
pub use crate::open_document::OpenDocument;
pub use crate::pdf::{
    FlowItem, Image, PageBreak, PageHeaderFooter, Paragraph, PdfDocument, PdfLink, PdfReader,
    PdfRect, PdfTextField, Spacer, Table,
};
pub use crate::template::{apply_context, substitute, Context, ReportBuilder};
pub use docrafter_core::{
    Alignment, Color, Error, Length, PageSize, Result, Style, TableStyle, VerticalAlign,
};
