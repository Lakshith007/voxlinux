[Unit]
Description=VoxLinux Self-Healing Daemon
After=network.target

[Service]
ExecStart=/usr/local/bin/voxlinuxd
Restart=always
User=root

[Install]
WantedBy=multi-user.target
