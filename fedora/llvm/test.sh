#!/bin/bash

# Define the package details
PACKAGE_NAME="llvm"

# Function to check if LLVM is installed
is_llvm_installed() {
    rpm -qa | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install LLVM package
install_llvm_package() {
    dnf install -y $PACKAGE_NAME clang
    return $?
}

# Function to compile a simple C program and test functionality
test_llvm_functionality() {
    local temp_dir=$(mktemp -d)
    local c_file="${temp_dir}/hello.c"
    local executable="${temp_dir}/hello"

    # Write a simple C program to test compilation
    cat <<EOF > "$c_file"
#include <stdio.h>

int main() {
    printf("Hello, LLVM!\n");
    return 0;
}
EOF

    # Compile the C program with LLVM
    clang "$c_file" -o "${executable}"

    # Check if the executable was created and runs without error
    if [[ -x "$executable" && "$("${executable}")" == "Hello, LLVM!" ]]; then
        echo "LLVM is functioning correctly."
        return 0
    else
        echo "Failed to compile or run LLVM test program."
        return 1
    fi
}

# Main script execution starts here

# Check if LLVM is installed
if is_llvm_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the LLVM package
    if install_llvm_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')

# Check LLVM functionality by compiling and running a simple C program
if test_llvm_functionality; then
    echo "LLVM is functioning correctly."
    return 0
else
    echo "LLVM is not functioning correctly."
    return 1
fi

# End of the script