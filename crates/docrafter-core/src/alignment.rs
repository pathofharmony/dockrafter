//! Text and block alignment.

/// Horizontal alignment for paragraphs, table cells, and images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[non_exhaustive]
pub enum Alignment {
    /// Align to the start edge (left in LTR).
    #[default]
    Start,
    /// Center horizontally.
    Center,
    /// Align to the end edge (right in LTR).
    End,
    /// Justify text to both margins.
    Justify,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_start() {
        assert_eq!(Alignment::default(), Alignment::Start);
    }
}
