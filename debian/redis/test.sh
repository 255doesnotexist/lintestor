#!/bin/bash

# Define the package details
PACKAGE_NAME="redis-server"
PACKAGE_SHOW_NAME="Redis"
PACKAGE_TYPE="In-Memory Data Structure Store"
REPORT_FILE="report.json"

# Function to check if Redis is installed
is_redis_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Redis package
install_redis_package() {
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

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local redis_version=$(redis-server --version | awk '{print $3}' | cut -d '=' -f2)
    local test_name="Redis Functionality Test"
    local test_passed=false

    # Check Redis functionality
    if test_redis_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$redis_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "$test_name",
            "passed": $test_passed
        }
    ],
    "all_tests_passed": $test_passed
}
EOF
)

    # Write the report to the file
    echo "$report_content" >$REPORT_FILE
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
        exit 1
    fi
fi

# Check Redis functionality by performing basic operations
if test_redis_functionality; then
    echo "Redis is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Redis is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script