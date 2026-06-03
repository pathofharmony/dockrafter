//! Office document body elements (paragraphs, tables, images, lists).

use std::path::Path;

use docrafter_core::{Alignment, Error, Result, Style, TableStyle};

/// A text run inside a paragraph.
#[derive(Debug, Clone)]
pub struct TextRun {
    text: String,
    style: Style,
}

impl TextRun {
    /// Run with default character formatting.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::new(),
        }
    }

    /// Character style for this run.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Run text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Resolved run style.
    #[must_use]
    pub fn resolved_style(&self) -> &Style {
        &self.style
    }
}

/// A paragraph (paragraph properties + runs).
#[derive(Debug, Clone)]
pub struct Paragraph {
    paragraph_style: Style,
    runs: Vec<TextRun>,
}

impl Paragraph {
    /// Build from parsed runs (readers in DOCX/ODT backends).
    #[must_use]
    pub fn from_runs(paragraph_style: Style, runs: Vec<TextRun>) -> Self {
        Self {
            paragraph_style,
            runs,
        }
    }

    /// Create a paragraph with a single run.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            paragraph_style: Style::new(),
            runs: vec![TextRun::new(text)],
        }
    }

    /// Append another run (mixed formatting within one paragraph).
    #[must_use]
    pub fn run(mut self, text: impl Into<String>, style: Style) -> Self {
        self.runs.push(TextRun::new(text).style(style));
        self
    }

    /// Set paragraph-level style (alignment, preset); applied to a single run as well.
    #[must_use]
    pub fn style(mut self, style: Style) -> Self {
        self.paragraph_style = style.clone();
        if self.runs.len() == 1 {
            self.runs[0].style = style;
        }
        self
    }

    /// Set horizontal alignment on the paragraph.
    #[must_use]
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.paragraph_style = self.paragraph_style.align(alignment);
        self
    }

    /// Full paragraph text (all runs concatenated).
    #[must_use]
    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// Paragraph-level properties (alignment, preset).
    #[must_use]
    pub fn paragraph_style(&self) -> &Style {
        &self.paragraph_style
    }

    /// Text runs in order.
    #[must_use]
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }

    /// Back-compat: style of the first run or paragraph style.
    #[must_use]
    pub fn resolved_style(&self) -> &Style {
        self.runs
            .first()
            .map_or(&self.paragraph_style, |r| &r.style)
    }
}

/// A table.
#[derive(Debug, Clone)]
pub struct Table {
    /// Header row labels.
    pub columns: Vec<String>,
    /// Data rows.
    pub rows: Vec<Vec<String>>,
    /// Visual style.
    pub style: TableStyle,
    /// Explicit column widths in points (PDF layout; empty = auto).
    pub column_widths: Vec<f32>,
    /// Repeat header row when the table spans a page break (PDF).
    pub repeat_header_on_new_page: bool,
}

impl Table {
    /// Empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            style: TableStyle::default(),
            column_widths: Vec::new(),
            repeat_header_on_new_page: false,
        }
    }

    /// Professional preset (header row styling; repeats header on new PDF pages).
    #[must_use]
    pub fn professional() -> Self {
        Self {
            style: TableStyle::professional(),
            repeat_header_on_new_page: true,
            ..Self::new()
        }
    }

    /// Set column headers.
    #[must_use]
    pub fn columns(mut self, cols: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.columns = cols.into_iter().map(Into::into).collect();
        self
    }

    /// Append a data row.
    #[must_use]
    pub fn row(mut self, cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    /// Set table style.
    #[must_use]
    pub fn style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// Repeat the header row when the table spans a page break (PDF).
    #[must_use]
    pub fn repeat_header_on_new_page(mut self, repeat: bool) -> Self {
        self.repeat_header_on_new_page = repeat;
        self
    }

    /// Explicit column widths in points (PDF layout).
    #[must_use]
    pub fn column_widths(mut self, widths: impl IntoIterator<Item = f32>) -> Self {
        self.column_widths = widths.into_iter().collect();
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

/// Raster image in the document body.
#[derive(Debug, Clone)]
pub struct Image {
    data: Vec<u8>,
    width_pt: Option<f32>,
    height_pt: Option<f32>,
}

impl Image {
    /// Load PNG or JPEG from disk.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let data = std::fs::read(path).map_err(|source| Error::io(path, source))?;
        Ok(Self {
            data,
            width_pt: None,
            height_pt: None,
        })
    }

    /// Create from encoded bytes.
    #[must_use]
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            width_pt: None,
            height_pt: None,
        }
    }

    /// Display size in points.
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width_pt = Some(width);
        self.height_pt = Some(height);
        self
    }

    /// Raw image bytes.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Optional width in points.
    #[must_use]
    pub fn width_pt(&self) -> Option<f32> {
        self.width_pt
    }

    /// Optional height in points.
    #[must_use]
    pub fn height_pt(&self) -> Option<f32> {
        self.height_pt
    }
}

/// Ordered list.
#[derive(Debug, Clone)]
pub struct List {
    items: Vec<String>,
}

impl List {
    /// Empty list.
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append a list item.
    #[must_use]
    pub fn item(mut self, text: impl Into<String>) -> Self {
        self.items.push(text.into());
        self
    }

    /// Append while parsing consecutive list paragraphs.
    pub fn push_item(&mut self, text: impl Into<String>) {
        self.items.push(text.into());
    }

    /// List item texts.
    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.items
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}
