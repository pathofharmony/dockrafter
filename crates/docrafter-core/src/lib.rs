//! Core primitives shared across PDF, DOCX, and template layers.

#![deny(missing_docs)]

pub mod alignment;
pub mod color;
pub mod error;
pub mod length;
pub mod page_size;
pub mod prelude;
pub mod style;
pub mod table;
pub mod vertical_align;

pub use alignment::Alignment;
pub use color::Color;
pub use error::{Error, Result};
pub use length::Length;
pub use page_size::PageSize;
pub use style::Style;
pub use table::TableStyle;
pub use vertical_align::VerticalAlign;
