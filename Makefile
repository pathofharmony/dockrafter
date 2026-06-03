.PHONY: check test update-snapshots example

check:
	./scripts/check.sh

test:
	cargo test --workspace --all-targets

update-snapshots:
	./scripts/update-snapshots.sh

example:
	cargo run -p docrafter --example hello_pdf
