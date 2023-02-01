format:
	cargo fmt --all
	cd integration-test && taplo format
	cd pallets && taplo format

test:
	cargo nextest run --workspace

integration-test:
	cargo nextest run --workspace -p xbi-integration-tests