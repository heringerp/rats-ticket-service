#!/bin/bash

echo "Start Installation"
mkdir /opt/rats
mkdir /opt/rats/db
mkdir /opt/rats/uploads
cp -r ./templates /opt/rats
cp -r ./static /opt/rats
cp ./Rocket.toml /opt/rats
cp ./ticket_service /opt/rats
cp ./rats.service /usr/lib/systemd/system/
chmod +x /opt/rats/ticket_service
chown -R www-data /opt/rats
echo "Installation finished successfully"
