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
        sudo dnf install -y java-11-openjdk
    fi

    # Download and install ZooKeeper
    local zk_version="3.9.3"
    local zk_url="https://dlcdn.apache.org/zookeeper/zookeeper-${zk_version}/apache-zookeeper-${zk_version}-bin.tar.gz"
    
    if ! curl -o zookeeper.tar.gz "$zk_url"; then
        # use curl since wget may not be present on certain systems
        echo "Failed to download ZooKeeper."
        return 1
    fi

    if ! sudo tar -xzf zookeeper.tar.gz -C /opt; then
        echo "Failed to extract ZooKeeper."
        return 1
    fi

    if ! sudo mv /opt/apache-zookeeper-${zk_version}-bin /opt/zookeeper; then
        echo "Failed to rename ZooKeeper directory."
        return 1
    fi

    # Configure ZooKeeper
    if ! sudo cp /opt/zookeeper/conf/zoo_sample.cfg /opt/zookeeper/conf/zoo.cfg; then
        echo "Failed to create ZooKeeper configuration."
        return 1
    fi

    log "ZooKeeper installation completed. Verifying installation..."
    if ! is_zookeeper_installed; then
        echo "ZooKeeper installation failed. ZooKeeper is not available at the expected location."
        return 1
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
    # TODO: check if zookeeper server is already running
    
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

# Main script execution
main() {
    log "Starting ZooKeeper test script..."

    if ! check_prerequisites; then
        return 1
    fi

    if ! is_zookeeper_installed; then
        if ! install_zookeeper; then
            return 1
        fi
    fi

    log "Verifying ZooKeeper installation again..."
    if ! is_zookeeper_installed; then
        echo "ZooKeeper installation verification failed."
        return 1
    fi

    PACKAGE_VERSION=$(/opt/zookeeper/bin/zkServer.sh version 2>&1 | grep -oP 'version \K[0-9.]+' | head -1) || PACKAGE_VERSION="Unknown"

    cd "$original_dir"
    if test_zookeeper_functionality; then
        log "ZooKeeper is functioning correctly."
        exit_code=0
    else
        log "ZooKeeper is not functioning correctly."
        exit_code=1
    fi
    # Clean up
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."
    log "ZooKeeper test script completed."

    return $exit_code
}

# Run the main function
main