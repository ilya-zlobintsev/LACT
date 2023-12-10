export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /
PREFIX ?= /usr/local

build-release:
	cargo build -p lact --release
	
build-release-libadwaita:
	cargo build -p lact --release --features=adw
	
build-release-headless:
	cargo build -p lact --release --no-default-features --features=drm

install:
	install -Dm755 target/release/lact $(DESTDIR)$(PREFIX)/bin/lact
	install -Dm644 res/lactd.service $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	install -Dm644 res/io.github.lact-linux.desktop $(DESTDIR)$(PREFIX)/share/applications/io.github.lact-linux.desktop
	install -Dm644 res/io.github.lact-linux.png $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.lact-linux.png
	install -Dm644 res/io.github.lact-linux.svg $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.lact-linux.svg
	install -Dm644 res/io.github.lact-linux.thermometer-symbolic.svg $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/status/io.github.lact-linux.thermometer-symbolic.svg

uninstall:
	rm $(DESTDIR)$(PREFIX)/bin/lact
	rm $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	rm $(DESTDIR)$(PREFIX)/share/applications/io.github.lact-linux.desktop
	rm $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.lact-linux.png
	rm $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.lact-linux.svg
