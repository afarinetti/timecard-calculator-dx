# Timecard Calculator

set shell := ["/bin/sh", "-c"]

# Default recipe — list available commands
[private]
default:
    @just --list

# ── Development ──

# Run clippy on all targets
lint:
    cargo clippy --all-targets --all-features

# Run all tests
test:
    cargo test

# ── SQLX Offline Cache ──

# Update sqlx offline query cache after adding / changing queries.
# This temporarily swaps the api .cargo/config.toml to use an absolute
# database path and disables SQLX_OFFLINE, runs `cargo sqlx prepare`,
# then restores the original config.
sqlx-prepare:
    #!/usr/bin/env bash
    set -euo pipefail

    API_DIR="packages/api"
    CONFIG="$(pwd)/$API_DIR/.cargo/config.toml"
    DB="$API_DIR/dev.db"
    MIGRATIONS="$API_DIR/migrations"

    if ! command -v cargo-sqlx &>/dev/null; then
        echo "cargo-sqlx not found. Install with: cargo install sqlx-cli --features sqlite"
        exit 1
    fi

    # Ensure dev.db exists and migrations are applied
    if [ ! -f "$DB" ]; then
        echo "Creating $DB ..."
        touch "$DB"
    fi
    echo "Running migrations ..."
    DATABASE_URL="sqlite:$(pwd)/$DB" cargo sqlx migrate run --source "$MIGRATIONS"

    # Backup original config
    cp "$CONFIG" "$CONFIG.bak"
    trap 'mv "$CONFIG.bak" "$CONFIG"' EXIT

    # Temporarily rewrite config with absolute DB path + offline=false
    {
        echo '[env]'
        echo "DATABASE_URL = { value = \"sqlite://$(pwd)/$DB\", force = true }"
        echo 'SQLX_OFFLINE = { value = "false", force = true }'
    } > "$CONFIG"

    echo "Generating sqlx offline query cache ..."
    cmd='cargo sqlx prepare -- --lib'
    cd "$API_DIR" && eval "$cmd"

    echo "Done. Query cache updated in $API_DIR/.sqlx/"
