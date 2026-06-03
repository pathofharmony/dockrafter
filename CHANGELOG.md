# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## [0.1.0] - 2026-06-03

### Added

- Superscript/subscript (`VerticalAlign`, `<sup>`/`<sub>`, DOCX `w:vertAlign`, PDF baseline shift)
- HTML table `rowspan`; DOCX review comments (`DocxDocument::add_comment`, load on open)
- PDF AcroForm text fields (`add_text_field`, CLI `pdf add-field` / `pdf fields`)
- [docs/API.md](docs/API.md) — prelude and feature entry points
- Strikethrough / underline across PDF, DOCX, ODT, HTML
- HTML table `colspan`; CLI `batch --to pdf|docx|odt`
- CLI `pdf render` → PNG; Linux `aarch64` release binaries
- HTML: `blockquote`, `a`, `hr`, `div`, `text-align`, `line-height`
- GitHub Release CLI binaries; `ocr` feature flag (AGPL isolation)
- CLI: merge, text/OCR, rotate, split, watermark, extract, metadata, bookmark, encrypt, template, links
- `OpenDocument::open`, `replace_text`, `PdfReader::encrypt`
- Multi-format export (PDF / DOCX / ODT), templates, in-repo OCR (RTen)

### Documentation

- [docs/RELEASE_GUIDE_RU.md](docs/RELEASE_GUIDE_RU.md), [docs/PUBLISH.md](docs/PUBLISH.md), [docs/FINAL_CHECKLIST.md](docs/FINAL_CHECKLIST.md)
- [LICENSE-NOTES.md](LICENSE-NOTES.md), [docs/ZENPDF.md](docs/ZENPDF.md)

[0.1.0]: https://github.com/pathofharmony/dockrafter/releases/tag/v0.1.0
