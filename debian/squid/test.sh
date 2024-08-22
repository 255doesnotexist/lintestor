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
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    if ! apt-get install -y squid; then
        echo "Failed to install Squid."
        return 1
    fi
    log "Squid installation command completed. Verifying installation..."
    if ! is_squid_installed; then
        echo "Squid installation failed. The 'squid' command is still not available."
        return 1
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

# Main script execution
main() {
    log "Starting Squid test script..."

    if !check_prerequisites; then
        return 1
    fi

    if !is_squid_installed; then
        if !install_squid; then
            return 1
        fi
    fi

    log "Verifying Squid installation again..."
    if ! is_squid_installed; then
        echo "Squid installation verification failed."
        return 1
    fi

    PACKAGE_VERSION=$(squid -v | head -n1 | awk '{print $4}') || PACKAGE_VERSION="Unknown"

    cd "$original_dir"
    if test_squid_functionality; then
        log "Squid is functioning correctly."
        return 0
    else
        log "Squid is not functioning correctly."
        return 1
    fi
}

# Run the main function
main

# Clean up
rm -rf "$temp_dir"
log "Cleaned up temporary directory."
log "Squid test script completed."