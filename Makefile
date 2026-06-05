FQBN = --fqbn esp32:esp32:esp32cam
BUILD = --build-property "build.extra_flags=-DBOARD_HAS_PSRAM"

dev:
	cd desktop && cargo run
rasp:
	cd desktop && cross build --release --target aarch64-unknown-linux-gnu
compile:
	cd espcam && arduino-cli compile $(FQBN) $(BUILD) .
upload: compile
	cd espcam && arduino-cli upload -p /dev/ttyUSB0 $(FQBN) .
monitor:
	arduino-cli monitor -p /dev/ttyUSB0 -c baudrate=115200
flash: upload monitor
package: rasp
	mkdir -p desktop/dist
	tar -czf desktop/dist/umbral-arm64.tar.gz \
		-C $$(pwd)/desktop/target/aarch64-unknown-linux-gnu/release umbral \
		-C $$(pwd)/desktop locale
