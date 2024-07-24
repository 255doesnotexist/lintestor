#!/bin/bash

# Define the package details
PACKAGE_NAME="nodejs"
PACKAGE_TYPE="JavaScript Runtime"
REPORT_FILE="report.json"

# Function to check if Node.js is installed
is_nodejs_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Node.js package
install_nodejs_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to compile a simple JavaScript program and test functionality
test_nodejs_functionality() {
    local temp_dir=$(mktemp -d)
    local js_file="${temp_dir}/hello.js"
    local executable="${temp_dir}/hello"

    # Write a simple JavaScript program to test compilation
    cat <<EOF > "$js_file"
console.log('Hello, Node.js!');
EOF

    # Compile the JavaScript program with Node.js
    node "$js_file" > "$executable"

    # Check if the executable was created and runs without error
    if [[ -x "$executable" && "$("${executable}")" == "Hello, Node.js!" ]]; then
        echo "Node.js is functioning correctly."
        return 0
    else
        echo "Failed to compile or run Node.js test program."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Node.js Functionality Test"
    local test_passed=false

    # Check Node.js functionality
    if test_nodejs_functionality; then
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

# Check if Node.js is installed
if is_nodejs_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Node.js package
    if install_nodejs_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check Node.js functionality by compiling and running a simple JavaScript program
if test_nodejs_functionality; then
    echo "Node.js is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Node.js is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
