# docrafter

<p align="center">
  <img src=".github/banner.jpg" alt="docrafter" width="720" />
</p>

[![CI](https://github.com/pathofharmony/dockrafter/actions/workflows/ci.yml/badge.svg)](https://github.com/pathofharmony/dockrafter/actions/workflows/ci.yml)
[![docs.rs](https://docs.rs/docrafter/badge.svg)](https://docs.rs/docrafter)
[![crates.io](https://img.shields.io/crates/v/docrafter.svg)](https://crates.io/crates/docrafter)

Unified Rust library for **PDF** and **DOCX** documents — ergonomic API, strong typing, snapshot-tested output.

## Status

| Phase | Scope | Status |
|-------|--------|--------|
| **0** | Workspace, core types, Hello PDF, snapshots | ✅ Done |
| **0.1** | Table, Spacer, PageBreak, Image, word wrap, TTF/Cyrillic | ✅ Done |
| **0.2** | PDF read / merge (`PdfReader`, `merge`) | ✅ Done |
| **0.3** | DOCX read/write (runs, tables, images, lists, styles) | ✅ Done |
| **0.4** | ODT / LibreOffice (`OdtDocument`, shared `office` model) | ✅ Done |
| **1.0** | Multi-format export (`OfficeDocument` → PDF / DOCX / ODT) | ✅ Done |
| **1.1** | PDF multi-run (reportlab), `PdfReader::extract_text` (pypdf) | ✅ Done |
| **1.2** | PDF read: lopdf-compatible ToUnicode, OCR (Tesseract CLI) | ✅ Done |
| **1.3** | In-repo OCR (`docrafter-ocr`) + PDF render (`docrafter-pdf-render`), `from_bytes` | ✅ Done |
| **1.4** | Templates (`{{vars}}`), `ReportBuilder`, `export_bundle` | ✅ Done |
| **1.5** | PDF manipulate: extract/split pages, rotate, metadata (pypdf) | ✅ Done |
| **1.6** | Bookmarks, watermark, decrypt, Office/DOCX/ODT `append` | ✅ Done |
| **1.7** | PDF links, replace text, compress, `PdfExportOptions`, `office::open` | ✅ Done |
| **1.8** | HTML → office/PDF, PDF header/footer (`{page}` / `{pages}`) | ✅ Done |
| **2.0** | CLI (`docrafter`), `OpenDocument::open` | ✅ Done |
| **3.0** | CLI metadata/bookmark/extract, HTML `style=`, OCR CI, publish docs | ✅ Done |
| **4.0** | `cargo deny`, CHANGELOG, CLI info/replace/compress, license notes | ✅ Done |
| **5.0** | Publish prep, `ocr` feature flag (AGPL isolation), CLI template/links/batch, release workflow | ✅ Done |
| **6.0** | GitHub Release CLI binaries (Linux/macOS/Windows), HTML `text-align`/`div`, [docs/RELEASE.md](docs/RELEASE.md) | ✅ Done |
| **7.0** | HTML `a`/`hr`, CLI `pdf add-link`, [examples/sample.html](examples/sample.html) | ✅ Done |
| **8.0** | `pdf render`→PNG, HTML `blockquote`/`line-height`, layout line-height, Linux arm64 release, [docs/ZENPDF.md](docs/ZENPDF.md) | ✅ Done |
| **9.0** | Underline PDF/DOCX/ODT/HTML, CLI `batch --to pdf` | ✅ Done |
| **10.0** | Strikethrough all formats, HTML `colspan`, [docs/FINAL_CHECKLIST.md](docs/FINAL_CHECKLIST.md) | ✅ Done |
| **11.0** | Sup/sub, HTML `rowspan`, DOCX comments, PDF text fields, [docs/API.md](docs/API.md) | ✅ Done |
| **12.0** | crates.io + GitHub release, [docs/RELEASE_GUIDE_RU.md](docs/RELEASE_GUIDE_RU.md) | ✅ Done |

## Quick start

```rust
use docrafter::prelude::*;

let mut doc = PdfDocument::new();
doc.push(Paragraph::new("Hello, docrafter!"));
doc.save("hello.pdf")?;
```

Merge and read PDFs (pypdf-style):

```rust
use docrafter::prelude::*;

let mut out = PdfReader::open("cover.pdf")?;
assert!(out.page_count() >= 1);
out.merge(&PdfReader::open("appendix.pdf")?)?;
out.save("merged.pdf")?;

// Text extraction (works on docrafter-generated PDFs too)
let text = PdfReader::open("merged.pdf")?.extract_text()?;

// Page tools (pypdf-style)
use docrafter::pdf::{PdfMetadata, Rotate, WatermarkOptions};
let mut pdf = PdfReader::open("merged.pdf")?;
pdf.set_metadata(&PdfMetadata { title: Some("Merged".into()), ..Default::default() })?;
pdf.rotate(Some(&[1]), Rotate::Clockwise90)?;
let page2 = pdf.with_pages(&[2])?;
let parts = pdf.split()?;
pdf.add_bookmark("Chapter 1", 1, None)?;
pdf.add_watermark(None, &WatermarkOptions { text: "DRAFT".into(), ..Default::default() })?;
let unlocked = PdfReader::open_with_password("secret.pdf", "pass")?;

// Scanned PDFs: pure-Rust OCR (no tesseract/pdftoppm on PATH)
```

### OCR (models + release builds)

OCR model weights are **not** in git. Download once:

```bash
./scripts/fetch-ocr-models.sh   # → crates/docrafter-ocr/models/*.rten
```

Detection and recognition run in-repo (`docrafter-ocr`, `docrafter-pdf-render`). For production OCR use **release** (debug builds of ocrs/rten are very slow):

```bash
cargo build --release -p docrafter-cli
./target/release/docrafter pdf text scan.pdf --ocr -o text.txt
```

```rust
use docrafter::pdf::{OcrOptions, TextExtractMode};
let bytes = std::fs::read("scan.pdf")?;
let ocr = PdfReader::from_bytes(&bytes)?
    .extract_text_mode(TextExtractMode::Ocr(OcrOptions::default()))?;
```

The default RTen models are Latin-focused (HierText). Cyrillic in **generated** PDFs uses embedded DejaVu fonts (`extract_text()`). For scanned Cyrillic, OCR quality depends on the model; train or swap recognition weights via `docrafter-ocr` model paths.

**PDF edit on docrafter PDFs:** `replace_text` / `replace_text_all` (DejaVu CID). **Encrypt on save:** `reader.encrypt(&EncryptOptions::user("pass"))?` then `save` (revision 2, compatible with `open_with_password`).

```rust
use docrafter::pdf::{EncryptOptions, PdfReader};
let mut pdf = PdfReader::open("report.pdf")?;
pdf.encrypt(&EncryptOptions::user("secret"))?;
pdf.save("locked.pdf")?;
```

PDF paragraphs support **multiple runs** (like python-docx / reportlab inline styles):

```rust
use docrafter::prelude::*;

let mut doc = PdfDocument::new();
doc.push(
    Paragraph::new("Hello ")
        .run("world", Style::new().bold().color_value(Color::rgb(200, 0, 0)?)),
);
doc.save("styled.pdf")?;
```

DOCX:

```rust
use docrafter::docx::{DocxDocument, Image, List, Paragraph, Table};
use docrafter::prelude::Style;

let mut doc = DocxDocument::new();
doc.push(
    Paragraph::new("Отчёт ")
        .run("2026", Style::new().bold()),
);
doc.push_table(
    Table::professional()
        .columns(["Name", "Hours"])
        .row(["Anna", "40"]),
);
doc.push_list(List::new().item("First").item("Second"));
doc.push_image(Image::from_path("logo.png")?.size(48.0, 48.0));
doc.save("report.docx")?;

// Roundtrip
let loaded = DocxDocument::open("report.docx")?;
```

ODT (LibreOffice Writer):

```rust
use docrafter::odt::{OdtDocument, Paragraph};

let mut doc = OdtDocument::new();
doc.push(Paragraph::new("Привет, LibreOffice!"));
doc.save("hello.odt")?;  // OpenDocument 1.2 package
```

One model → PDF + Word + LibreOffice:

```rust
use docrafter::office::{OfficeDocument, Paragraph, Table};
use docrafter::prelude::*;

let mut doc = OfficeDocument::new();
doc.push(Paragraph::new("Отчёт").style(Style::heading1()));
doc.push_table(
    Table::professional()
        .columns(["Name", "Hours"])
        .row(["Anna", "40"]),
);

export_save_auto(&doc, "report.pdf")?;
export_save_auto(&doc, "report.docx")?;
export_save_auto(&doc, "report.odt")?;
// or all three at once:
export_bundle(&doc, "report")?;
```

Templates and report builder (Phase 1.4):

```rust
use docrafter::prelude::*;

let doc = ReportBuilder::new()
    .title("Report for {{month}}")
    .table_professional(["Name", "Hours"], &[vec!["Anna".into(), "40".into()]])
    .build(&Context::new().with("month", "May"))?;

export_bundle(&doc, "report")?;
```

Combine sections (Phase 1.6):

```rust
use docrafter::office::{OfficeDocument, Paragraph};

let mut report = OfficeDocument::new();
report.append(&intro);
report.append(&body);
export_bundle(&report, "full_report")?;
```

PDF export layout and links (Phase 1.7):

```rust
use docrafter::export::{PdfExportOptions, export_bytes_with_pdf_options, OutputFormat};
use docrafter::office::open;
use docrafter_core::PageSize;

let pdf = export_bytes_with_pdf_options(&doc, OutputFormat::Pdf, &PdfExportOptions {
    page_size: PageSize::letter(),
    ..Default::default()
})?;

let doc = open("report.docx")?; // or .odt
```

HTML → PDF / DOCX / ODT (Phase 1.8):

```rust
use docrafter::prelude::*;

let doc = html_to_office(
    "<h1>Report</h1><p>Hello <b>world</b></p><table><tr><th>A</th></tr><tr><td>1</td></tr></table>",
)?;
export_save_auto(&doc, "from_html.pdf")?;

let mut pdf = PdfDocument::new().with_header_footer(
    PageHeaderFooter::new().header("DRAFT").page_numbers(),
);
pdf.push(Paragraph::new("Content"));
pdf.save("with_footer.pdf")?;
```

CLI (Phase 2):

```bash
# Install binary into ~/.cargo/bin
./scripts/install-cli.sh
# or: cargo install --path crates/docrafter-cli --locked

docrafter pdf merge out.pdf a.pdf b.pdf
docrafter pdf text scan.pdf --ocr -o text.txt
docrafter pdf rotate in.pdf -o rot.pdf --angle 90
docrafter pdf split in.pdf -o ./pages/
docrafter pdf watermark in.pdf -o draft.pdf --text DRAFT
docrafter pdf extract in.pdf -o part.pdf --pages 1,3
docrafter pdf metadata in.pdf -o out.pdf --title "Report"
docrafter pdf bookmark in.pdf -o out.pdf --title "Start" --page 1
docrafter pdf encrypt in.pdf -o locked.pdf --password secret
docrafter pdf info report.pdf
docrafter pdf replace in.pdf -o out.pdf --from world --to docrafter
docrafter pdf compress big.pdf -o small.pdf
docrafter export report.docx -o report --bundle   # → report.pdf, .docx, .odt
docrafter convert report.docx -o report.pdf
docrafter html page.html -o page.pdf
docrafter template render examples/template-spec.json -o report --bundle
docrafter pdf links scan.pdf
docrafter pdf add-link doc.pdf -o linked.pdf --page 1 --rect 72,72,300,100 --uri https://example.com
docrafter pdf text-batch a.pdf b.pdf -o texts/ --ocr
docrafter html examples/sample.html -o sample.pdf
docrafter pdf render scan.pdf -o page1.png --page 1 --dpi 150
docrafter pdf render scan.pdf -o pages/ --all
docrafter batch report.html invoice.docx -o out/ --to pdf
docrafter pdf add-field form.pdf -o filled.pdf --page 1 --name email --rect 72,72,280,90 --value user@example.com
```

Rust API summary: [docs/API.md](docs/API.md).

**AGPL-free builds** (no OCR / zenpdf): depend on `docrafter` with `default-features = false`; enable only what you need. OCR needs feature `ocr` (default on the CLI crate).

From git (when published on crates.io: `cargo install docrafter-cli`):

```bash
cargo install --git https://github.com/pathofharmony/dockrafter --locked docrafter-cli
```

**Публикация и безопасность:** [docs/RELEASE_GUIDE_RU.md](docs/RELEASE_GUIDE_RU.md) (лицензии, секреты, crates.io, GitHub). Кратко: [docs/PUBLISH.md](docs/PUBLISH.md), [docs/FINAL_CHECKLIST.md](docs/FINAL_CHECKLIST.md).

**Releases:** tag `v0.1.0` → CI builds CLI archives ([docs/RELEASE.md](docs/RELEASE.md)).

## License

MIT OR Apache-2.0. PDF rendering for OCR uses **zenpdf** (AGPL-3.0) — see [LICENSE-NOTES.md](LICENSE-NOTES.md).

Quality gate: `./scripts/check.sh` (fmt, clippy, test, doc, `cargo deny` when installed).

Unified open API:

```rust
use docrafter::OpenDocument;

let doc = OpenDocument::open("report.docx")?;
match doc {
    OpenDocument::Office(o) => { /* export, edit blocks */ }
    OpenDocument::Pdf(p) => println!("pages: {}", p.page_count()),
}
```

```bash
cargo run --example hello_docx
cargo run --example hello_odt
cargo run --example hello_pdf
cargo run --example report_multi
cargo run --example report_template
./scripts/check.sh          # fmt + clippy + test + doc (как в CI)
./scripts/update-snapshots.sh  # обновить PDF fingerprints после смены вёрстки
```

## Workspace crates

- `docrafter` — public facade, `prelude`, `OpenDocument`
- `docrafter-cli` — `docrafter` binary (`cargo install --path crates/docrafter-cli`)
- `docrafter-core` — `Error`, `Style`, `Color`, `Length`, `PageSize`
- `docrafter-pdf-write` — PDF generation
- `docrafter-pdf-read` — PDF open, page count, merge
- `docrafter-office` — shared `Paragraph`, `Table`, `Image`, `List` (DOCX + ODT)
- `docrafter-docx` — DOCX OOXML read/write (Word + LibreOffice via filter)
- `docrafter-odt` — ODT OpenDocument read/write (native LibreOffice)
- `docrafter-layout` — flow layout engine
- `docrafter-font` — embedded TrueType (DejaVu Sans)
- `docrafter-template` — `{{variable}}` substitution and `ReportBuilder`
- `docrafter-html` — minimal HTML → `OfficeDocument`
- `docrafter-pdf-render` — pure-Rust PDF page rasterization
- `docrafter-ocr` — pure-Rust OCR engine
- `docrafter-testing` — snapshot and structural assertions

## License

MIT OR Apache-2.0
