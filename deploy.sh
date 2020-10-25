#!/bin/sh
cargo build --release
install -Dm755 target/release/daemon /usr/local/bin/lact-daemon
install -Dm755 target/release/cli /usr/local/bin/lact-cli
install -Dm755 target/release/gui /usr/local/bin/lact-gui
install -Dm644 lact.service /etc/systemd/system/lact.service
systemctl daemon-reload
systemctl restart lact