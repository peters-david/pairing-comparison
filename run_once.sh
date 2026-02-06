#!/usr/bin/env bash
set -euo pipefail

if (( $# < 1 || $# > 3 )); then
  echo "Usage: . run_once.sh [dashboard] [sync] [simulation]"
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

pids=()

# start scripts
for arg in "$@"; do
  case "$arg" in
    dashboard)
      cd "$SCRIPT_DIR/dashboard"
      ./start.sh &
      pids+=($!)
      ;;
    sync)
      cd "$SCRIPT_DIR/shared"
      ./sync.sh &
      pids+=($!)
      ;;
    simulation)
      cd "$SCRIPT_DIR/simulation"
      ./start.sh &
      pids+=($!)
      ;;
    *)
      echo "$arg can not be started, only dashboard, sync and simulation exist"
      exit 1
      ;;
  esac
done

# kill children on sigterm
trap 'kill "${pids[@]}" 2>/dev/null' SIGINT SIGTERM

# wait until finished
wait
