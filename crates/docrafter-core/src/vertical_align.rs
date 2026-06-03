//! Superscript / subscript (vertical alignment of runs).

/// Vertical placement of a text run relative to the line baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum VerticalAlign {
    /// Normal baseline.
    #[default]
    Baseline,
    /// Raised (exponent, footnote mark).
    Superscript,
    /// Lowered (chemical formula index).
    Subscript,
}
