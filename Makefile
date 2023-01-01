export CARGO_TARGET_DIR ?= ./target
DESTDIR ?= /usr/local

build-release:
	cargo build --release

install:
	install -Dm755 target/release/lact-daemon ${DESTDIR}/bin/lact-daemon
	install -Dm755 target/release/lact-gui ${DESTDIR}/bin/lact-gui
	install -Dm755 lactd.service ${DESTDIR}/lib/systemd/system/lactd.service

uninstall:
	rm ${DESTDIR}/bin/lact-daemon
	rm ${DESTDIR}/bin/lact-gui
	rm ${DESTDIR}/lib/systemd/system/lactd.service
