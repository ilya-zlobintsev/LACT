export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /
PREFIX ?= /usr/local

build-release:
	cargo build --release

install:
	install -Dm755 target/release/lact $(DESTDIR)$(PREFIX)/bin/lact
	install -Dm644 res/lactd.service $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	install -Dm755 res/io.github.lact-linux.desktop $(DESTDIR)$(PREFIX)/share/applications/io.github.lact-linux.desktop
	install -Dm755 res/io.github.lact-linux.png $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.lact-linux.png

uninstall:
	rm $(DESTDIR)$(PREFIX)/bin/lact
	rm $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	rm $(DESTDIR)$(PREFIX)/share/applications/io.github.lact-linux.desktop
	rm $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.lact-linux.png
