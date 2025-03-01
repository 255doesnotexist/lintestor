#!/bin/bash

# Define the package details
PACKAGE_NAME="redis"

# Function to check if Redis is installed
is_redis_installed() {
    rpm -q $PACKAGE_NAME &> /dev/null
    return $?
}

# Function to ensure repositories are properly set up
setup_repositories() {
    echo "Ensuring required repositories are enabled..."
    # Make sure standard repositories are enabled
    dnf config-manager --set-enabled fedora updates &>/dev/null || true
    # Also enable PowerTools/crb repository if available (for dependencies)
    dnf config-manager --set-enabled crb &>/dev/null || true
    
    # Check if EPEL is installed and install if not (provides additional packages)
    if ! rpm -q epel-release &>/dev/null; then
        echo "Installing EPEL repository..."
        dnf install -y epel-release &>/dev/null || true
    fi
    
    # Update repo cache
    dnf makecache &>/dev/null
    
    echo "Repository status:"
    dnf repolist | grep -i "enabled" || true
}

# Function to install Redis package
install_redis_package() {
    # First ensure repositories are set up
    setup_repositories
    
    echo "Attempting to install $PACKAGE_NAME..."
    dnf install -y $PACKAGE_NAME || {
        echo "Failed to install Redis. Trying with --allowerasing flag..."
        dnf install -y --allowerasing $PACKAGE_NAME || return 1
    }
    
    # Check if installation was successful
    if ! is_redis_installed; then
        echo "Installation failed. Package not found."
        return 1
    fi
    
    echo "Starting Redis service..."
    systemctl start redis || {
        echo "Failed to start Redis service. Checking status:"
        systemctl status redis || true
        return 1
    }
    
    echo "Enabling Redis service..."
    systemctl enable redis
    
    # Give Redis a moment to start up
    sleep 2
    return 0
}

# Function to test Redis functionality
test_redis_functionality() {
    local test_key="test_key"
    local test_value="test_value"
    local max_attempts=3
    local attempt=1

    echo "Testing Redis connectivity..."
    
    # Try to ping Redis first to ensure it's responding
    if ! redis-cli ping &>/dev/null; then
        echo "Redis server is not responding to ping. Checking service status..."
        systemctl status redis || true
        return 1
    fi

    # Set a key-value pair with retries
    while [ $attempt -le $max_attempts ]; do
        if redis-cli SET $test_key $test_value &>/dev/null; then
            break
        fi
        echo "Attempt $attempt failed, retrying..."
        sleep 2
        ((attempt++))
    done
    
    if [ $attempt -gt $max_attempts ]; then
        echo "Failed to set test key after $max_attempts attempts."
        return 1
    fi

    # Get the value
    local retrieved_value=$(redis-cli GET $test_key)

    # Delete the key
    redis-cli DEL $test_key &>/dev/null

    # Check if the retrieved value matches the set value
    if [ "$retrieved_value" = "$test_value" ]; then
        echo "Redis is functioning correctly."
        return 0
    else
        echo "Failed to perform Redis operations."
        echo "Retrieved value: $retrieved_value"
        echo "Expected value: $test_value"
        return 1
    fi
}

# Main script execution starts here
echo "Testing Redis on Fedora..."

# Check if Redis is installed
if is_redis_installed; then
    echo "Redis is installed."
else
    echo "Redis is not installed. Attempting to install..."
    # Attempt to install the Redis package
    if install_redis_package; then
        echo "Redis installed successfully."
    else
        echo "Failed to install Redis."
        return 1
    fi
fi

# Check if Redis service is running, if not try to start it
if ! systemctl is-active --quiet redis; then
    echo "Redis service is not running. Attempting to start..."
    systemctl start redis || {
        echo "Failed to start Redis service."
        return 1
    }
fi

# Display Redis version for debugging
if command -v redis-server &>/dev/null; then
    PACKAGE_VERSION=$(redis-server --version | awk '{print $3}' | cut -d '=' -f2)
    echo "Redis version: $PACKAGE_VERSION"
else
    echo "Redis server command not found."
    return 1
fi

# Check Redis functionality by performing basic operations
if test_redis_functionality; then
    echo "Redis is functioning correctly."
    return 0
else
    echo "Redis is not functioning correctly."
    return 1
fi