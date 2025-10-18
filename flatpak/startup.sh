#!/bin/env bash
set -e 

DAEMON_SOCKET=/run/lactd.sock
UNIT_PATH=/etc/systemd/system/lactd.service

if [ ! -e "${DAEMON_SOCKET}" ]; then
    echo "${DAEMON_SOCKET} does not exist, showing setup prompt"
    
    APP_COMMIT=$(grep app-commit= /.flatpak-info | sed "s/app-commit=//")
    APP_PATH=$(grep app-path= /.flatpak-info | sed "s/app-path=//" | sed "s/${APP_COMMIT}/active/")
    DAEMON_SH_PATH="${APP_PATH}/bin/daemon.sh"
    
    set +e

    YAD_OUTPUT=$(yad --title="LACT Flatpak Setup" \
    --text="The LACT system service could not be found.
In order to edit GPU settings, LACT requires a service to be running as root.
This setup will install <u><tt>lactd.service</tt></u> outside of the Flatpak sandbox.

It is possible to skip this step if you wish to use LACT only for information and monitoring.

Do you wish to install the service?" \
    --form \
    --field "Autostart service at boot":CHK "TRUE")
    EXIT_CODE=$?
    
    set -e
    
    if [ ${EXIT_CODE} -eq 0 ]; then
        IFS="|" read -ra OPTIONS <<< "$YAD_OUTPUT"
        AUTOSTART="${OPTIONS[0]}"
        echo "Setting up the service with autostart ${AUTOSTART}"

        UNIT="\
[Unit]
Description=GPU Control Daemon (via Flatpak)
After=multi-user.target

[Service]
Environment=FLATPAK_INSTALL_USER=$USER
ExecStart=$DAEMON_SH_PATH
Nice=-10
Restart=on-failure

[Install]
WantedBy=multi-user.target\
        "

        if flatpak-spawn --host sh -c "command -v pkexec" > /dev/null 2>&1; then
            ROOT_WRAPPER="pkexec"
        else
            ROOT_WRAPPER="run0 --pipe"
        fi

        echo "Using root wrapper ${ROOT_WRAPPER}"

        echo "${UNIT}" | flatpak-spawn --host tee /tmp/lact-unit-setup

        flatpak-spawn --host $ROOT_WRAPPER sh -c "cp /tmp/lact-unit-setup ${UNIT_PATH} && (chcon -R -t bin_t $DAEMON_SH_PATH || true)"
        echo "Unit file created at ${UNIT_PATH}"
        
        flatpak-spawn --host rm /tmp/lact-unit-setup

        if [ "${AUTOSTART}" == "TRUE" ]; then
            echo "Enabling the service with autostart"
            flatpak-spawn --host $ROOT_WRAPPER sh -c "systemctl daemon-reload && systemctl enable --now lactd.service"
        else
            echo "Starting the service without autostart"
            flatpak-spawn --host $ROOT_WRAPPER sh -c "systemctl daemon-reload && systemctl start lactd.service"
        fi

        yad --text 'The service has been started. Please run LACT again.'  --button=OK:0
        exit 0
    else
        echo "Service setup rejected, skipping"
    fi
fi

if [ -z "$GTK_THEME" ]; then
    set +e
    if [ "$(gsettings get org.gnome.desktop.interface gtk-theme)" = "'Adwaita'" ] && [ "$(gsettings get org.gnome.desktop.interface color-scheme)" = "'default'" ]; then
        COLOR_SCHEME=$(dbus-send --session --print-reply \
          --dest=org.freedesktop.portal.Desktop \
          /org/freedesktop/portal/desktop \
          org.freedesktop.portal.Settings.Read \
          string:"org.freedesktop.appearance" \
          string:"color-scheme" \
	  | grep -oP '(?<=uint32 )\d+')

	if [ "$COLOR_SCHEME" = "1" ]; then
	    export GTK_THEME="Adwaita:dark"
	    echo "Detected dark theme system setting, GTK theme overriden to $GTK_THEME"
	fi
    fi
    set -e
fi

lact $@
