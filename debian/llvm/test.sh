#!/bin/bash

# Define the package details
PACKAGE_NAME="llvm"
PACKAGE_SHOW_NAME="llvm"
PACKAGE_TYPE="Compiler Toolchain"
REPORT_FILE="report.json"

# Function to check if LLVM is installed
is_llvm_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install LLVM package
install_llvm_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME clang
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

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="LLVM Functionality Test"
    local test_passed=false

    # Check LLVM functionality
    if test_llvm_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_SHOW_NAME",
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
        exit 1
    fi
fi

# Check LLVM functionality by compiling and running a simple C program
if test_llvm_functionality; then
    echo "LLVM is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "LLVM is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
