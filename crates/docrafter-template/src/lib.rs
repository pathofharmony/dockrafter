//! Template variables and report builders (Phase 1.4).

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod context;
mod report;
mod substitute;

pub use context::Context;
pub use report::ReportBuilder;
pub use substitute::{apply_context, substitute};
