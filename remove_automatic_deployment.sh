#!/usr/bin/env bash
# fail on error
set -e

# remove all registered services
echo "Stop services"
sudo systemctl stop {pairing_deployment,pairing_sync,pairing_dashboard,pairing_simulation}.service
echo "Remove services"
sudo systemctl disable {pairing_deployment,pairing_sync,pairing_dashboard,pairing_simulation}.service
sudo rm /etc/systemd/system/{pairing_deployment,pairing_sync,pairing_dashboard,pairing_simulation}.service
sudo systemctl daemon-reload
echo "Finished"
