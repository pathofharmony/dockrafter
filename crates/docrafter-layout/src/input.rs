//! Layout input types (format-agnostic).

use docrafter_core::{Style, TableStyle};

/// Character-level text fragment.
#[derive(Debug, Clone)]
pub struct TextRunInput {
    /// Run text.
    pub text: String,
    /// Character style (bold, color, size, italic).
    pub style: Style,
}

/// Paragraph block (paragraph properties + one or more runs).
#[derive(Debug, Clone)]
pub struct ParagraphInput {
    /// Paragraph-level style (alignment, presets).
    pub paragraph_style: Style,
    /// Text runs in order (python-docx / reportlab inline styles).
    pub runs: Vec<TextRunInput>,
}

impl ParagraphInput {
    /// Single-run paragraph (backward compatible).
    #[must_use]
    pub fn single(text: impl Into<String>, style: Style) -> Self {
        let text = text.into();
        Self {
            paragraph_style: style.clone(),
            runs: vec![TextRunInput { text, style }],
        }
    }
}

/// Vertical whitespace.
#[derive(Debug, Clone, Copy)]
pub struct SpacerInput {
    /// Height in points.
    pub height: f32,
}

/// Raster image placement.
#[derive(Debug, Clone)]
pub struct ImageInput {
    /// Raw encoded image bytes (PNG/JPEG).
    pub data: Vec<u8>,
    /// Target width in points (`None` = intrinsic at 1pt/px).
    pub width: Option<f32>,
    /// Target height in points.
    pub height: Option<f32>,
}

/// Numbered / bulleted list (expanded to paragraphs in layout).
#[derive(Debug, Clone)]
pub struct ListInput {
    /// Item texts.
    pub items: Vec<String>,
}

/// Table data.
#[derive(Debug, Clone)]
pub struct TableInput {
    /// Column titles (also used as first row when non-empty).
    pub columns: Vec<String>,
    /// Body rows.
    pub rows: Vec<Vec<String>>,
    /// Table style.
    pub style: TableStyle,
    /// Column widths in points (filled by layout if empty).
    pub column_widths: Vec<f32>,
    /// Repeat header row when the table continues on a new page.
    pub repeat_header_on_new_page: bool,
}

/// Flow element fed into the layout engine.
#[derive(Debug, Clone)]
pub enum FlowInput {
    /// Text paragraph (multi-run).
    Paragraph(ParagraphInput),
    /// Vertical gap.
    Spacer(SpacerInput),
    /// Force a new page.
    PageBreak,
    /// Bitmap image.
    Image(ImageInput),
    /// Tabular data.
    Table(TableInput),
    /// List (rendered as numbered paragraphs in PDF).
    List(ListInput),
}
