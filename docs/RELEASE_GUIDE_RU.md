# Руководство по релизу docrafter

Пошаговая инструкция: права, проверки, публикация на crates.io и GitHub.

## 1. Права и лицензии

| Что | Статус |
|-----|--------|
| Код workspace | **MIT OR Apache-2.0** (см. заголовки crate) |
| Зависимости | `cargo deny` в CI — только разрешённые лицензии в [`deny.toml`](../deny.toml) |
| OCR / рендер PDF | **AGPL** через `zenpdf` — см. [LICENSE-NOTES.md](../LICENSE-NOTES.md), [ZENPDF.md](ZENPDF.md) |
| Модели OCR | **не в git**; скачиваются [`scripts/fetch-ocr-models.sh`](../scripts/fetch-ocr-models.sh) (веса ocrs, отдельная лицензия upstream) |

Перед публикацией:

```bash
./scripts/check.sh          # fmt, clippy, test, doc, deny
cargo deny check            # дублируется в check.sh
```

Не коммитить: `.env`, `*.key`, `target/`, `hello.pdf`, `crates/docrafter-ocr/models/*.rten`.

## 2. Секреты и утечки

Проверка перед `git push`:

```bash
git grep -iE 'token|api[_-]?key|password\s*=|BEGIN (RSA|OPENSSH)' -- ':!*.md' ':!target'
git status                  # нет .env, ключей, бинарников PDF
```

- Токен crates.io — только в `~/.cargo/credentials.toml` или `CARGO_REGISTRY_TOKEN` в GitHub Secrets, **не в репозитории**.
- В истории git не должно быть `cargo login` с токеном в терминале (при утечке — отозвать токен на crates.io).

## 3. Тесты

```bash
./scripts/check.sh

# OCR (опционально, нужны модели):
./scripts/fetch-ocr-models.sh
cargo test -p docrafter-pdf-read
cargo test -p docrafter --test pdf_extract
```

Обновление снапшотов (только при намеренном изменении вывода):

```bash
DOCRAFTER_UPDATE_SNAPSHOTS=1 cargo test -p docrafter
```

## 4. Версия и CHANGELOG

1. В корневом [`Cargo.toml`](../Cargo.toml): `version = "0.1.0"` (или следующий semver).
2. В [`CHANGELOG.md`](../CHANGELOG.md): перенести блок **Unreleased** в `## [0.1.0] - YYYY-MM-DD`.

## 5. crates.io

Порядок публикации (каждый crate зависит от предыдущих на registry):

```bash
cargo login                    # один раз, локально
./scripts/publish-crates.sh --dry-run
./scripts/publish-crates.sh    # ~14 crate по очереди
```

Список: `docrafter-core` → … → `docrafter` → `docrafter-cli`.  
`docrafter-testing` **не публикуется**.

После публикации:

```bash
cargo install docrafter-cli --locked
docrafter --help
```

## 6. GitHub

Репозиторий: **https://github.com/pathofharmony/dockrafter**

```bash
git init
git add -A
git commit -m "Initial release v0.1.0"
git branch -M main
git remote add origin https://github.com/pathofharmony/dockrafter.git
git push -u origin main
```

Превью для соцсетей / Open Graph: [`.github/social-preview.png`](../.github/social-preview.png) (из репозитория ipkt).

Релиз с бинарниками CLI:

```bash
git tag v0.1.0
git push origin v0.1.0
```

CI ([`release.yml`](../.github/workflows/release.yml)) соберёт архивы Linux/macOS/Windows.  
Опционально: секрет `CARGO_REGISTRY_TOKEN` для job `publish-crates`.

## 7. Авторы и контрибьюторы

- В `Cargo.toml`: `authors = ["pathofharmony"]` — без автоматических записей IDE/агентов.
- Не добавлять Cursor, Copilot и т.п. в `AUTHORS`, `CONTRIBUTORS` или co-authored-by в коммитах.

## 8. Установка из git (до crates.io)

```bash
cargo install --git https://github.com/pathofharmony/dockrafter --locked docrafter-cli
```

## 9. Чеклист одной строкой

- [ ] `./scripts/check.sh` зелёный  
- [ ] Нет секретов в git  
- [ ] CHANGELOG + версия  
- [ ] `./scripts/publish-crates.sh`  
- [ ] `git push` + тег `v0.1.0`  
- [ ] Smoke: `docrafter html examples/sample.html -o /tmp/out.pdf`

См. также: [FINAL_CHECKLIST.md](FINAL_CHECKLIST.md), [PUBLISH.md](PUBLISH.md), [RELEASE.md](RELEASE.md).
