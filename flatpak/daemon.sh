#!/bin/env bash
set -e

FLATPAK_BIN_DIR=$(dirname "$0")
FLATBOX_BIN="${FLATPAK_BIN_DIR}/flatbox"

FLATPAK_USER_DIR=/home/${FLATPAK_INSTALL_USER}/.local/share/flatpak

export LACT_DAEMON_SOCKET_PATH="/run/host/root/run/lactd.sock"
export LACT_DAEMON_CONFIG_DIR="/run/host/root/etc/lact"

exec dbus-launch $FLATBOX_BIN run --flatpak-install-path $FLATPAK_USER_DIR --app io.github.ilya_zlobintsev.LACT lact daemon
