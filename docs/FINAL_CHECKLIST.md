# Final checklist (end of roadmap)

Полная инструкция на русском: [RELEASE_GUIDE_RU.md](RELEASE_GUIDE_RU.md).

Use this when the API is stable and you are ready to ship.

## 1. Quality gate

```bash
./scripts/check.sh
```

Optional: fetch OCR models and run OCR tests locally:

```bash
./scripts/fetch-ocr-models.sh
cargo test -p docrafter-pdf-read
```

## 2. Version and changelog

- Bump `version` in the workspace root [`Cargo.toml`](../Cargo.toml).
- Update [`CHANGELOG.md`](../CHANGELOG.md) — move **Unreleased** entries under the new version.

## 3. Git tag and GitHub Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

CI ([`release.yml`](../.github/workflows/release.yml)) builds CLI archives for Linux (x64 + arm64), macOS, and Windows. Optional: set `CARGO_REGISTRY_TOKEN` for automatic `cargo publish`.

See [RELEASE.md](RELEASE.md).

## 4. crates.io (manual)

When you are ready (not required for local use):

```bash
cargo login   # if not already
./scripts/publish-crates.sh
```

See [PUBLISH.md](PUBLISH.md) — publish **in order**, first release only after each dependency exists on crates.io.

## 5. Install smoke test

```bash
cargo install --path crates/docrafter-cli --locked --release
docrafter --help
docrafter html examples/sample.html -o /tmp/sample.pdf
```

After crates.io publish:

```bash
cargo install docrafter-cli --locked
```
