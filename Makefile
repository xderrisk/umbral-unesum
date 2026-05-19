dev:
	cd desktop && cargo run
rasp:
	cd desktop && cross build --release --target aarch64-unknown-linux-gnu
