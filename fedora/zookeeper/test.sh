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
    
    # Install netcat for connectivity tests
    if ! command -v nc >/dev/null 2>&1; then
        log "Netcat not found. Installing..."
        sudo dnf install -y nc
    fi

    # Download and install ZooKeeper
    local zk_version="3.9.3"
    local zk_url="https://dlcdn.apache.org/zookeeper/zookeeper-${zk_version}/apache-zookeeper-${zk_version}-bin.tar.gz"
    
    log "Downloading ZooKeeper ${zk_version}..."
    if ! curl -L -o zookeeper.tar.gz "$zk_url"; then
        log "Failed to download ZooKeeper. Trying alternate mirror..."
        local alt_url="https://archive.apache.org/dist/zookeeper/zookeeper-${zk_version}/apache-zookeeper-${zk_version}-bin.tar.gz"
        if ! curl -L -o zookeeper.tar.gz "$alt_url"; then
            log "Failed to download ZooKeeper from alternate mirror."
            return 1
        fi
    fi

    log "Extracting ZooKeeper..."
    if ! sudo mkdir -p /opt; then
        log "Failed to create /opt directory."
        return 1
    fi
    
    if ! sudo tar -xzf zookeeper.tar.gz -C /opt; then
        log "Failed to extract ZooKeeper."
        return 1
    fi

    if [ -d "/opt/zookeeper" ]; then
        log "Removing existing ZooKeeper directory..."
        sudo rm -rf /opt/zookeeper
    fi

    if ! sudo mv /opt/apache-zookeeper-${zk_version}-bin /opt/zookeeper; then
        log "Failed to rename ZooKeeper directory."
        return 1
    fi

    # Create data directory with proper permissions
    log "Creating data directory..."
    sudo mkdir -p /opt/zookeeper/data
    sudo chmod -R 755 /opt/zookeeper/data

    # Configure ZooKeeper with a custom configuration
    log "Creating ZooKeeper configuration..."
    cat > zoo.cfg <<EOF
# Basic ZooKeeper configuration
tickTime=2000
initLimit=10
syncLimit=5
dataDir=/opt/zookeeper/data
clientPort=2181
maxClientCnxns=60
admin.enableServer=false
4lw.commands.whitelist=*
EOF
    sudo cp zoo.cfg /opt/zookeeper/conf/zoo.cfg
    sudo chmod 644 /opt/zookeeper/conf/zoo.cfg

    log "ZooKeeper installation completed. Verifying installation..."
    if ! is_zookeeper_installed; then
        log "ZooKeeper installation failed. ZooKeeper is not available at the expected location."
        return 1
    }
    log "ZooKeeper installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."
    
    # Check for Java
    if command -v java >/dev/null 2>&1; then
        log "Java is installed: $(java -version 2>&1 | head -1)"
    else
        log "Java is not installed. It will be installed during ZooKeeper installation."
    fi
    
    # Check for curl
    if ! command -v curl >/dev/null 2>&1; then
        log "curl is not installed. Installing..."
        sudo dnf install -y curl
    fi
    
    # Check port availability
    if command -v nc >/dev/null 2>&1; then
        if nc -z localhost 2181 2>/dev/null; then
            log "WARNING: Port 2181 is already in use. ZooKeeper might not start properly."
        fi
    fi
    
    log "System prerequisites check passed."
}

# Function to test ZooKeeper functionality
test_zookeeper_functionality() {
    # Ensure any previous instance is stopped
    log "Stopping any existing ZooKeeper instance..."
    sudo /opt/zookeeper/bin/zkServer.sh stop || true
    
    # Make sure port is free
    sleep 2
    if command -v nc >/dev/null 2>&1 && nc -z localhost 2181 2>/dev/null; then
        log "Port 2181 is still in use. Attempting to free it..."
        sudo fuser -k 2181/tcp || true
        sleep 2
    fi
    
    # Set JVMFLAGS to use IPv4 and set heap size
    export JVMFLAGS="-Djava.net.preferIPv4Stack=true -Xms256m -Xmx512m -XX:MaxDirectMemorySize=256m"
    
    log "Starting ZooKeeper server..."
    log "Java path: $(which java)"
    
    # Create myid file if it doesn't exist
    if [ ! -f "/opt/zookeeper/data/myid" ]; then
        log "Creating myid file..."
        echo "1" | sudo tee /opt/zookeeper/data/myid > /dev/null
    fi
    
    # Start ZooKeeper in the background
    if ! sudo ZOOCFGDIR=/opt/zookeeper/conf /opt/zookeeper/bin/zkServer.sh start; then
        log "Failed to start ZooKeeper server."
        log "Checking logs for errors:"
        sudo cat /opt/zookeeper/logs/zookeeper*log 2>/dev/null || log "No log files found"
        return 1
    fi
    
    # Wait longer for ZooKeeper to start, especially on slower systems
    log "Waiting 15 seconds for ZooKeeper to start completely..."
    sleep 15
    
    log "Checking ZooKeeper server status..."
    if ! sudo /opt/zookeeper/bin/zkServer.sh status; then
        log "ZooKeeper server is not running properly according to zkServer.sh status."
        log "Trying alternative verification methods..."
        
        # Check if process is running
        if pgrep -f zookeeper >/dev/null; then
            log "ZooKeeper process is running."
            
            # Try direct connection test
            if echo ruok | nc localhost 2181 | grep -q "imok"; then
                log "ZooKeeper is responding to 'ruok' command. Service is running."
                return 0
            else
                log "ZooKeeper didn't respond to 'ruok' command."
            fi
        else
            log "No ZooKeeper process found running."
        fi
        
        log "Checking ZooKeeper logs for errors:"
        sudo find /opt/zookeeper/logs -name "*.log" -exec cat {} \; 2>/dev/null || log "No log files found"
        
        return 1
    fi
    
    log "Testing ZooKeeper client connection..."
    if echo ruok | nc localhost 2181 | grep -q "imok"; then
        log "Successfully connected to ZooKeeper and received 'imok' response."
    else
        log "Failed to connect to ZooKeeper server or didn't receive 'imok' response."
        return 1
    fi
    
    log "Stopping ZooKeeper server..."
    if ! sudo /opt/zookeeper/bin/zkServer.sh stop; then
        log "Failed to stop ZooKeeper server. Attempting force kill..."
        sudo pkill -f zookeeper || true
    fi
    
    sleep 2
    
    # Verify it's stopped
    if pgrep -f zookeeper >/dev/null; then
        log "WARNING: ZooKeeper process is still running after stop command."
    else
        log "ZooKeeper process successfully stopped."
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
        log "ZooKeeper installation verification failed."
        return 1
    fi
    
    # Try to get ZooKeeper version
    log "Checking ZooKeeper version..."
    PACKAGE_VERSION=$(grep -a "version=" /opt/zookeeper/lib/zookeeper-*.jar 2>/dev/null | head -1 | cut -d'=' -f2) || PACKAGE_VERSION="Unknown"
    log "ZooKeeper version: $PACKAGE_VERSION"
    
    if test_zookeeper_functionality; then
        log "ZooKeeper is functioning correctly."
        exit_code=0
    else
        log "ZooKeeper is not functioning correctly."
        exit_code=1
    fi
    
    # Clean up
    cd "$original_dir"
    sudo rm -rf "$temp_dir"
    log "Cleaned up temporary directory."
    log "ZooKeeper test script completed."
    
    return $exit_code
}

# Run the main function
main
return $?