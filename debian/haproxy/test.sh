#!/bin/bash

# Define the package details
PACKAGE_NAME="haproxy"
PACKAGE_TYPE="Load Balancer"
REPORT_FILE="report.json"

# Function to check if HAProxy is installed
is_haproxy_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install HAProxy package
install_haproxy_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to check HAProxy service status
test_haproxy_service() {
    local service_status=$(systemctl is-active --quiet haproxy)
    if [[ $service_status == "active" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
    local test_name="HAProxy Service Test"
    local test_passed=false

    # Check HAProxy service status
    if test_haproxy_service; then
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
        exit 1
    fi
fi

# Check HAProxy service status
if test_haproxy_service; then
    echo "HAProxy service is active."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "HAProxy service is not active."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
