# Releases

## What runs on a version tag

Pushing a tag `v*` (for example `v0.1.0`) triggers [`.github/workflows/release.yml`](../.github/workflows/release.yml):

| Job | Always | Needs |
|-----|--------|--------|
| `check` | `./scripts/check.sh` | — |
| `binaries` | CLI archives for Linux, macOS (arm64 + x64), Windows | `check` |
| `github-release` | Attaches archives to a GitHub Release | `binaries` |
| `publish-crates` | `cargo publish` in dependency order | `check` + secret |

`publish-crates` runs only when the repository has **`CARGO_REGISTRY_TOKEN`** in GitHub Actions secrets (crates.io API token). Without it, binaries and the GitHub Release are still created.

## Cut a release (maintainer)

1. Bump `version` in the workspace root [`Cargo.toml`](../Cargo.toml) (and sync crate versions via `workspace.package`).
2. Update [`CHANGELOG.md`](../CHANGELOG.md).
3. Commit, tag, push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

4. After CI finishes, download `docrafter-*-*.tar.gz` / `.zip` from the GitHub Release page, or install from source / crates.io.

## Local install (no GitHub Release)

```bash
./scripts/install-cli.sh
# OCR: ./scripts/fetch-ocr-models.sh && cargo build --release -p docrafter-cli
```

## crates.io only (no tag)

```bash
cargo login   # once, on your machine
./scripts/publish-crates.sh
```

See [PUBLISH.md](PUBLISH.md) for crate order and first-publish notes.
