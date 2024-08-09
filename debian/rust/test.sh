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
PACKAGE_NAME="rust"
PACKAGE_SHOW_NAME="Rust"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Function to check if Rust is installed
is_rust_installed() {
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        log "Rust is installed."
        return 0
    else
        log "Rust is not installed."
        return 1
    fi
}

# Function to install Rust
install_rust() {
    log "Attempting to install Rust..."
    if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        error_exit "Failed to install Rust."
    fi
    source $HOME/.cargo/env
    log "Rust installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check for curl
    if ! command -v curl >/dev/null 2>&1; then
        error_exit "curl is not installed. Please install curl and try again."
    fi

    # Check for gcc
    if ! command -v gcc >/dev/null 2>&1; then
        error_exit "gcc is not installed. Please install build-essential and try again."
    fi

    log "System prerequisites check passed."
}

# Function to test Rust functionality
test_rust_functionality() {
    local temp_dir
    temp_dir=$(mktemp -d) || error_exit "Failed to create temporary directory."
    log "Created temporary directory: $temp_dir"

    cd "$temp_dir"

    # Create a new Rust project
    log "Creating a new Rust project..."
    if ! cargo new hello_world; then
        rm -rf "$temp_dir"
        error_exit "Failed to create Rust project."
    fi

    cd hello_world

    # Build the project
    log "Building the Rust project..."
    if ! cargo build; then
        rm -rf "$temp_dir"
        error_exit "Failed to build Rust project."
    fi

    # Run the project
    log "Running the Rust project..."
    if ! output=$(cargo run 2>&1); then
        log "Failed to run Rust project. Output:"
        log "$output"
        rm -rf "$temp_dir"
        return 1
    fi

    log "Project output: $output"

    # Clean up
    cd ..
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    # After cleaning up:
    log "Cleaned up temporary directory."
    cd "$original_dir"
    log "Changed back to original directory: $original_dir"

    # Check if the output is as expected
    if [[ "$output" == *"Hello, world!"* ]]; then
        log "Rust test passed successfully."
        return 0
    else
        log "Unexpected output from Rust project."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local rust_version
    local cargo_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    rust_version=$(rustc --version | awk '{print $2}') || rust_version="Unknown"
    cargo_version=$(cargo --version | awk '{print $2}') || cargo_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$rust_version",
    "cargo_version": "$cargo_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "Rust Functionality Test",
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
    log "Starting Rust test script..."

    check_prerequisites

    if ! is_rust_installed; then
        install_rust
    fi

    if test_rust_functionality; then
        log "Rust is functioning correctly."
        generate_report true
    else
        log "Rust is not functioning correctly."
        generate_report false
    fi

    log "Rust test script completed."
}

# Run the main function
main