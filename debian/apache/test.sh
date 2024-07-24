#!/bin/bash

# Define the package details
PACKAGE_NAME="apache2"
PACKAGE_TYPE="Web Server"
REPORT_FILE="report.json"

# Function to check if Apache service is active
is_apache_active() {
    systemctl is-active --quiet apache2
    return $?
}

# Function to check if Apache service is enabled
is_apache_enabled() {
    systemctl is-enabled --quiet apache2
    return $?
}

# Function to check if a package is installed
is_package_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Apache package
install_apache_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep$PACKAGE_NAME | awk '{print $3}')
    local test_name="Apache Service Test"
    local test_passed=false

    # Check if Apache service is running
    if is_apache_active; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
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
    # Attempt to install the Apache package
    if install_apache_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check the initial state of Apache service
initial_state_active=$(is_apache_active; echo$?)
initial_state_enabled=$(is_apache_enabled; echo$?)

# Check if Apache service is running
if is_apache_active; then
    echo "Apache service is running."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Apache service is not running."
    # Try to start Apache service
    systemctl start apache2
    # Check again if Apache service is running
    if is_apache_active; then
        echo "Apache service started successfully."
        # Generate the report
        generate_report
        echo "Report generated at $REPORT_FILE"
    else
        echo "Failed to start Apache service."
        # Generate the report with test failed
        generate_report
        echo "Report generated at $REPORT_FILE with failed test."
    fi
fi

# Restore the initial state of Apache service
if [ "$initial_state_active" -eq 0 ]; then
    # If Apache was active initially, ensure it's still active
    systemctl start apache2
else
    # If Apache was not active initially, stop it
    systemctl stop apache2
fi

# If Apache was enabled initially, ensure it's still enabled
if [ "$initial_state_enabled" -eq 0 ]; then
    systemctl enable apache2
else
    systemctl disable apache2
fi

echo "Apache service state has been restored."

# End of the script
