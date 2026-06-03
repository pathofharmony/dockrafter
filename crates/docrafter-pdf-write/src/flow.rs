//! Flow document elements (platypus-style, reportlab-compatible).

use docrafter_core::Length;

pub use docrafter_office::{Image, List, Paragraph, Table, TextRun};

/// Vertical whitespace in the flow.
#[derive(Debug, Clone, Copy)]
pub struct Spacer {
    height: f32,
}

impl Spacer {
    /// Fixed height spacer.
    #[must_use]
    pub fn new(height: Length) -> Self {
        Self {
            height: height.as_pt(),
        }
    }

    /// Height in points.
    #[must_use]
    pub const fn height_pt(self) -> f32 {
        self.height
    }
}

/// Start a new page.
#[derive(Debug, Clone, Copy, Default)]
pub struct PageBreak;

/// Any element that can be pushed into a [`crate::PdfDocument`].
#[derive(Debug, Clone)]
pub enum FlowItem {
    /// Text block (multi-run, like python-docx).
    Paragraph(Paragraph),
    /// Vertical space.
    Spacer(Spacer),
    /// Page boundary.
    PageBreak(PageBreak),
    /// Bitmap.
    Image(Image),
    /// Table.
    Table(Table),
    /// Numbered list.
    List(List),
}

impl From<Paragraph> for FlowItem {
    fn from(value: Paragraph) -> Self {
        FlowItem::Paragraph(value)
    }
}

impl From<Spacer> for FlowItem {
    fn from(value: Spacer) -> Self {
        FlowItem::Spacer(value)
    }
}

impl From<PageBreak> for FlowItem {
    fn from(value: PageBreak) -> Self {
        FlowItem::PageBreak(value)
    }
}

impl From<Image> for FlowItem {
    fn from(value: Image) -> Self {
        FlowItem::Image(value)
    }
}

impl From<Table> for FlowItem {
    fn from(value: Table) -> Self {
        FlowItem::Table(value)
    }
}

impl From<List> for FlowItem {
    fn from(value: List) -> Self {
        FlowItem::List(value)
    }
}
