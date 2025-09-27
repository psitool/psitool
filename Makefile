all: lint psitool
	cargo install --path . --force

psitool:
	cargo build --release

fmt:
	cargo fmt

lint: fmt
	cargo clippy -- -D warnings

clean:
	cargo clean

test:
	cargo test --verbose

ctags:
	ctags -R . ~/.cargo/registry/src
