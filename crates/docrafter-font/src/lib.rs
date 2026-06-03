//! TrueType parsing, text measurement, and PDF embedding.

#![deny(missing_docs)]

mod embed;
mod metrics;

pub use embed::{embed_truetype, EmbeddedFont, FontBundle, ParsedFace, FONT_BOLD, FONT_REGULAR};
pub use metrics::{measure_text, TextMeasurer};

/// DejaVu Sans regular (full TTF, embedded in PDFs by default).
pub fn dejavu_sans_regular_bytes() -> &'static [u8] {
    include_bytes!("../fonts/DejaVuSans.ttf")
}

/// DejaVu Sans bold.
pub fn dejavu_sans_bold_bytes() -> &'static [u8] {
    include_bytes!("../fonts/DejaVuSans-Bold.ttf")
}
