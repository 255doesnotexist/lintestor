#!/bin/bash

# Define the package details
PACKAGE_NAME="openssl"
PACKAGE_SHOW_NAME="OpenSSL"
PACKAGE_TYPE="Cryptography and SSL/TLS Toolkit"
REPORT_FILE="report.json"

# Function to check if OpenSSL is installed
is_openssl_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install OpenSSL package
install_openssl_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME

    return $?
}

# Function to test OpenSSL functionality
test_openssl_functionality() {
    local temp_dir=$(mktemp -d)
    local test_file="${temp_dir}/test.txt"
    local encrypted_file="${temp_dir}/test.enc"
    local decrypted_file="${temp_dir}/test.dec"

    # Create a test file
    echo "OpenSSL test message" > "$test_file"

    # Encrypt the file
    openssl enc -aes-256-cbc -salt -in "$test_file" -out "$encrypted_file" -pass pass:testpassword

    # Decrypt the file
    openssl enc -d -aes-256-cbc -in "$encrypted_file" -out "$decrypted_file" -pass pass:testpassword

    # Check if the decrypted content matches the original
    if diff "$test_file" "$decrypted_file" >/dev/null; then
        echo "OpenSSL encryption and decryption test passed."
        return 0
    else
        echo "OpenSSL encryption and decryption test failed."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local openssl_version=$(openssl version | awk '{print $2}')
    local test_name="OpenSSL Functionality Test"
    local test_passed=false

    # Check OpenSSL functionality
    if test_openssl_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$openssl_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
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

# Check if OpenSSL is installed
if is_openssl_installed; then
    echo "OpenSSL is installed."
else
    echo "OpenSSL is not installed. Attempting to install..."
    # Attempt to install the OpenSSL package
    if install_openssl_package; then
        echo "OpenSSL installed successfully."
    else
        echo "Failed to install OpenSSL."
        exit 1
    fi
fi

# Check OpenSSL functionality by performing encryption and decryption
if test_openssl_functionality; then
    echo "OpenSSL is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "OpenSSL is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script