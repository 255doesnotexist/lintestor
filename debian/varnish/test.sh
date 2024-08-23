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
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    if ! apt-get install -y varnish; then
        echo "Failed to install Varnish."
        return 1
    fi
    log "Varnish installation command completed. Verifying installation..."
    if ! is_varnish_installed; then
        echo "Varnish installation failed. The 'varnishd' command is still not available."
        return 1
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

# Main script execution
main() {
    log "Starting Varnish test script..."

    if ! check_prerequisites; then
        return 1
    fi

    if ! is_varnish_installed; then
        if ! install_varnish; then
            return 1
        fi
    fi

    log "Verifying Varnish installation again..."
    if ! is_varnish_installed; then
        echo "Varnish installation verification failed."
        return 1
    fi


    PACKAGE_VERSION=$(varnishd -V 2>&1 | grep -oP 'varnish-\K\d+\.\d+\.\d+' | head -1) || PACKAGE_VERSION="Unknown"

    cd "$original_dir"
    if test_varnish_functionality; then
        log "Varnish is functioning correctly."
        return 0
    else
        log "Varnish is not functioning correctly."
        return 1
    fi
}

# Run the main function
main

# Clean up
rm -rf "$temp_dir"
log "Cleaned up temporary directory."
log "Varnish test script completed."