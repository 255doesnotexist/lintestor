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
PACKAGE_NAME="squid"
PACKAGE_SHOW_NAME="Squid"
PACKAGE_TYPE="Proxy Server"
REPORT_FILE="report.json"

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Function to check if Squid is installed
is_squid_installed() {
    if command -v squid >/dev/null 2>&1; then
        log "Squid is installed. Path: $(which squid)"
        return 0
    else
        log "Squid is not installed."
        return 1
    fi
}

# Function to install Squid
install_squid() {
    log "Attempting to install Squid..."
    apt-get update
    if ! apt-get install -y squid; then
        error_exit "Failed to install Squid."
    fi
    log "Squid installation command completed. Verifying installation..."
    if ! is_squid_installed; then
        error_exit "Squid installation failed. The 'squid' command is still not available."
    fi
    log "Squid installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    # No specific prerequisites for Squid
    log "System prerequisites check passed."
}

# Function to test Squid functionality
test_squid_functionality() {
    log "Starting Squid service..."
    if ! sudo systemctl start squid; then
        log "Failed to start Squid service."
        return 1
    fi

    log "Checking Squid service status..."
    if ! sudo systemctl is-active --quiet squid; then
        log "Squid service is not active."
        return 1
    fi

    log "Testing Squid configuration..."
    if ! sudo squid -k parse; then
        log "Squid configuration test failed."
        return 1
    fi

    log "Stopping Squid service..."
    if ! sudo systemctl stop squid; then
        log "Failed to stop Squid service."
        return 1
    fi

    log "Squid functionality test passed successfully."
    return 0
}

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local squid_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    squid_version=$(squid -v | head -n1 | awk '{print $4}') || squid_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$squid_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "Squid Functionality Test",
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
    log "Starting Squid test script..."

    check_prerequisites

    if ! is_squid_installed; then
        install_squid
    fi

    log "Verifying Squid installation again..."
    if ! is_squid_installed; then
        error_exit "Squid installation verification failed."
    fi

    cd "$original_dir"
    if test_squid_functionality; then
        log "Squid is functioning correctly."
        generate_report true
    else
        log "Squid is not functioning correctly."
        generate_report false
    fi

    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    log "Squid test script completed."
}

# Run the main function
main