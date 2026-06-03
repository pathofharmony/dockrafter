# Publishing to crates.io

> **When:** do this at the **end** of the roadmap, when the API is stable. Until then, install from the repo (`./scripts/install-cli.sh`).

Publish **dependencies first**, then the facade and CLI.

## Prerequisites

- `cargo login` (crates.io token)
- OCR model files are **not** published (`crates/docrafter-ocr/models/` is gitignored)
- Run `./scripts/check.sh` before any publish

## Order

```bash
./scripts/publish-crates.sh --dry-run   # cargo package (works before deps are on crates.io)
./scripts/publish-crates.sh             # cargo publish in order
```

`--dry-run` runs `./scripts/check.sh`, packages `docrafter-core`, and prints the `cargo publish` commands. Full `cargo package` for downstream crates only works after their path dependencies are already on crates.io.

Crates in dependency order:

1. `docrafter-core`
2. `docrafter-font`
3. `docrafter-layout`
4. `docrafter-office`
5. `docrafter-pdf-write`
6. `docrafter-docx`
7. `docrafter-odt`
8. `docrafter-html`
9. `docrafter-template`
10. `docrafter-ocr`
11. `docrafter-pdf-render`
12. `docrafter-pdf-read`
13. `docrafter` (library)
14. `docrafter-cli` (binary `docrafter`)

`docrafter-testing` is internal — not published by default.

Tagged releases also ship prebuilt CLI archives; see [RELEASE.md](RELEASE.md).

## Install after publish

```bash
cargo install docrafter-cli --locked
# binary: docrafter
```

From this repo without publishing:

```bash
./scripts/install-cli.sh
# or: cargo install --path crates/docrafter-cli --locked --release
```
