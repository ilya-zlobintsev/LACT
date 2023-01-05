export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /usr/local

build-release:
	cargo build --release

install:
	install -Dm755 target/release/lact ${DESTDIR}/bin/lact
	install -Dm755 res/lactd.service ${DESTDIR}/lib/systemd/system/lactd.service
	install -Dm755 res/lact.desktop ${DESTDIR}/share/applications/lact.desktop
	install -Dm755 res/lact.png ${DESTDIR}/share/pixmaps/lact.png

uninstall:
	rm ${DESTDIR}/bin/lact
	rm ${DESTDIR}/lib/systemd/system/lactd.service
	rm ${DESTDIR}/share/applications/lact.desktop
	rm ${DESTDIR}/share/pixmaps/lact.png
