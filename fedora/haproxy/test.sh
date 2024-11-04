#!/bin/bash

# Define the package details
PACKAGE_NAME="haproxy"

# Function to check if HAProxy is installed
is_haproxy_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install HAProxy package
install_haproxy_package() {
    dnf install -y $PACKAGE_NAME
    return $?
}

# Function to check HAProxy service status
test_haproxy_service() {
    systemctl is-active --quiet haproxy
    return $?
}

# Function to start HAProxy service
start_haproxy_service() {
    systemctl start haproxy
    return $?
}

# Main script execution starts here

# Check if HAProxy is installed
if is_haproxy_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the HAProxy package
    if install_haproxy_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

PACKAGE_VERSION=$(rpm -q --queryformat '%{VERSION}' $PACKAGE_NAME)
# Check HAProxy service status
if test_haproxy_service; then
    echo "HAProxy service is active."
    return 0
else
    echo "HAProxy service is not active. Attempting to start the service..."
    # Attempt to start the HAProxy service
    if start_haproxy_service; then
        echo "HAProxy service started successfully."
        # Recheck HAProxy service status
        if test_haproxy_service; then
            echo "HAProxy service is now active."
            return 0
        else
            echo "Failed to start HAProxy service."
            return 1
        fi
    else
        echo "Failed to start HAProxy service."
        return 1
    fi
fi

# End of the script