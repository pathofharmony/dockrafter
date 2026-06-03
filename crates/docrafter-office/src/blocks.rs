//! Document body blocks in flow order (format-agnostic).

use crate::elements::{Image, List, Paragraph, Table};

/// Top-level body element preserving document order.
#[derive(Debug, Clone)]
pub enum OfficeBlock {
    /// Text paragraph (one or more runs).
    Paragraph(Paragraph),
    /// Table with optional header row.
    Table(Table),
    /// Inline image.
    Image(Image),
    /// Numbered list.
    List(List),
}
