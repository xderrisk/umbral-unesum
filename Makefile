FQBN = --fqbn esp32:esp32:esp32cam
BUILD = --build-property "build.extra_flags=-DBOARD_HAS_PSRAM"
.PHONY: desktop package compile upload monitor flash android build

desktop:
	cd desktop && cargo run
package:
	cd desktop && cross build --release --target aarch64-unknown-linux-gnu
	mkdir -p dist
	tar -czf dist/umbral-arm64.tar.gz \
		-C $$(pwd)/desktop/target/aarch64-unknown-linux-gnu/release umbral \
		-C $$(pwd)/desktop locale \
		-C $$(pwd)/desktop/data umbral.desktop umbral.png

compile:
	cd espcam && arduino-cli compile $(FQBN) $(BUILD) .
upload: compile
	cd espcam && arduino-cli upload -p /dev/ttyUSB0 $(FQBN) .
monitor:
	arduino-cli monitor -p /dev/ttyUSB0 -c baudrate=115200
flash: upload monitor

android:
	cd android && flutter run
build:
	mkdir -p dist
	cd android && flutter build apk --release --target-platform android-arm64
	cp android/build/app/outputs/flutter-apk/app-release.apk dist/umbral-arm64.apk
