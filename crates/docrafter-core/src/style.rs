//! Fluent paragraph and run styling.

use crate::alignment::Alignment;
use crate::color::Color;
use crate::error::Result;
use crate::length::Length;
use crate::vertical_align::VerticalAlign;

/// CSS-like style descriptor (immutable builder pattern).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    vertical_align: Option<VerticalAlign>,
    font_size: Option<f32>,
    color: Option<Color>,
    align: Option<Alignment>,
    line_height: Option<Length>,
}

impl Style {
    /// Empty style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Preset for top-level headings (18 pt, bold).
    #[must_use]
    pub fn heading1() -> Self {
        Self::new().bold().font_size(18.0)
    }

    /// Preset for secondary headings (14 pt, bold).
    #[must_use]
    pub fn heading2() -> Self {
        Self::new().bold().font_size(14.0)
    }

    /// Enable bold.
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Enable italic.
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Enable underline.
    #[must_use]
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Enable strikethrough.
    #[must_use]
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Superscript (smaller raised run).
    #[must_use]
    pub fn superscript(mut self) -> Self {
        self.vertical_align = Some(VerticalAlign::Superscript);
        self
    }

    /// Subscript (smaller lowered run).
    #[must_use]
    pub fn subscript(mut self) -> Self {
        self.vertical_align = Some(VerticalAlign::Subscript);
        self
    }

    /// Font size in points.
    #[must_use]
    pub fn font_size(mut self, pt: f32) -> Self {
        self.font_size = Some(pt);
        self
    }

    /// Text color from hex string (`#RRGGBB`).
    pub fn color(mut self, hex: &str) -> Result<Self> {
        self.color = Some(Color::from_hex(hex)?);
        Ok(self)
    }

    /// Text color from [`Color`].
    #[must_use]
    pub fn color_value(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Paragraph alignment.
    #[must_use]
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.align = Some(alignment);
        self
    }

    /// Line height.
    #[must_use]
    pub fn line_height(mut self, height: Length) -> Self {
        self.line_height = Some(height);
        self
    }

    /// Whether bold is set.
    #[must_use]
    pub const fn is_bold(&self) -> bool {
        self.bold
    }

    /// Whether italic is set.
    #[must_use]
    pub const fn is_italic(&self) -> bool {
        self.italic
    }

    /// Whether underline is set.
    #[must_use]
    pub const fn is_underline(&self) -> bool {
        self.underline
    }

    /// Whether strikethrough is set.
    #[must_use]
    pub const fn is_strikethrough(&self) -> bool {
        self.strikethrough
    }

    /// Vertical alignment of the run.
    #[must_use]
    pub fn vertical_align(&self) -> VerticalAlign {
        match self.vertical_align {
            Some(v) => v,
            None => VerticalAlign::Baseline,
        }
    }

    /// Whether this run is superscript.
    #[must_use]
    pub const fn is_superscript(&self) -> bool {
        matches!(self.vertical_align, Some(VerticalAlign::Superscript))
    }

    /// Whether this run is subscript.
    #[must_use]
    pub const fn is_subscript(&self) -> bool {
        matches!(self.vertical_align, Some(VerticalAlign::Subscript))
    }

    /// Effective font size in points (default 12).
    #[must_use]
    pub fn effective_font_size(&self) -> f32 {
        let base = self.font_size.unwrap_or(12.0);
        match self.vertical_align {
            Some(VerticalAlign::Superscript) | Some(VerticalAlign::Subscript) => base * 0.65,
            _ => base,
        }
    }

    /// Baseline shift in points (positive = up) for superscript/subscript layout.
    #[must_use]
    pub fn baseline_shift_pt(&self) -> f32 {
        let size = self.font_size.unwrap_or(12.0);
        match self.vertical_align {
            Some(VerticalAlign::Superscript) => size * 0.35,
            Some(VerticalAlign::Subscript) => -size * 0.15,
            _ => 0.0,
        }
    }

    /// Resolved text color.
    #[must_use]
    pub fn effective_color(&self) -> Color {
        self.color.unwrap_or(Color::rgb(0, 0, 0))
    }

    /// Resolved alignment.
    #[must_use]
    pub fn effective_align(&self) -> Alignment {
        self.align.unwrap_or_default()
    }

    /// Resolved line height in points (default: 1.35× font size).
    #[must_use]
    pub fn effective_line_height(&self) -> f32 {
        self.line_height
            .map(Length::as_pt)
            .unwrap_or_else(|| self.effective_font_size() * 1.35)
    }

    /// Validate numeric fields.
    pub fn validate(self) -> Result<Self> {
        if let Some(size) = self.font_size {
            if !size.is_finite() || size <= 0.0 {
                return Err(crate::error::Error::InvalidInput(format!(
                    "font size must be positive and finite, got {size}"
                )));
            }
        }
        if let Some(lh) = self.line_height {
            lh.validate()?;
            if lh.as_pt() == 0.0 {
                return Err(crate::error::Error::InvalidInput(
                    "line height must be greater than zero".into(),
                ));
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading1_preset() {
        let s = Style::heading1();
        assert!(s.is_bold());
        assert_eq!(s.effective_font_size(), 18.0);
    }

    #[test]
    fn fluent_color_chain() {
        let s = Style::new().color("#1e40af").unwrap();
        assert_eq!(s.effective_color(), Color::rgb(0x1e, 0x40, 0xaf));
    }

    #[test]
    fn validate_rejects_zero_font() {
        assert!(Style::new().font_size(0.0).validate().is_err());
    }
}
