//! Text width measurement from TrueType advances.

use crate::embed::ParsedFace;

/// Measures string widths using glyph advances.
pub trait TextMeasurer {
    /// Total width in points for `text` at `font_size` pt.
    fn measure(&self, text: &str, font_size: f32, bold: bool) -> f32;
}

/// Measure using a parsed face.
#[must_use]
pub fn measure_text(face: &ParsedFace, text: &str, font_size: f32) -> f32 {
    let scale = font_size / face.units_per_em;
    text.chars()
        .map(|ch| face.advance_width(ch).unwrap_or(face.units_per_em / 2.0) * scale)
        .sum()
}

impl TextMeasurer for FontBundle {
    fn measure(&self, text: &str, font_size: f32, bold: bool) -> f32 {
        let face = if bold {
            &self.bold.parsed
        } else {
            &self.regular.parsed
        };
        measure_text(face, text, font_size)
    }
}

use crate::embed::FontBundle;
