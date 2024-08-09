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
PACKAGE_NAME="varnish"
PACKAGE_SHOW_NAME="Varnish"
PACKAGE_TYPE="HTTP Cache"
REPORT_FILE="report.json"

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Function to check if Varnish is installed
is_varnish_installed() {
    if command -v varnishd >/dev/null 2>&1; then
        log "Varnish is installed. Path: $(which varnishd)"
        return 0
    else
        log "Varnish is not installed."
        return 1
    fi
}

# Function to install Varnish
install_varnish() {
    log "Attempting to install Varnish..."
    apt-get update
    if ! apt-get install -y varnish; then
        error_exit "Failed to install Varnish."
    fi
    log "Varnish installation command completed. Verifying installation..."
    if ! is_varnish_installed; then
        error_exit "Varnish installation failed. The 'varnishd' command is still not available."
    fi
    log "Varnish installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    # No specific prerequisites for Varnish
    log "System prerequisites check passed."
}

# Function to test Varnish functionality
test_varnish_functionality() {
    log "Starting Varnish service..."
    if ! systemctl start varnish; then
        log "Failed to start Varnish service."
        return 1
    fi

    log "Checking Varnish service status..."
    if ! systemctl is-active --quiet varnish; then
        log "Varnish service is not active."
        return 1
    fi

    log "Testing Varnish configuration..."
    if ! varnishd -C -f /etc/varnish/default.vcl; then
        log "Varnish configuration test failed."
        return 1
    fi

    log "Checking Varnish stats..."
    if ! varnishstat -1; then
        log "Failed to retrieve Varnish stats."
        return 1
    fi

    log "Stopping Varnish service..."
    if ! systemctl stop varnish; then
        log "Failed to stop Varnish service."
        return 1
    fi

    log "Varnish functionality test passed successfully."
    return 0
}

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local varnish_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    varnish_version=$(varnishd -V 2>&1 | grep -oP 'varnish-\K\d+\.\d+\.\d+' | head -1) || varnish_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$varnish_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "Varnish Functionality Test",
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
    log "Starting Varnish test script..."

    check_prerequisites

    if ! is_varnish_installed; then
        install_varnish
    fi

    log "Verifying Varnish installation again..."
    if ! is_varnish_installed; then
        error_exit "Varnish installation verification failed."
    fi

    cd "$original_dir"
    if test_varnish_functionality; then
        log "Varnish is functioning correctly."
        generate_report true
    else
        log "Varnish is not functioning correctly."
        generate_report false
    fi

    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    log "Varnish test script completed."
}

# Run the main function
main