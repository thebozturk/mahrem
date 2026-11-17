.PHONY: check fmt lint test py-test deny audit

check: fmt lint test

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --exclude mahrem-py -- -D warnings

test:
	cargo test --workspace --exclude mahrem-py

py-test:
	uv sync
	uv run pytest
	uv run ruff check .
	uv run ruff format --check .

deny:
	cargo deny check

audit:
	cargo audit
