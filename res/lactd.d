[Unit]
Description=GPU Control Daemon
After=multi-user.target

[Service]
ExecStart=lact daemon
Nice=-10
Restart=on-failure

[Install]
WantedBy=multi-user.target
