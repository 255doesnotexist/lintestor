#!/bin/bash

# Define the package details
PACKAGE_NAME="gdb"

# Function to check if GDB is installed
is_gdb_installed() {
    rpm -qa | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install GDB package
install_gdb_package() {
    dnf install -y $PACKAGE_NAME
    return $?
}

# Function to check GDB version and functionality
test_gdb_functionality() {
    PACKAGE_VERSION=$(gdb --version | head -n1)
    if [[ -n $PACKAGE_VERSION ]]; then
        echo "$PACKAGE_VERSION"
        return 0
    else
        return 1
    fi
}

# Main script execution starts here

# Check if GDB is installed
if is_gdb_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the GDB package
    if install_gdb_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

# Check GDB functionality
if test_gdb_functionality; then
    echo "GDB is functioning correctly."
    return 0
else
    echo "GDB is not functioning correctly."
    return 1
fi

# End of the script