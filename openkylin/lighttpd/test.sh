#!/bin/bash

# Define the package details
PACKAGE_NAME="lighttpd"

# Function to check if Lighttpd is installed
is_lighttpd_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Lighttpd package
install_lighttpd_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to test Lighttpd service status
test_lighttpd_service() {
    local curl_response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost)
    if [[ $curl_response -eq 200 ]]; then
        return 0
    else
        return 1
    fi
}

# Main script execution starts here

# Check if Lighttpd is installed
if is_lighttpd_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Lighttpd package
    if install_lighttpd_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')

# Check Lighttpd service status by connecting to the default port
if test_lighttpd_service; then
    echo "Lighttpd service is active and responding."
    return 0
else
    echo "Lighttpd service is active but not responding."
    return 1
fi

# End of the script
