#!/usr/bin/env bash
# Download RTen OCR models into crates/docrafter-ocr/models/
set -euo pipefail
cd "$(dirname "$0")/.."

MODELS_DIR="crates/docrafter-ocr/models"
mkdir -p "$MODELS_DIR"

DETECTION_MODEL="https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten"
RECOGNITION_MODEL="https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten"

echo "==> text-detection.rten"
curl -fsSL "$DETECTION_MODEL" -o "$MODELS_DIR/text-detection.rten"

echo "==> text-recognition.rten"
curl -fsSL "$RECOGNITION_MODEL" -o "$MODELS_DIR/text-recognition.rten"

echo "OK: models in $MODELS_DIR"
echo "Note: default recognition model is Latin-focused; use --release for OCR (debug is very slow)."
