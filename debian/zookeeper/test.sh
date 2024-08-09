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
PACKAGE_NAME="zookeeper"
PACKAGE_SHOW_NAME="Apache ZooKeeper"
PACKAGE_TYPE="Distributed Coordination Service"
REPORT_FILE="report.json"

# Error handling function
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Function to check if ZooKeeper is installed
is_zookeeper_installed() {
    if [ -d "/opt/zookeeper" ] && [ -f "/opt/zookeeper/bin/zkServer.sh" ]; then
        log "ZooKeeper is installed. Path: /opt/zookeeper"
        return 0
    else
        log "ZooKeeper is not installed."
        return 1
    fi
}

# Function to install ZooKeeper
install_zookeeper() {
    log "Attempting to install ZooKeeper..."
    
    # Install Java if not already installed
    if ! command -v java >/dev/null 2>&1; then
        log "Java not found. Installing OpenJDK..."
        sudo apt-get update
        if ! sudo apt-get install -y openjdk-*-jdk; then
            error_exit "Failed to install Java."
        fi
    fi

    # Download and install ZooKeeper
    local zk_version="3.9.2"
    local zk_url="https://dlcdn.apache.org/zookeeper/zookeeper-${zk_version}/apache-zookeeper-${zk_version}-bin.tar.gz"
    
    if ! wget "$zk_url" -O zookeeper.tar.gz; then
        error_exit "Failed to download ZooKeeper."
    fi

    if ! sudo tar -xzf zookeeper.tar.gz -C /opt; then
        error_exit "Failed to extract ZooKeeper."
    fi

    if ! sudo mv /opt/apache-zookeeper-${zk_version}-bin /opt/zookeeper; then
        error_exit "Failed to rename ZooKeeper directory."
    fi

    # Configure ZooKeeper
    if ! sudo cp /opt/zookeeper/conf/zoo_sample.cfg /opt/zookeeper/conf/zoo.cfg; then
        error_exit "Failed to create ZooKeeper configuration."
    fi

    log "ZooKeeper installation completed. Verifying installation..."
    if ! is_zookeeper_installed; then
        error_exit "ZooKeeper installation failed. ZooKeeper is not available at the expected location."
    fi
    log "ZooKeeper installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    if ! command -v java >/dev/null 2>&1; then
        log "Java is not installed. It will be installed during ZooKeeper installation."
    fi
    log "System prerequisites check passed."
}

# Function to test ZooKeeper functionality
test_zookeeper_functionality() {
    log "Starting ZooKeeper server..."
    if ! sudo /opt/zookeeper/bin/zkServer.sh start; then
        log "Failed to start ZooKeeper server."
        return 1
    fi

    sleep 5  # Give some time for ZooKeeper to start

    log "Checking ZooKeeper server status..."
    if ! sudo /opt/zookeeper/bin/zkServer.sh status; then
        log "ZooKeeper server is not running properly."
        return 1
    fi

    log "Testing ZooKeeper client connection..."
    echo "stat" | nc localhost 2181 > /dev/null
    if [ $? -ne 0 ]; then
        log "Failed to connect to ZooKeeper server."
        return 1
    fi

    log "Stopping ZooKeeper server..."
    if ! sudo /opt/zookeeper/bin/zkServer.sh stop; then
        log "Failed to stop ZooKeeper server."
        return 1
    fi

    log "ZooKeeper functionality test passed successfully."
    return 0
}

# Function to generate the report.json
generate_report() {
    local test_passed=$1
    local os_version
    local kernel_version
    local zookeeper_version

    os_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2) || os_version="Unknown"
    kernel_version=$(uname -r) || kernel_version="Unknown"
    zookeeper_version=$(/opt/zookeeper/bin/zkServer.sh version 2>&1 | grep -oP 'version: \K[0-9.]+' | head -1) || zookeeper_version="Unknown"

    local report_content
    report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$zookeeper_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "ZooKeeper Functionality Test",
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
    log "Starting ZooKeeper test script..."

    check_prerequisites

    if ! is_zookeeper_installed; then
        install_zookeeper
    fi

    log "Verifying ZooKeeper installation again..."
    if ! is_zookeeper_installed; then
        error_exit "ZooKeeper installation verification failed."
    fi

    cd "$original_dir"
    if test_zookeeper_functionality; then
        log "ZooKeeper is functioning correctly."
        generate_report true
    else
        log "ZooKeeper is not functioning correctly."
        generate_report false
    fi

    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    log "ZooKeeper test script completed."
}

# Run the main function
main