#!/bin/bash

# Define the package details
PACKAGE_NAME="golang-go"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

# Function to check if Go is installed
is_go_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Go package
install_go_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to compile a simple Go program and test functionality
test_go_functionality() {
    local temp_dir=$(mktemp -d)
    local go_file="${temp_dir}/hello.go"
    local executable="${temp_dir}/hello"

    # Write a simple Go program to test compilation
    cat <<EOF > "$go_file"
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
EOF

    # Compile the Go program
    go build -o "$executable" "$go_file"

    # Check if the executable was created and runs without error
    if [[ -x "$executable" && "$($executable)" == "Hello, World!" ]]; then
        echo "Go program compiled and ran successfully."
        return 0
    else
        echo "Failed to compile or run Go program."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep$PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Go Compilation Test"
    local test_passed=false

    # Check Go functionality
    if test_go_functionality; then
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

# Check if Go is installed
if is_go_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Go package
    if install_go_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check Go functionality by compiling a simple Go program
if test_go_functionality; then
    echo "Go is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Go is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
