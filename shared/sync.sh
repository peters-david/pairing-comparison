#!/usr/bin/env bash
# fail on error
set -e

# get path
SCRIPT_DIR="$(dirname "$(realpath "${BASH_SOURCE[0]}")")"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# copy all simulation files (if not existent)
echo "Copying simulation result files"
mkdir -p "$PROJECT_DIR/dashboard/results/"
cp -rnv -t "$PROJECT_DIR/dashboard/results/" $PROJECT_DIR/simulation/.??????????????/

# function to install packages that are not there yet
install_if_missing() {
    PACKAGE=$1
    if ! dpkg -s "$PACKAGE" &> /dev/null; then
        echo "Installing $PACKAGE"
        sudo apt update
        sudo apt install -y "$PACKAGE"
    fi
}

echo "Ensuring dependencies"
# install required dependencies
install_if_missing findutils
install_if_missing jq

# add all files to index
echo "Creating file index"
find $PROJECT_DIR/dashboard/results/.??????????????/ -type f | sed "s|.*/dashboard/||" | jq -R -s 'split("\n")[:-1]' > "$PROJECT_DIR/dashboard/results/index.json"
echo "Finished"
