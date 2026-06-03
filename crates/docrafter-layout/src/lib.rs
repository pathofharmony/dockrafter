//! Single-column flow layout (word wrap, spacing, page breaks).

#![deny(missing_docs)]

mod config;
mod engine;
mod input;
mod table_layout;
mod wrap;
mod wrap_runs;

pub use config::{LayoutConfig, LayoutMargins};
pub use engine::{layout_flow, LayoutPage, LayoutPlacement};
pub use input::{
    FlowInput, ImageInput, ListInput, ParagraphInput, SpacerInput, TableInput, TextRunInput,
};
pub use table_layout::{compute_column_widths, paginate_table, slice_height, table_row_height};
pub use wrap::{measure_text_width, wrap_text};
pub use wrap_runs::{align_styled_line, effective_run_style, wrap_styled_runs};
