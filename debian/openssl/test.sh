#!/bin/bash

# Define the package details
PACKAGE_NAME="openssl"

# Function to check if OpenSSL is installed
is_openssl_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install OpenSSL package
install_openssl_package() {
    export DEBIAN_FRONTEND=noninteractive
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
        return 1
    fi
fi

PACKAGE_VERSION=$(openssl version | awk '{print $2}')
# Check OpenSSL functionality by performing encryption and decryption
if test_openssl_functionality; then
    echo "OpenSSL is functioning correctly."
    return 0
else
    echo "OpenSSL is not functioning correctly."
    return 1
fi

# End of the script