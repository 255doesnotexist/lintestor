#!/bin/bash

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

set -euo pipefail
original_dir=$(pwd)
log "Current directory: $original_dir"

temp_dir=$(mktemp -d)
log "Created temporary directory: $temp_dir"
cd "$temp_dir"

# Define the package details
PACKAGE_NAME="sqlite3"

# Function to check if SQLite is installed
is_sqlite_installed() {
    if command -v sqlite3 >/dev/null 2>&1; then
        log "SQLite is installed."
        return 0
    else
        log "SQLite is not installed."
        return 1
    fi
}

# Function to install SQLite
install_sqlite() {
    log "Attempting to install SQLite..."
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y sqlite3
    
    if ! is_sqlite_installed; then
        echo "SQLite installation failed. The 'sqlite3' command is still not available."
        return 1
    fi

    log "SQLite installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    # No specific prerequisites for SQLite
    log "System prerequisites check passed."
}

# Function to test SQLite functionality
test_sqlite_functionality() {
    local db_file="test.db"
    local output

    log "Creating test database..."
    sqlite3 "$db_file" <<EOF
CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT);
INSERT INTO test (name) VALUES ('Alice');
INSERT INTO test (name) VALUES ('Bob');
SELECT * FROM test;
.quit
EOF

    log "Querying test database..."
    output=$(sqlite3 "$db_file" "SELECT * FROM test;")

    log "Database output:"
    log "$output"

    # Check if the output contains expected data
    if [[ "$output" == *"1|Alice"* && "$output" == *"2|Bob"* ]]; then
        log "SQLite test passed successfully."
        return 0
    else
        log "Unexpected output from SQLite test."
        return 1
    fi
}

# Main script execution
main() {
    log "Starting SQLite test script..."

    
    if !check_prerequisites; then
        return 1;
    fi

    if !is_sqlite_installed; then
        if !install_sqlite; then
            return 1;
        fi
    fi

    PACKAGE_VERSION=$(sqlite3 --version | awk '{print $1}') || PACKAGE_VERSION="Unknown"

    # Clean up
    cd "$original_dir"
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."


    if test_sqlite_functionality; then
        log "SQLite is functioning correctly."
        return 0
    else
        log "SQLite is not functioning correctly."
        return 1
    fi
}

# Run the main function
main
rm test.db