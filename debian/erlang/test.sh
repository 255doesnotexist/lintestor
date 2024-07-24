#!/bin/bash

# Define the package details
PACKAGE_NAME="erlang"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

# Function to check if Erlang service is active
is_erlang_active() {
    # Erlang does not run as a service like Apache, so we check if the Erlang shell is running
    pgrep -x beam.smp &>/dev/null
    return $?
}

# Function to check if a package is installed
is_package_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Erlang package
install_erlang_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Erlang Service Test"
    local test_passed=false

    # Check if Erlang shell is running
    if is_erlang_active; then
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

# Check if the package is installed
if is_package_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Erlang package
    if install_erlang_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check if Erlang shell is running
if is_erlang_active; then
    echo "Erlang shell is running."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Erlang shell is not running."
    # Erlang does not have a service to start like Apache, so we skip this step
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
