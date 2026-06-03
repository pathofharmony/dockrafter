//! Physical lengths with explicit units.

use crate::error::{Error, Result};

/// A length value stored internally as points (1 pt = 1/72 inch).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Length(f32);

impl Length {
    /// Points per inch.
    pub const PT_PER_INCH: f32 = 72.0;
    /// Points per millimeter.
    pub const PT_PER_MM: f32 = Self::PT_PER_INCH / 25.4;
    /// Points per centimeter.
    pub const PT_PER_CM: f32 = Self::PT_PER_MM * 10.0;

    /// Create a length from points.
    #[must_use]
    pub const fn pt(value: f32) -> Self {
        Self(value)
    }

    /// Create a length from millimeters.
    #[must_use]
    pub const fn mm(value: f32) -> Self {
        Self(value * Self::PT_PER_MM)
    }

    /// Create a length from centimeters.
    #[must_use]
    pub const fn cm(value: f32) -> Self {
        Self(value * Self::PT_PER_CM)
    }

    /// Create a length from inches.
    #[must_use]
    pub const fn inch(value: f32) -> Self {
        Self(value * Self::PT_PER_INCH)
    }

    /// Length in points (PDF native unit).
    #[must_use]
    pub const fn as_pt(self) -> f32 {
        self.0
    }

    /// Length in millimeters.
    #[must_use]
    pub fn as_mm(self) -> f32 {
        self.0 / Self::PT_PER_MM
    }

    /// Reject non-finite or negative lengths.
    pub fn validate(self) -> Result<Self> {
        if !self.0.is_finite() {
            return Err(Error::InvalidInput(format!(
                "length must be finite, got {self:?}"
            )));
        }
        if self.0 < 0.0 {
            return Err(Error::InvalidInput(format!(
                "length must be non-negative, got {self} pt",
                self = self.0
            )));
        }
        Ok(self)
    }
}

impl std::fmt::Display for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}pt", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_conversions_are_consistent() {
        let mm = Length::mm(25.4);
        let inch = Length::inch(1.0);
        assert!((mm.as_pt() - inch.as_pt()).abs() < 0.01);
    }

    #[test]
    fn validate_rejects_negative() {
        assert!(Length::pt(-1.0).validate().is_err());
    }

    #[test]
    fn validate_rejects_nan() {
        assert!(Length::pt(f32::NAN).validate().is_err());
    }
}
