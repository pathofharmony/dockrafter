//! Running page header and footer templates.

use docrafter_core::Style;

/// Header/footer text drawn on every PDF page.
///
/// Footer templates may include `{page}` and `{pages}` placeholders.
#[derive(Debug, Clone, Default)]
pub struct PageHeaderFooter {
    /// Optional header line (top margin area).
    pub header: Option<String>,
    /// Optional footer line (bottom margin area).
    pub footer: Option<String>,
    /// Font size in points for header/footer text.
    pub font_size: f32,
}

impl PageHeaderFooter {
    /// Empty header/footer (no drawing).
    #[must_use]
    pub fn new() -> Self {
        Self {
            font_size: 9.0,
            ..Self::default()
        }
    }

    /// Set header text.
    #[must_use]
    pub fn header(mut self, text: impl Into<String>) -> Self {
        self.header = Some(text.into());
        self
    }

    /// Set footer text (supports `{page}` / `{pages}`).
    #[must_use]
    pub fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer = Some(text.into());
        self
    }

    /// Footer showing `Page {page} of {pages}`.
    #[must_use]
    pub fn page_numbers(mut self) -> Self {
        self.footer = Some("Page {page} of {pages}".into());
        self
    }

    /// Whether any header or footer is configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.header.is_none() && self.footer.is_none()
    }

    /// Style used when drawing header/footer lines.
    #[must_use]
    pub fn draw_style(&self) -> Style {
        Style::new().font_size(self.font_size)
    }
}

/// Expand `{page}` and `{pages}` in a footer template.
#[must_use]
pub fn expand_page_template(template: &str, page: usize, pages: usize) -> String {
    template
        .replace("{page}", &page.to_string())
        .replace("{pages}", &pages.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_placeholders() {
        assert_eq!(
            expand_page_template("Page {page} of {pages}", 2, 5),
            "Page 2 of 5"
        );
    }
}
