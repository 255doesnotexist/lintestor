#!/bin/bash

# Define the package details
PACKAGE_NAME="mariadb-server"

# Function to check if MariaDB is installed
is_mariadb_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install MariaDB package
install_mariadb_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to test MariaDB service status
test_mariadb_service() {
    local mysql_status=$(mysqladmin ping --host=localhost --user=root --password=root)
    if [[ $mysql_status == "mysqld is alive" ]]; then
        return 0
    else
        return 1
    fi
}

# Main script execution starts here

# Check if MariaDB is installed
if is_mariadb_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the MariaDB package
    if install_mariadb_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')

# Check MariaDB service status
if test_mariadb_service; then
    echo "MariaDB service is active and responding."
    return 0
else
    echo "MariaDB service is active but not responding."
    return 1
fi

# End of the script
