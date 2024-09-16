#!/bin/bash

# Define the package details
PACKAGE_NAME="redis-server"

# Function to check if Redis is installed
is_redis_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Redis package
install_redis_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    systemctl start redis-server
    systemctl enable redis-server
    return $?
}

# Function to test Redis functionality
test_redis_functionality() {
    local test_key="test_key"
    local test_value="test_value"

    # Set a key-value pair
    redis-cli SET $test_key $test_value > /dev/null

    # Get the value
    local retrieved_value=$(redis-cli GET $test_key)

    # Delete the key
    redis-cli DEL $test_key > /dev/null

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

PACKAGE_VERSION=$(redis-server --version | awk '{print $3}' | cut -d '=' -f2)

# Check Redis functionality by performing basic operations
if test_redis_functionality; then
    echo "Redis is functioning correctly."
    return 0
else
    echo "Redis is not functioning correctly."
    return 1
fi

# End of the script