#!/bin/bash

# Get system information
OS_VERSION=$(uname -v)
KERNEL_VERSION=$(uname -r)
PACKAGE_NAME="gcc"
PACKAGE_TYPE="Toolchain"
PACKAGE_VERSION=$(gcc --version | head -n1 | cut -d' ' -f4)
distro="debian"

# Initialize the JSON structure
JSON="{"
JSON+="\"distro\": \"$distro\","
JSON+="\"os_version\": \"$OS_VERSION\","
JSON+="\"kernel_version\": \"$KERNEL_VERSION\","
JSON+="\"package_name\": \"$PACKAGE_NAME\","
JSON+="\"package_type\": \"$PACKAGE_TYPE\","
JSON+="\"package_version\": \"$PACKAGE_VERSION\","
JSON+="\"test_results\": ["

# Check if the test executable exists and is executable
if [ -x "$1" ]; then
    TEST_RESULT=$($1)
    if [ $? -eq 0 ]; then
        JSON+="{\"test_name\": \"GCC Compilation Test\", \"passed\": true}"
        ALL_TESTS_PASSED=true
    else
        JSON+="{\"test_name\": \"GCC Compilation Test\", \"passed\": false}"
        ALL_TESTS_PASSED=false
    fi
else
    JSON+="{\"test_name\": \"GCC Compilation Test\", \"passed\": false}"
    ALL_TESTS_PASSED=false
fi

JSON+="],"
JSON+="\"all_tests_passed\": $ALL_TESTS_PASSED"
JSON+="}"

# Write the JSON to the report file
echo "$JSON" > report.json