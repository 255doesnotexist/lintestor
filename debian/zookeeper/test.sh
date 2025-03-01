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
# Track if we started ZooKeeper ourselves
ZK_STARTED_BY_SCRIPT=false

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

# Function to check if ZooKeeper is already running
is_zookeeper_running() {
    # Try to get status
    status_output=$(sudo /opt/zookeeper/bin/zkServer.sh status 2>&1) || true
    
    if echo "$status_output" | grep -q "Mode: leader" || \
       echo "$status_output" | grep -q "Mode: follower" || \
       echo "$status_output" | grep -q "Mode: standalone"; then
        log "ZooKeeper is already running."
        return 0
    else
        log "ZooKeeper is not running."
        return 1
    fi
}

# Function to install ZooKeeper
install_zookeeper() {
    log "Attempting to install ZooKeeper..."
    export DEBIAN_FRONTEND=noninteractive
    # Install Java if not already installed
    if ! command -v java >/dev/null 2>&1; then
        log "Java not found. Installing OpenJDK..."
        sudo apt-get update
        if ! sudo apt-get install -y openjdk-*-jdk; then
            echo "Failed to install Java."
            return 1
        fi
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
    else
        log "Java is installed: $(command -v java)"
    fi
    log "System prerequisites check passed."
}

# Function to test ZooKeeper functionality
test_zookeeper_functionality() {
    # Check if ZooKeeper is already running
    if is_zookeeper_running; then
        log "ZooKeeper is already running, will use the existing instance."
    else
        log "Starting ZooKeeper server..."
        if ! sudo /opt/zookeeper/bin/zkServer.sh start; then
            log "Failed to start ZooKeeper server."
            return 1
        fi
        ZK_STARTED_BY_SCRIPT=true
        
        sleep 5  # Give some time for ZooKeeper to start
        
        log "Checking ZooKeeper server status after start..."
        if ! is_zookeeper_running; then
            log "ZooKeeper server failed to start properly."
            return 1
        fi
    fi

    log "Testing ZooKeeper client connection..."
    if command -v nc >/dev/null 2>&1; then
        if ! echo "stat" | nc localhost 2181 > /dev/null; then
            log "Failed to connect to ZooKeeper server using netcat."
            return 1
        fi
    else
        log "Netcat not available, using zkCli.sh to test connection..."
        if ! echo "quit" | sudo /opt/zookeeper/bin/zkCli.sh -server localhost:2181 > /dev/null 2>&1; then
            log "Failed to connect to ZooKeeper server using zkCli.sh."
            return 1
        fi
    fi

    # Only stop ZooKeeper if we started it ourselves
    if [ "$ZK_STARTED_BY_SCRIPT" = true ]; then
        log "Stopping ZooKeeper server that we started..."
        if ! sudo /opt/zookeeper/bin/zkServer.sh stop; then
            log "Failed to stop ZooKeeper server."
            return 1
        fi
    else
        log "Leaving the existing ZooKeeper instance running."
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