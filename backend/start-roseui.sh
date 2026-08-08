#!/usr/bin/env bash
# Start the RoseUi backend (rosecore) with a FIXED data directory and a STABLE
# JWT secret so stored provider API keys survive restarts.
#
# Why this matters:
#   rosecore defaults `--data-dir` to the RELATIVE path "data", which resolves
#   against the current working directory. Launching from different folders
#   created different roseui-backend.db files, making provider config (and its
#   encrypted API keys) appear to "vanish" after a rebuild/restart. Pinning an
#   absolute data dir + a constant JWT_SECRET eliminates that drift: the
#   encryption key is always derived from the same secret, so existing API-key
#   ciphertext keeps decrypting.
#
# Usage:
#   ./start-roseui.sh          # start in background (owner mode, 127.0.0.1:3080)
#   ./start-roseui.sh --stop   # stop the running instance
#   ./start-roseui.sh --restart

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data"
BIN="$SCRIPT_DIR/target/debug/dodiddoneui"
PID_FILE="$SCRIPT_DIR/.dodiddoneui.pid"
LOG_FILE="$SCRIPT_DIR/dodiddoneui.log"

# Stable JWT secret — MUST stay constant. It is the source for the API-key
# encryption key; rotating it makes all stored API keys undecryptable.
export JWT_SECRET="***REMOVED_JWT_SECRET***"

mkdir -p "$DATA_DIR"

stop_instance() {
  if [[ -f "$PID_FILE" ]]; then
    local pid
    pid="$(cat "$PID_FILE" 2>/dev/null || true)"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "Stopping dodiddoneui (pid $pid)..."
      kill "$pid" 2>/dev/null || true
      sleep 1
    fi
    rm -f "$PID_FILE"
  fi
  # Fallback: any dodiddoneui on :3080
  local port_pid
  port_pid="$(lsof -ti :3080 2>/dev/null || true)"
  if [[ -n "$port_pid" ]]; then
    echo "Stopping dodiddoneui on :3080 (pid $port_pid)..."
    kill "$port_pid" 2>/dev/null || true
    sleep 1
  fi
}

case "${1:-}" in
  --stop)
    stop_instance
    echo "Stopped."
    exit 0
    ;;
  --restart)
    stop_instance
    ;;
esac

if [[ ! -x "$BIN" ]]; then
  echo "Binary not found at $BIN — run 'cargo build -p roseui-app' first." >&2
  exit 1
fi

# Avoid double-start
if [[ -f "$PID_FILE" ]]; then
  existing="$(cat "$PID_FILE" 2>/dev/null || true)"
  if [[ -n "$existing" ]] && kill -0 "$existing" 2>/dev/null; then
    echo "dodiddoneui already running (pid $existing). Use --restart to replace."
    exit 0
  fi
fi

echo "Starting dodiddoneui (data-dir=$DATA_DIR)..."
nohup "$BIN" --port 3080 --host 127.0.0.1 --identity-mode owner --data-dir "$DATA_DIR" \
  > "$LOG_FILE" 2>&1 &
echo $! > "$PID_FILE"
disown
sleep 4
if lsof -ti :3080 >/dev/null 2>&1; then
  echo "dodiddoneui is up on http://127.0.0.1:3080 (pid $(cat "$PID_FILE"))"
else
  echo "dodiddoneui did not come up. Check $LOG_FILE"
  exit 1
fi
