#!/usr/bin/env bash
# fail on error
set -e

echo "Getting local user"
# get path
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
REAL_USER="$(echo "$SCRIPT_DIR" | cut -d'/' -f3)"
USER_HOME_PATH="/home/$REAL_USER"

setup_restart_service() {
  SUBDIR="$1"
  SERVICE="$2"
  export SCRIPT_USER="$3"
  export DIR="$SCRIPT_DIR/$SUBDIR"
  export USER_HOME=$USER_HOME_PATH

  echo "Setting and restarting $SERVICE in dir $SUBDIR using user $REAL_USER"
  # copy sync service to systemd
  sudo -E envsubst < "$SCRIPT_DIR/$SUBDIR/$SERVICE.service.template" > "/etc/systemd/system/$SERVICE.service"
  sudo chmod 644 "/etc/systemd/system/$SERVICE.service"

  echo "Reloading and starting"
  # reload and restart service
  sudo systemctl daemon-reload
  sudo systemctl enable "$SERVICE.service" 2>/dev/null
  sudo systemctl start "$SERVICE.service" --no-block
}

echo "Updating with git"
# remember credentials
git config --global credential.helper store

# set repository as safe
git config --global --add safe.directory $SCRIPT_DIR

# fetch latest changes
git -C "$SCRIPT_DIR" fetch origin

# check for repository changes
BEHIND=$(git -C "$SCRIPT_DIR" rev-list --count HEAD..origin/main)
if [ "$BEHIND" -gt 0 ]; then
    echo "Branch main has changed ($BEHIND commits behind)"

  if [ -n "$(git -C "$SCRIPT_DIR" diff --name-only HEAD..origin/main -- "shared")" ]; then
    echo "Changes in shared/"
    setup_restart_service shared pairing_sync root
  fi

  if [ -n "$(git -C "$SCRIPT_DIR" diff --name-only HEAD..origin/main -- "dashboard")" ]; then
    echo "Changes in dashboard/"
    setup_restart_service dashboard pairing_dashboard "$REAL_USER"
  fi

  if [ -n "$(git -C "$SCRIPT_DIR" diff --name-only HEAD..origin/main -- "simulation")" ]; then
    echo "Changes in simulation/"
    setup_restart_service simulation pairing_simulation "$REAL_USER"
  fi
fi

service_exists() {
  SERVICE_TO_CHECK=$1
  systemctl list-unit-files "$SERVICE_TO_CHECK" &>/dev/null
}

# first setup, no service there yet
if ! (service_exists pairing_sync && service_exists pairing_dashboard && service_exists pairing_simulation); then
    echo "Missing services, starting all"
    setup_restart_service shared pairing_sync root
    setup_restart_service dashboard pairing_dashboard "$REAL_USER"
    setup_restart_service simulation pairing_simulation "$REAL_USER"
fi

echo "Finish"
