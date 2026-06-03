//! Table styling presets.

use crate::color::Color;

/// Visual style for PDF/DOCX tables.
#[derive(Debug, Clone, PartialEq)]
pub struct TableStyle {
    header_background: Option<Color>,
    header_bold: bool,
    border_width: f32,
    cell_padding: f32,
    font_size: f32,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            header_background: None,
            header_bold: true,
            border_width: 0.5,
            cell_padding: 6.0,
            font_size: 10.0,
        }
    }
}

impl TableStyle {
    /// Empty customizable style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Corporate-style preset (header row, light gray background).
    #[must_use]
    pub fn professional() -> Self {
        Self {
            header_background: Color::from_hex("#f1f5f9").ok(),
            header_bold: true,
            border_width: 0.75,
            cell_padding: 8.0,
            font_size: 10.0,
        }
    }

    /// Header row background color.
    #[must_use]
    pub fn header_bg(mut self, color: Color) -> Self {
        self.header_background = Some(color);
        self
    }

    /// Font size for table body cells (points).
    #[must_use]
    pub fn font_size(mut self, pt: f32) -> Self {
        self.font_size = pt;
        self
    }

    /// Inner cell padding (points).
    #[must_use]
    pub fn cell_padding(mut self, pt: f32) -> Self {
        self.cell_padding = pt;
        self
    }

    /// Resolved header background (light gray default).
    #[must_use]
    pub fn effective_header_bg(&self) -> Color {
        self.header_background.unwrap_or(Color::rgb(241, 245, 249))
    }

    /// Body font size in points.
    #[must_use]
    pub fn effective_font_size(&self) -> f32 {
        self.font_size
    }

    /// Whether header row uses bold text.
    #[must_use]
    pub const fn header_bold(&self) -> bool {
        self.header_bold
    }

    /// Border line width in points.
    #[must_use]
    pub const fn border_width(&self) -> f32 {
        self.border_width
    }

    /// Cell padding in points.
    #[must_use]
    pub const fn padding_pt(&self) -> f32 {
        self.cell_padding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn professional_has_header_bg() {
        let s = TableStyle::professional();
        assert!(s.header_background.is_some());
        assert!(s.header_bold());
    }
}
