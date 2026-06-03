//! OCR pipeline: word detection → line grouping → character recognition.

use std::sync::OnceLock;

use docrafter_core::{Error, Result};
use ocrs::{ImageSource, OcrEngine as OcrsEngine, OcrEngineParams};
use rten::Model;

use crate::models::{detection_model_path, engine_available, recognition_model_path};

static SHARED: OnceLock<std::result::Result<OcrsEngine, String>> = OnceLock::new();

/// Loaded OCR engine (models from [`models_dir`](crate::models_dir)).
pub struct OcrEngine;

impl OcrEngine {
    /// Load models from `crates/docrafter-ocr/models/`.
    pub fn open() -> Result<Self> {
        if !engine_available() {
            return Err(Error::Pdf(
                "OCR models missing: run ./scripts/fetch-ocr-models.sh".into(),
            ));
        }
        let _ = shared_inner().map_err(Error::Pdf)?;
        Ok(Self)
    }

    /// Recognize text in an RGBA8 image.
    pub fn recognize_rgba(&self, width: u32, height: u32, rgba: &[u8]) -> Result<String> {
        recognize_rgba(width, height, rgba)
    }
}

/// Recognize text in an RGBA8 buffer (uses a process-wide cached engine).
pub fn recognize_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<String> {
    let engine = shared_inner().map_err(Error::Pdf)?;
    let rgb = rgba_to_rgb8(width, height, rgba)?;
    let source = ImageSource::from_bytes(rgb.as_raw(), (width, height))
        .map_err(|e| Error::Pdf(format!("OCR image: {e}")))?;
    let input = engine
        .prepare_input(source)
        .map_err(|e| Error::Pdf(format!("OCR prepare: {e}")))?;

    let word_rects = engine
        .detect_words(&input)
        .map_err(|e| Error::Pdf(format!("OCR detect: {e}")))?;
    let line_rects = engine.find_text_lines(&input, &word_rects);
    let line_texts = engine
        .recognize_text(&input, &line_rects)
        .map_err(|e| Error::Pdf(format!("OCR recognize: {e}")))?;

    let mut out = String::new();
    for line in line_texts.iter().flatten() {
        let s = line.to_string();
        if s.len() <= 1 {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&s);
    }
    Ok(out)
}

fn shared_inner() -> std::result::Result<&'static OcrsEngine, String> {
    SHARED
        .get_or_init(|| {
            let detection = Model::load_file(detection_model_path())
                .map_err(|e| format!("load detection model: {e}"))?;
            let recognition = Model::load_file(recognition_model_path())
                .map_err(|e| format!("load recognition model: {e}"))?;
            OcrsEngine::new(OcrEngineParams {
                detection_model: Some(detection),
                recognition_model: Some(recognition),
                ..Default::default()
            })
            .map_err(|e| format!("init OCR engine: {e}"))
        })
        .as_ref()
        .map_err(|e: &String| e.clone())
}

fn rgba_to_rgb8(width: u32, height: u32, rgba: &[u8]) -> Result<image::RgbImage> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| Error::Pdf("invalid image dimensions".into()))?;
    if rgba.len() < expected {
        return Err(Error::Pdf(format!(
            "RGBA buffer too short: got {}, need {expected}",
            rgba.len()
        )));
    }
    let img = image::ImageBuffer::from_raw(width, height, rgba[..expected].to_vec())
        .ok_or_else(|| Error::Pdf("failed to build RGBA image buffer".into()))?;
    Ok(image::DynamicImage::ImageRgba8(img).into_rgb8())
}
