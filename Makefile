export CARGO_TARGET_DIR ?= ./target
export CARGO_NET_GIT_FETCH_WITH_CLI ?= true
DESTDIR ?= /
PREFIX ?= /usr/local

.PHONY: build-release
build-release:
	# Build PGO-instrumented binary
	cargo pgo build -p lact --release
	# Run benchmarks to gather PGO profiles
	cargo pgo bench -p lact --release
	# Run benchmarks to show results
	cargo pgo optimize bench -p lact --release
	# Optimize binary with PGO
	cargo pgo optimize -p lact --release

.PHONY: build-debug
build-debug:
	cargo build -p lact

.PHONY: build-release-libadwaita
build-release-libadwaita:
	# Build PGO-instrumented binary with libadwaita features
	cargo pgo build -p lact --release --features=adw
	# Run benchmarks to gather PGO profiles
	cargo pgo bench -p lact --release --features=adw
	# Run benchmarks to show results
	cargo pgo optimize bench -p lact --release --features=adw
	# Optimize binary with PGO
	cargo pgo optimize -p lact --release --features=adw

.PHONY: build-release-headless
build-release-headless:
	# Build PGO-instrumented binary with headless (nvidia) features
	cargo pgo build -p lact --release --no-default-features --features=nvidia
	# Run benchmarks to gather PGO profiles
	cargo pgo bench -p lact --release --no-default-features --features=nvidia
	# Run benchmarks to show results
	cargo pgo optimize bench -p lact --release --no-default-features --features=nvidia
	# Optimize binary with PGO
	cargo pgo optimize -p lact --release --no-default-features --features=nvidia

.PHONY: install-resources
install-resources:
	install -Dm644 res/lactd.service $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.desktop $(DESTDIR)$(PREFIX)/share/applications/io.github.ilya_zlobintsev.LACT.desktop
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.png $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.ilya_zlobintsev.LACT.png
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.svg $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.svg
	install -Dm644 res/io.github.ilya_zlobintsev.LACT.metainfo.xml $(DESTDIR)$(PREFIX)/share/metainfo/io.github.ilya_zlobintsev.LACT.metainfo.xml

.PHONY: install
install: install-resources
	install -Dm755 target/release/lact $(DESTDIR)$(PREFIX)/bin/lact

.PHONY: install-debug
install-debug: install-resources
	install -Dm755 target/debug/lact $(DESTDIR)$(PREFIX)/bin/lact

.PHONY: uninstall
uninstall:
	rm $(DESTDIR)$(PREFIX)/bin/lact
	rm $(DESTDIR)$(PREFIX)/lib/systemd/system/lactd.service
	rm $(DESTDIR)$(PREFIX)/share/applications/io.github.ilya_zlobintsev.LACT.desktop
	rm $(DESTDIR)$(PREFIX)/share/pixmaps/io.github.ilya_zlobintsev.LACT.png
	rm $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.svg
	rm $(DESTDIR)$(PREFIX)/share/metainfo/io.github.ilya_zlobintsev.LACT.metainfo.xml