//! Standard page dimensions.

use crate::length::Length;

/// Common paper sizes (media box in points).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    width: Length,
    height: Length,
}

impl PageSize {
    /// ISO A4 (210 × 297 mm).
    #[must_use]
    pub fn a4() -> Self {
        Self {
            width: Length::mm(210.0),
            height: Length::mm(297.0),
        }
    }

    /// US Letter (8.5 × 11 in).
    #[must_use]
    pub fn letter() -> Self {
        Self {
            width: Length::inch(8.5),
            height: Length::inch(11.0),
        }
    }

    /// Custom page size.
    #[must_use]
    pub fn custom(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    /// Page width.
    #[must_use]
    pub const fn width(self) -> Length {
        self.width
    }

    /// Page height.
    #[must_use]
    pub const fn height(self) -> Length {
        self.height
    }

    /// Media box as `(x_min, y_min, x_max, y_max)` in points.
    #[must_use]
    pub fn media_box(self) -> (f32, f32, f32, f32) {
        (0.0, 0.0, self.width.as_pt(), self.height.as_pt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_dimensions_match_iso() {
        let a4 = PageSize::a4();
        assert!((a4.width().as_mm() - 210.0).abs() < 0.1);
        assert!((a4.height().as_mm() - 297.0).abs() < 0.1);
    }

    #[test]
    fn letter_media_box_origin_is_zero() {
        let (x0, y0, _, _) = PageSize::letter().media_box();
        assert_eq!(x0, 0.0);
        assert_eq!(y0, 0.0);
    }
}
