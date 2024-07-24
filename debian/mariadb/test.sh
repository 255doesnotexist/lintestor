#!/bin/bash

# Define the package details
PACKAGE_NAME="mariadb-server"
PACKAGE_TYPE="Database Server"
REPORT_FILE="report.json"

# Function to check if MariaDB is installed
is_mariadb_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install MariaDB package
install_mariadb_package() {
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

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="MariaDB Service Test"
    local test_passed=false

    # Check MariaDB service status
    if test_mariadb_service; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_NAME",
    "package_type": "$PACKAGE_TYPE",
    "package_version": "$package_version",
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
        exit 1
    fi
fi

# Check MariaDB service status
if test_mariadb_service; then
    echo "MariaDB service is active and responding."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "MariaDB service is active but not responding."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
