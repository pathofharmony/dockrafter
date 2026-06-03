//! Pure-Rust OCR for docrafter (RTen detection + recognition, no Tesseract binary).

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![allow(clippy::multiple_crate_versions)] // ocrs / rten dependency tree

mod engine;
mod models;

pub use engine::{recognize_rgba, OcrEngine};
pub use models::{engine_available, models_dir};
