# PDF rasterization (zenpdf / AGPL)

Scanned-PDF **OCR** and **`pdf render`** rasterize pages through `docrafter-pdf-render`, which depends on [zenpdf](https://crates.io/crates/zenpdf) (**AGPL-3.0**). See [LICENSE-NOTES.md](../LICENSE-NOTES.md).

## Avoiding AGPL in your app

Depend on `docrafter` without default features and omit `ocr`:

```toml
docrafter = { version = "0.1", default-features = false }
```

You keep PDF generation, DOCX/ODT, merge, text extract (embedded fonts), and HTML export. You lose in-process OCR and CLI `pdf text --ocr` / `pdf render`.

## Future options (not implemented)

| Approach | Notes |
|----------|--------|
| Commercial zenpdf license | From Imazen; keeps current code path |
| Alternate rasterizer | e.g. PDFium (license review), pure-Rust subset |
| External render service | Out of scope for the library |

The workspace keeps zenpdf behind the **`ocr`** feature so MIT/Apache-only consumers can opt out explicitly.
