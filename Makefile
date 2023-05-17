.PHONY: lint
lint:
	cargo clippy --tests --workspace -- -D warnings
