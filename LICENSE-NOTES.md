# License notes

## MIT OR Apache-2.0

All **docrafter** workspace crates are dual-licensed under MIT OR Apache-2.0 unless noted below.

## AGPL: `zenpdf` (PDF rendering)

`docrafter-pdf-render` depends on [zenpdf](https://crates.io/crates/zenpdf), which is licensed under **AGPL-3.0-only** (or a commercial Imazen license).

Implications:

- Using **OCR** or `render_page_rgba` pulls in this dependency chain.
- If you distribute a combined work that links zenpdf, AGPL obligations may apply to that distribution.
- PDF **generation** and **lopdf** read/edit paths do not require zenpdf.

For proprietary products, evaluate zenpdf’s commercial license or avoid the render/OCR code path.

### Cargo feature `ocr`

The facade crate `docrafter` and `docrafter-pdf-read` expose an optional feature **`ocr`** (enabled by default):

```toml
docrafter = { version = "0.1", default-features = false }
```

With `default-features = false`, PDF read/write, DOCX/ODT, and templates do not pull `zenpdf` or `docrafter-ocr`. Enable `features = ["ocr"]` when you need scanned-PDF text or page rasterization.

See also [docs/ZENPDF.md](docs/ZENPDF.md).
