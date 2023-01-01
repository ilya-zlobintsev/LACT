export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /usr/local

build-release:
	cargo build --release

install:
	install -Dm755 target/release/lact-daemon ${DESTDIR}/bin/lact-daemon
	install -Dm755 target/release/lact-gui ${DESTDIR}/bin/lact-gui
	install -Dm755 target/release/lact-cli ${DESTDIR}/bin/lact-cli
	install -Dm755 res/lactd.service ${DESTDIR}/lib/systemd/system/lactd.service
	install -Dm755 res/lact.desktop ${DESTDIR}/share/applications/lact.desktop
	install -Dm755 res/lact.png ${DESTDIR}/share/pixmaps/lact.png

uninstall:
	rm ${DESTDIR}/bin/lact-daemon
	rm ${DESTDIR}/bin/lact-gui
	rm ${DESTDIR}/bin/lact-cli
	rm ${DESTDIR}/lib/systemd/system/lactd.service
	rm ${DESTDIR}/share/applications/lact.desktop
	rm ${DESTDIR}/share/pixmaps/lact.png
