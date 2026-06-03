//! Color representation with hex parsing.

use crate::error::{Error, Result};

/// sRGB color with 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    /// Create from explicit sRGB channels.
    #[must_use]
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse `#RRGGBB` or `RRGGBB` hex notation.
    pub fn from_hex(hex: &str) -> Result<Self> {
        let s = hex.trim().strip_prefix('#').unwrap_or(hex);
        if s.len() != 6 {
            return Err(Error::InvalidInput(format!(
                "hex color must be 6 digits, got {s:?}"
            )));
        }
        let r = u8::from_str_radix(&s[0..2], 16)
            .map_err(|_| Error::InvalidInput(format!("invalid hex color: {hex}")))?;
        let g = u8::from_str_radix(&s[2..4], 16)
            .map_err(|_| Error::InvalidInput(format!("invalid hex color: {hex}")))?;
        let b = u8::from_str_radix(&s[4..6], 16)
            .map_err(|_| Error::InvalidInput(format!("invalid hex color: {hex}")))?;
        Ok(Self { r, g, b })
    }

    /// Red channel `0..=255`.
    #[must_use]
    pub const fn r(self) -> u8 {
        self.r
    }

    /// Green channel `0..=255`.
    #[must_use]
    pub const fn g(self) -> u8 {
        self.g
    }

    /// Blue channel `0..=255`.
    #[must_use]
    pub const fn b(self) -> u8 {
        self.b
    }

    /// PDF graphics operators use `0.0..=1.0` floats.
    #[must_use]
    pub fn as_pdf_rgb(self) -> (f32, f32, f32) {
        (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        )
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_with_hash() {
        let c = Color::from_hex("#1e40af").unwrap();
        assert_eq!(c, Color::rgb(0x1e, 0x40, 0xaf));
    }

    #[test]
    fn parses_hex_without_hash() {
        assert_eq!(Color::from_hex("ff00ff").unwrap(), Color::rgb(255, 0, 255));
    }

    #[test]
    fn rejects_short_hex() {
        assert!(Color::from_hex("#fff").is_err());
    }

    #[test]
    fn pdf_rgb_scale() {
        let (r, g, b) = Color::rgb(255, 128, 0).as_pdf_rgb();
        assert!((r - 1.0).abs() < f32::EPSILON);
        assert!((g - 128.0 / 255.0).abs() < 0.01);
        assert!(b.abs() < f32::EPSILON);
    }
}
