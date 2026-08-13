#!/usr/bin/env bash
# Start the dodiddoneui backend with a FIXED data directory and a STABLE
# JWT secret so stored provider API keys survive restarts.
#
# Why this matters:
#   dodiddoneui defaults `--data-dir` to the RELATIVE path "data", which resolves
#   against the current working directory. Launching from different folders
#   created different roseui-backend.db files, making provider config (and its
#   encrypted API keys) appear to "vanish" after a rebuild/restart. Pinning an
#   absolute data dir eliminates that drift: the DB is always at the same path,
#   so existing API-key ciphertext keeps decrypting.
#
# Secret handling (SECURITY):
#   The JWT secret that encrypts stored provider API keys is NEVER hardcoded
#   here. It is resolved at runtime in priority order:
#     1. $JWT_SECRET env var (sourced from .env below if present)
#     2. the persisted `jwt_secret` column in the system user row (auto-generated
#        on first launch and reused thereafter)
#   If neither is set, dodiddoneui generates a random secret, persists it to the
#   DB, and reuses it on every later start. Never commit a real secret — keep it
#   in backend/.env (gitignored). See backend/.env.example for the template.
#
# Usage:
#   ./start-roseui.sh            # start DEV build (target/debug + disk dist), background
#   ./start-roseui.sh --release  # start RELEASE build (target/release, embedded dist)
#   ./start-roseui.sh --stop     # stop the running instance
#   ./start-roseui.sh --restart  # restart (applies current mode: dev or --release)
#
# Mode notes:
#   dev   (default): uses target/debug/dodiddoneui and serves the on-disk frontend
#                    dist via --static-dir (frontend/.../dist). Good for iterating
#                    without recompiling the binary.
#   --release:       uses target/release/dodiddoneui with the frontend embedded at
#                    compile time (rust-embed). This is the SHIPPED/USER form.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/data"
MODE="debug"
BIN="$SCRIPT_DIR/target/debug/dodiddoneui"
PID_FILE="$SCRIPT_DIR/.dodiddoneui.pid"
LOG_FILE="$SCRIPT_DIR/dodiddoneui.log"

# Load JWT secret from backend/.env if present (gitignored — never committed).
# If absent, dodiddoneui auto-generates and persists a random secret on first
# launch, then reuses it on every subsequent start. See .env.example.
if [[ -f "$SCRIPT_DIR/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "$SCRIPT_DIR/.env"
  set +a
fi

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
  --release)
    MODE="release"
    ;;
  --stop)
    stop_instance
    echo "Stopped."
    exit 0
    ;;
  --restart)
    stop_instance
    ;;
  "")
    # default dev mode, nothing to do
    ;;
  *)
    echo "Unknown argument: $1" >&2
    echo "Usage: $0 [--release|--stop|--restart]" >&2
    exit 1
    ;;
esac

if [[ "$MODE" == "release" ]]; then
  BIN="$SCRIPT_DIR/target/release/dodiddoneui"
else
  BIN="$SCRIPT_DIR/target/debug/dodiddoneui"
fi

if [[ ! -x "$BIN" ]]; then
  if [[ "$MODE" == "release" ]]; then
    echo "Release binary not found at $BIN — run './build.sh' (or 'cargo build --release -p roseui-app') first." >&2
  else
    echo "Debug binary not found at $BIN — run 'cargo build -p roseui-app' first." >&2
  fi
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

echo "Starting dodiddoneui [$MODE] (data-dir=$DATA_DIR)..."
if [[ "$MODE" == "release" ]]; then
  # SHIPPED form: frontend is embedded at compile time (rust-embed). No --static-dir.
  nohup "$BIN" --port 3080 --host 127.0.0.1 --identity-mode owner --data-dir "$DATA_DIR" \
    > "$LOG_FILE" 2>&1 &
else
  # DEV form: serve the freshly built frontend from disk instead of the embedded
  # build (embedded assets only refresh by recompiling the Rust binary; the built
  # renderer dist lives at frontend/packages/desktop/src/renderer/dist).
  STATIC_DIR="$SCRIPT_DIR/../frontend/packages/desktop/src/renderer/dist"
  nohup "$BIN" --port 3080 --host 127.0.0.1 --identity-mode owner --data-dir "$DATA_DIR" --static-dir "$STATIC_DIR" \
    > "$LOG_FILE" 2>&1 &
fi
echo $! > "$PID_FILE"
disown
sleep 4
if lsof -ti :3080 >/dev/null 2>&1; then
  echo "dodiddoneui [$MODE] is up on http://127.0.0.1:3080 (pid $(cat "$PID_FILE"))"
else
  echo "dodiddoneui did not come up. Check $LOG_FILE"
  exit 1
fi
