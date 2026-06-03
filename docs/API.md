# API overview

## Prelude

```rust
use docrafter::prelude::*;
```

Exports the usual builders (`PdfDocument`, `OfficeDocument`, `DocxDocument`, `ReportBuilder`), `OpenDocument`, export helpers, and core types including `VerticalAlign`.

## Vertical alignment (superscript / subscript)

```rust
let p = Paragraph::new("E=mc")
    .run("2", Style::new().superscript());
```

Works in PDF, DOCX, ODT, and HTML (`<sup>`, `<sub>`, `vertical-align`).

## DOCX comments

```rust
let mut doc = DocxDocument::new();
doc.push(Paragraph::new("Contract text"));
doc.add_comment("Legal", "Verify clause 4");
doc.save("review.docx")?;
```

Comments attach to the first paragraph in the saved file (`word/comments.xml` + range markers). `DocxDocument::open` / `from_bytes` reloads comments into `comments()`.

## PDF forms (text fields)

```rust
let mut pdf = PdfReader::open("form.pdf")?;
pdf.add_text_field(1, [72.0, 72.0, 300.0, 96.0], "name_field", "Alice")?;
pdf.save("filled.pdf")?;
```

CLI:

```bash
docrafter pdf add-field form.pdf -o out.pdf --page 1 --name email --value user@example.com --rect 72,72,300,96
docrafter pdf fields form.pdf
```

## OCR / AGPL

Default builds include OCR and PDF render (zenpdf). Use `default-features = false` on `docrafter` and enable `ocr` only when needed. See [ZENPDF.md](ZENPDF.md).

## Publishing

Deferred until the roadmap is complete — [FINAL_CHECKLIST.md](FINAL_CHECKLIST.md).
