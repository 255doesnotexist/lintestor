#!/bin/bash

# Define the package details
PACKAGE_NAME="gdb"
PACKAGE_TYPE="Debugger"
REPORT_FILE="report.json"

# Function to check if GDB is installed
is_gdb_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install GDB package
install_gdb_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to check GDB version and functionality
test_gdb_functionality() {
    local gdb_version=$(gdb --version | head -n1)
    if [[ -n $gdb_version ]]; then
        echo "$gdb_version"
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
    local test_name="GDB Functionality Test"
    local test_passed=false

    # Check GDB functionality
    if test_gdb_functionality; then
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
        exit 1
    fi
fi

# Check GDB functionality
if test_gdb_functionality; then
    echo "GDB is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "GDB is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
