//! Bundled RTen model files (`text-detection.rten`, `text-recognition.rten`).

use std::path::{Path, PathBuf};

/// Directory with OCR models inside this crate (`crates/docrafter-ocr/models/`).
#[must_use]
pub fn models_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("models")
}

/// Whether both model files are present (run `./scripts/fetch-ocr-models.sh` once).
#[must_use]
pub fn engine_available() -> bool {
    let dir = models_dir();
    dir.join("text-detection.rten").is_file() && dir.join("text-recognition.rten").is_file()
}

pub(crate) fn detection_model_path() -> PathBuf {
    models_dir().join("text-detection.rten")
}

pub(crate) fn recognition_model_path() -> PathBuf {
    models_dir().join("text-recognition.rten")
}
