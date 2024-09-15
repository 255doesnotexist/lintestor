#!/bin/bash

set -euo pipefail

# Define the package details
PACKAGE_NAME="runc"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to check if runc is installed
is_runc_installed() {
    if command -v runc >/dev/null 2>&1; then
        log "runC is installed."
        return 0
    else
        log "runC is not installed."
        return 1
    fi
}

# Function to install runc package
install_runc_package() {
    export DEBIAN_FRONTEND=noninteractive
    log "Attempting to install runC..."
    if ! apt-get update; then
        echo "Failed to update package lists."
        return 1
    fi
    if ! apt-get install -y runc; then
        echo "Failed to install runC."
        return 1
    fi
    log "runC installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        echo "This script must be run as root."
        return 1
    fi

    # Check kernel version
    local kernel_version
    kernel_version=$(uname -r)
    log "Kernel version: $kernel_version"

    # Check /proc mount
    if ! mount | grep -q "proc on /proc type proc"; then
        log "WARNING: /proc is not mounted correctly. Attempting to remount..."
        if ! mount -t proc proc /proc; then
            echo "Failed to mount /proc."
            return 1
        fi
    fi

    # Check namespaces support
    if [[ ! -d /proc/self/ns ]]; then
        log "ERROR: /proc/self/ns directory not found."
        log "Kernel namespaces configuration:"
        grep CONFIG_NAMESPACES /boot/config-"$(uname -r)" || log "Unable to find namespace configuration"
        log "Contents of /proc/self:"
        ls -l /proc/self || log "Unable to list /proc/self"
        echo "Namespace support is not available."
        return 1
    fi

    # Check specific namespaces
    for ns in ipc mnt net pid user uts; do
        if [[ ! -e /proc/self/ns/$ns ]]; then
            log "WARNING: $ns namespace is not available"
        fi
    done

    # Check cgroups support
    if [[ ! -d /sys/fs/cgroup ]]; then
        echo "Cgroups are not available."
        return 1
    fi

    # Check unshare command
    if ! unshare --fork --pid --mount-proc sleep 1; then
        log "WARNING: unshare command failed. This might indicate issues with namespace support."
    fi

    log "System prerequisites check completed."
}

# Function to test runc functionality
test_runc_functionality() {
    local temp_dir
    temp_dir=$(mktemp -d) || error_exit "Failed to create temporary directory."
    log "Created temporary directory: $temp_dir"

    local bundle_dir="${temp_dir}/bundle"
    local rootfs_dir="${bundle_dir}/rootfs"

    mkdir -p "$rootfs_dir" || error_exit "Failed to create rootfs directory."

    # Create a simple config.json
    cat > "${bundle_dir}/config.json" <<EOF
{
    "ociVersion": "1.0.1-dev",
    "process": {
        "terminal": false,
        "user": {
            "uid": 0,
            "gid": 0
        },
        "args": [
            "echo",
            "Hello from runc!"
        ],
        "env": [
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        ],
        "cwd": "/"
    },
    "root": {
        "path": "rootfs",
        "readonly": true
    },
    "linux": {
        "namespaces": [
            {"type": "pid"},
            {"type": "ipc"},
            {"type": "uts"},
            {"type": "mount"}
        ]
    }
}
EOF

    log "Created config.json in $bundle_dir"

    # Run the container
    log "Attempting to run runc container..."
    if ! output=$(runc run --bundle "$bundle_dir" test-container 2>&1); then
        log "Failed to run runc container. Output:"
        log "$output"
        rm -rf "$temp_dir"
        return 1
    fi

    log "Container output: $output"

    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    # Check if the output is as expected
    if [[ "$output" == "Hello from runc!" ]]; then
        log "runC test passed successfully."
        return 0
    else
        log "Unexpected output from runc container."
        return 1
    fi
}

# Main script execution
main() {
    log "Starting runC test script..."

    if ! check_prerequisites; then
        return 1
    fi

    if ! is_runc_installed; then
        if ! install_runc_package; then
            return 1
        fi
    fi

    PACKAGE_VERSION=$(runc --version | awk '/runc version/{print $3}') || PACKAGE_VERSION="Unknown"
    
    if test_runc_functionality; then
        log "runC is functioning correctly."
        return 0
    else
        log "runC is not functioning correctly."
        return 1
    fi
}

# Run the main function
main