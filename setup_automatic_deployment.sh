#!/usr/bin/env bash
# fail on error
set -e

if [[ $EUID -ne 0 ]]; then
   echo "Run with sudo:" 1>&2
   echo "sudo bash setup_automatic_deployment.sh" 1>&2
   exit 1
fi

# get user
REAL_USER="${SUDO_USER:-$USER}"

# install rust
echo "Checking for rust"
sudo -H -u "$REAL_USER" bash -c '
        export CARGO_HOME="$HOME/.cargo"
        export RUSTUP_HOME="$HOME/.rustup"
        export PATH="$HOME/.cargo/bin:$PATH"
        if ! command -v rustup >/dev/null 2>&1; then
            echo "Rust not present, installing"
            curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
        fi
        source "$HOME/.cargo/env"
    '

# get path
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
export DIR="$SCRIPT_DIR"

# setup deployment service
echo "Adding deployment service"
sudo -E envsubst < "$SCRIPT_DIR/pairing_deployment.service.template" > /etc/systemd/system/pairing_deployment.service
sudo chmod 644 /etc/systemd/system/pairing_deployment.service
sudo systemctl daemon-reload
sudo systemctl enable pairing_deployment.service
echo "Starting deployment service"
sudo systemctl start pairing_deployment.service --no-block
echo "Finished"
