rmall: src/
	cargo build --release

install:
	cp ./target/release/rmall ~/.local/bin/rmall

run:
	./target/release/rmall