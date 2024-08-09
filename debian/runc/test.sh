#!/bin/bash

set -euo pipefail

# Define the package details
PACKAGE_NAME="runc"
PACKAGE_SHOW_NAME="runC"
PACKAGE_TYPE="Container Runtime"
REPORT_FILE="report.json"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
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
    log "Attempting to install runC..."
    if ! apt-get update; then
        error_exit "Failed to update package lists."
    fi
    if ! apt-get install -y runc; then
        error_exit "Failed to install runC."
    fi
    log "runC installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check if running as root
    if [[ $EUID -ne 0 ]]; then
        error_exit "This script must be run as root."
    fi

    # Check kernel version
    local kernel_version
    kernel_version=$(uname -r)
    log "Kernel version: $kernel_version"

    # Check /proc mount
    if ! mount | grep -q "proc on /proc type proc"; then
        log "WARNING: /proc is not mounted correctly. Attempting to remount..."
        if ! mount -t proc proc /proc; then
            error_exit "Failed to mount /proc."
        fi
    fi

    # Check namespaces support
    if [[ ! -d /proc/self/ns ]]; then
        log "ERROR: /proc/self/ns directory not found."
        log "Kernel namespaces configuration:"
        grep CONFIG_NAMESPACES /boot/config-"$(uname -r)" || log "Unable to find namespace configuration"
        log "Contents of /proc/self:"
        ls -l /proc/self || log "Unable to list /proc/self"
        error_exit "Namespace support is not available."
    fi

    # Check specific namespaces
    for ns in ipc mnt net pid user uts; do
        if [[ ! -e /proc/self/ns/$ns ]]; then
            log "WARNING: $ns namespace is not available"
        fi
    done

    # Check cgroups support
    if [[ ! -d /sys/fs/cgroup ]]; then
        error_exit "Cgroups are not available."
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

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local runc_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    runc_version=$(runc --version | awk '/runc version/{print $3}') || runc_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "$(cat /etc/os-release | grep ID | cut -d'=' -f2 | tr -d '"')",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$runc_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "runC Functionality Test",
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
    log "Starting runC test script..."

    check_prerequisites

    if ! is_runc_installed; then
        install_runc_package
    fi

    if test_runc_functionality; then
        log "runC is functioning correctly."
        generate_report true
    else
        log "runC is not functioning correctly."
        generate_report false
    fi

    log "runC test script completed."
}

# Run the main function
main