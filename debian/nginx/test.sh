#!/bin/bash

# Define the package details
PACKAGE_NAME="nginx"

# Function to check if Nginx is installed
is_nginx_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Nginx package
install_nginx_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to test Nginx service status
test_nginx_service() {
    local curl_response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost)
    if [[ $curl_response -eq 200 ]]; then
        return 0
    else
        return 1
    fi
}

# Main script execution starts here

# Check if Nginx is installed
if is_nginx_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Nginx package
    if install_nginx_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')

# Check Nginx service status by connecting to the default port
if test_nginx_service; then
    echo "Nginx service is active and responding."
    return 0
else
    echo "Nginx service is active but not responding."
    return 1
fi

# End of the script
