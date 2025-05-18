export CARGO_TARGET_DIR ?= ./target
export CARGO_NET_GIT_FETCH_WITH_CLI ?= true
DESTDIR ?= /
PREFIX ?= /usr/local

build-release:
	cargo build -p lact --release

build-debug:
	cargo build -p lact
	
build-release-libadwaita:
	cargo build -p lact --release --features=adw
	
build-release-headless:
	cargo build -p lact --release --no-default-features
	
install-resources:
	install -Dm644 res/lactd.service $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.desktop $(DESTDIR)$(PREFIX)/share/applications/io.github.ilya_zlobintsev.LACT.desktop
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.png $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.ilya_zlobintsev.LACT.png
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.svg $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.svg
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.metainfo.xml $(DESTDIR)$(PREFIX)/share/metainfo/io.github.ilya_zlobintsev.LACT.metainfo.xml

install: install-resources
	install -Dm755 target/release/lact $(DESTDIR)$(PREFIX)/bin/lact
	
install-debug: install-resources
	install -Dm755 target/debug/lact $(DESTDIR)$(PREFIX)/bin/lact

uninstall:
	rm $(DESTDIR)$(PREFIX)/bin/lact
	rm $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	rm $(DESTDIR)$(PREFIX)/share/applications/io.github.ilya_zlobintsev.LACT.desktop
	rm $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.ilya_zlobintsev.LACT.png
	rm $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.svg
	rm $(DESTDIR)$(PREFIX)/share/metainfo/io.github.ilya_zlobintsev.LACT.metainfo.xml
