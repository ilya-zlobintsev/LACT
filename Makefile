export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /usr/local

build-release:
	cargo build --release

install:
	install -Dm755 target/release/lact ${DESTDIR}/bin/lact
	install -Dm644 res/lactd.service ${DESTDIR}/lib/systemd/system/lactd.service
	install -Dm755 res/io.github.lact-linux.desktop ${DESTDIR}/share/applications/io.github.lact-linux.desktop
	install -Dm755 res/io.github.lact-linux.png ${DESTDIR}/share/pixmaps/io.github.lact-linux.png

uninstall:
	rm ${DESTDIR}/bin/lact
	rm ${DESTDIR}/lib/systemd/system/lactd.service
	rm ${DESTDIR}/share/applications/io.github.lact-linux.desktop
	rm ${DESTDIR}/share/pixmaps/io.github.lact-linux.png
