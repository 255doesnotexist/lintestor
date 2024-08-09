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
PACKAGE_SHOW_NAME="SQLite"
PACKAGE_TYPE="Database"
REPORT_FILE="report.json"

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
}

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
    apt-get update
    apt-get install -y sqlite3
    
    if ! is_sqlite_installed; then
        error_exit "SQLite installation failed. The 'sqlite3' command is still not available."
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

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local sqlite_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    sqlite_version=$(sqlite3 --version | awk '{print $1}') || sqlite_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$sqlite_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "SQLite Functionality Test",
            "passed": $test_passed
        }
    ],
    "all_tests_passed": $test_passed
}
EOF
)

    echo "$report_content" > "$REPORT_FILE"
    log "Report generated at $REPORT_FILE"
}

# Main script execution
main() {
    log "Starting SQLite test script..."

    check_prerequisites

    if ! is_sqlite_installed; then
        install_sqlite
    fi

    # Clean up
    cd "$original_dir"
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."


    if test_sqlite_functionality; then
        log "SQLite is functioning correctly."
        generate_report true
    else
        log "SQLite is not functioning correctly."
        generate_report false
    fi

    log "SQLite test script completed."
}

# Run the main function
main
rm test.db