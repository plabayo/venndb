set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D rustdoc::broken-intra-doc-links"
export RUST_LOG := "debug"

fmt:
	cargo fmt --all

sort:
	cargo sort --workspace --grouped

lint: fmt sort

check:
	cargo check --all --all-targets --all-features

clippy:
	cargo clippy --all --all-targets --all-features

clippy-fix:
	cargo clippy --fix

typos:
	typos -w

doc:
	cargo doc --all-features --no-deps

doc-open:
	cargo doc --all-features --no-deps --open

hack:
	cargo hack check --each-feature --no-dev-deps --workspace

test:
	cargo test --all-features --workspace

qa: lint check clippy doc hack test

watch-docs:
	cargo watch -x doc

watch-check:
	cargo watch -x check -x test

fuzz:
	cargo +nightly fuzz run fuzz_employee_db -- -max_len=131072

fuzz-30s:
	cargo +nightly fuzz run fuzz_employee_db -- -max_len=131072 -max_total_time=60

bench:
	cargo bench

detect-unused-deps:
	@cargo install cargo-machete
	cargo machete --skip-target-dir

update-deps:
    cargo upgrade
    cargo upgrades
    cargo update

publish:
    cargo publish -p venndb-macros
    cargo publish -p venndb
