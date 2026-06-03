//! Convert a small HTML subset into [`docrafter_office::OfficeDocument`].

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod parse;

pub use parse::html_to_office;

/// Alias for [`html_to_office`].
pub fn from_html(html: &str) -> docrafter_core::Result<docrafter_office::OfficeDocument> {
    html_to_office(html)
}
