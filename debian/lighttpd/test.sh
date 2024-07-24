#!/bin/bash

# Define the package details
PACKAGE_NAME="lighttpd"
PACKAGE_TYPE="Web Server"
REPORT_FILE="report.json"

# Function to check if Lighttpd is installed
is_lighttpd_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Lighttpd package
install_lighttpd_package() {
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

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Lighttpd Service Test"
    local test_passed=false

    # Check Lighttpd service status
    if test_lighttpd_service; then
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
        exit 1
    fi
fi

# Check Lighttpd service status by connecting to the default port
if test_lighttpd_service; then
    echo "Lighttpd service is active and responding."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Lighttpd service is active but not responding."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
