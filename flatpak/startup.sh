#!/bin/env bash
set -e 

DAEMON_SOCKET=/run/lactd.sock
UNIT_PATH=/etc/systemd/system/lactd.service

if [ ! -e "${DAEMON_SOCKET}" ]; then
    echo "${DAEMON_SOCKET} does not exist, showing setup prompt"
    
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
Description=AMDGPU Control Daemon (via Flatpak)
After=multi-user.target

[Service]
Environment=FLATPAK_INSTALL_USER=$USER
Environment=VULKANINFO_COMMAND=\"sudo -u $USER -s flatpak run --filesystem=/tmp --command=vulkaninfo io.github.lact-linux\"
ExecStart=sh -c \"eval \$(sudo -u \$FLATPAK_INSTALL_USER -s flatpak run --command=lact io.github.lact-linux flatpak generate-daemon-cmd)\"
Nice=-10
Restart=on-failure

[Install]
WantedBy=multi-user.target\
        "

        echo "${UNIT}" | flatpak-spawn --host pkexec tee "${UNIT_PATH}"
        echo "Unit file created at ${UNIT_PATH}"
        
        if [ "${AUTOSTART}" == "TRUE" ]; then
            echo "Enabling the service with autostart"
            flatpak-spawn --host pkexec sh -c "systemctl daemon-reload && systemctl enable --now lactd.service"
        else
            echo "Starting the service without autostart"
            flatpak-spawn --host pkexec sh -c "systemctl daemon-reload && systemctl start lactd.service"
        fi

        yad --text 'The service has been started. Please run LACT again.'  --button=OK:0
        exit 0
    else
        echo "Service setup rejected, skipping"
    fi
else
    echo "${DAEMON_SOCKET} exists, skipping setup prompt"
fi

lact $@
