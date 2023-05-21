.PHONY: lint
lint:
	@cargo clippy --tests --workspace -- -D warnings

.PHONY: test
test:
	@cargo test -- --nocapture $(name)

.PHONY: deny
deny:
	@cargo deny check
