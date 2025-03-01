#!/bin/bash

set -euo pipefail

# Define the package details
PACKAGE_NAME="runc"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Error exit function
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Function with timeout
run_with_timeout() {
    local timeout=$1
    local command="${@:2}"
    
    log "Running command with $timeout second timeout: $command"
    
    # Run the command in background
    eval "$command" &
    local pid=$!
    
    # Wait for command with timeout
    local timeout_happened=0
    (
        sleep $timeout
        kill -0 $pid 2>/dev/null && {
            log "Command timed out after $timeout seconds"
            kill -9 $pid 2>/dev/null
            timeout_happened=1
        }
    ) &
    local watchdog_pid=$!
    
    # Wait for the command to finish
    wait $pid 2>/dev/null || true
    local exit_code=$?
    
    # Kill the watchdog
    kill -9 $watchdog_pid 2>/dev/null || true
    
    if [ "$timeout_happened" -eq 1 ]; then
        return 124  # Standard timeout exit code
    fi
    
    return $exit_code
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
    if ! sudo dnf install -y runc; then
        log "Failed to install runC."
        return 1
    fi
    log "runC installed successfully."
    return 0
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    
    # Check kernel version
    local kernel_version
    kernel_version=$(uname -r)
    log "Kernel version: $kernel_version"

    # Check /proc mount
    if ! mount | grep -q "proc on /proc type proc"; then
        log "WARNING: /proc is not mounted correctly. Attempting to remount..."
        if ! sudo mount -t proc proc /proc; then
            log "Failed to mount /proc."
            return 1
        fi
    fi

    # Check namespaces support
    if [[ ! -d /proc/self/ns ]]; then
        log "ERROR: /proc/self/ns directory not found."
        log "Namespace support is not available."
        return 1
    fi

    # Check cgroups support
    if [[ ! -d /sys/fs/cgroup ]]; then
        log "Cgroups are not available."
        return 1
    fi

    log "System prerequisites check completed."
    return 0
}

# Create a minimal rootfs for testing
create_minimal_rootfs() {
    local rootfs_dir="$1"
    log "Creating minimal rootfs in $rootfs_dir..."

    # Create basic directory structure
    mkdir -p "$rootfs_dir"/{bin,dev,etc,lib,lib64,proc,sys}
    
    # Copy basic binaries
    for bin in sh ls echo cat; do
        if which "$bin" &>/dev/null; then
            cp "$(which $bin)" "$rootfs_dir/bin/" 2>/dev/null || true
        fi
    done
    
    # If busybox is available, use it
    if which busybox &>/dev/null; then
        cp "$(which busybox)" "$rootfs_dir/bin/"
        
        # Create symlinks for basic commands
        pushd "$rootfs_dir/bin" > /dev/null
        for cmd in sh ls echo cat; do
            ln -sf busybox "$cmd" 2>/dev/null || true
        done
        popd > /dev/null
    fi
    
    # Create /dev/null
    sudo mknod -m 666 "$rootfs_dir/dev/null" c 1 3 2>/dev/null || true
    
    # Create /etc/passwd with root user
    cat > "$rootfs_dir/etc/passwd" <<EOF
root:x:0:0:root:/root:/bin/sh
EOF
    
    # Create a simple init script
    cat > "$rootfs_dir/init" <<EOF
#!/bin/sh
echo "Container initialized"
exec "\$@"
EOF
    chmod +x "$rootfs_dir/init"

    log "Minimal rootfs created."
    return 0
}

# Function to test runc functionality with a simpler approach
test_runc_functionality() {
    local temp_dir
    temp_dir=$(mktemp -d) || error_exit "Failed to create temporary directory."
    log "Created temporary directory: $temp_dir"

    local bundle_dir="${temp_dir}/bundle"
    local rootfs_dir="${bundle_dir}/rootfs"

    mkdir -p "$rootfs_dir" || error_exit "Failed to create rootfs directory."

    # Create a minimal rootfs
    create_minimal_rootfs "$rootfs_dir"

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
            "/bin/echo",
            "Hello from runc!"
        ],
        "env": [
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
        ],
        "cwd": "/"
    },
    "root": {
        "path": "rootfs"
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

    # Run the container with timeout
    log "Attempting to run runc container..."
    local container_id="test-container-$(date +%s)"
    local output=""
    
    # Clean up any previous container with the same name
    sudo runc delete -f "$container_id" 2>/dev/null || true

    # Run with timeout to prevent hanging
    if ! output=$(run_with_timeout 10 "sudo runc run --bundle '$bundle_dir' '$container_id' 2>&1"); then
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log "runc execution timed out after 10 seconds"
        else
            log "Failed to run runc container. Exit code: $exit_code, Output:"
            log "$output"
        fi
        
        # Try to cleanup the container
        sudo runc delete -f "$container_id" 2>/dev/null || true
        
        rm -rf "$temp_dir"
        return 1
    fi

    log "Container output: $output"

    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    # Check if the output contains our message
    if [[ "$output" == *"Hello from runc!"* ]]; then
        log "runC test passed successfully."
        return 0
    else
        log "Unexpected output from runc container."
        return 1
    fi
}

# Alternative test that just checks runc version
test_runc_version() {
    log "Testing runC by checking its version..."
    local version_output
    
    if ! version_output=$(sudo runc --version 2>&1); then
        log "Failed to get runC version"
        return 1
    fi
    
    log "runC version output: $version_output"
    return 0
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

    # Get package version
    PACKAGE_VERSION=$(sudo runc --version | awk '/runc version/{print $3}' 2>/dev/null) || PACKAGE_VERSION="Unknown"
    log "runC version: $PACKAGE_VERSION"
    
    # First try a simple version check
    if ! test_runc_version; then
        log "Basic runC version check failed"
        return 1
    fi
    
    # Then try the full functionality test
    if test_runc_functionality; then
        log "runC is functioning correctly."
        return 0
    else
        log "Warning: Full container test failed, but version check passed."
        log "This may still indicate that runC is installed correctly but may have issues with container execution."
        return 1
    fi
}

# Run the main function and capture exit code
main
exit_code=$?
return $exit_code