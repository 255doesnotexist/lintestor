#!/bin/bash

# Check if cmake is installed, if not, install it
if ! command -v cmake &> /dev/null; then
    echo "CMake is not installed. Installing..."
    if [ -x "$(command -v apt-get)" ]; then
        apt-get update
        apt-get install -y cmake
    elif [ -x "$(command -v yum)" ]; then
        yum install -y cmake
    elif [ -x "$(command -v dnf)" ]; then
        dnf install -y cmake
    else
        echo "Unable to install CMake. Please install it manually."
        exit 1
    fi
fi

# Get system information
OS_VERSION=$(uname -v)
KERNEL_VERSION=$(uname -r)
PACKAGE_NAME="cmake"
PACKAGE_TYPE="Toolchain"
PACKAGE_VERSION=$(cmake --version | head -n1 | cut -d' ' -f3)
CURRENT_DIR=$(pwd)
DISTRO="debian"
# Initialize the JSON structure
JSON="{"
JSON+="\"distro\": \"$DISTRO\","
JSON+="\"os_version\": \"$OS_VERSION\","
JSON+="\"kernel_version\": \"$KERNEL_VERSION\","
JSON+="\"package_name\": \"$PACKAGE_NAME\","
JSON+="\"package_type\": \"$PACKAGE_TYPE\","
JSON+="\"package_version\": \"$PACKAGE_VERSION\","
JSON+="\"test_results\": ["

# Setup temporary test directory
TEMP_DIR="/tmp/cmake_test_dir"
mkdir -p "$TEMP_DIR"
echo "cmake_minimum_required(VERSION 3.10)" > "$TEMP_DIR/CMakeLists.txt"
echo "project(TestProject)" >> "$TEMP_DIR/CMakeLists.txt"
echo  "add_executable(test_app main.cpp)" >> "$TEMP_DIR/CMakeLists.txt"
echo 'int main() { return 0; }' > "$TEMP_DIR/main.cpp"

# Run cmake and make to compile the test project
cd "$TEMP_DIR" && cmake . && make

# Check if cmake and make succeeded
if [ -f "$TEMP_DIR/test_app" ]; then
    if "$TEMP_DIR/test_app"; then
        JSON+="{\"test_name\": \"CMake Compilation Test\", \"passed\": true}"
        ALL_TESTS_PASSED=true
    else
        JSON+="{\"test_name\": \"CMake Compilation Test\", \"passed\": false}"
        ALL_TESTS_PASSED=false
    fi
else
    JSON+="{\"test_name\": \"CMake Compilation Test\", \"passed\": false}"
    ALL_TESTS_PASSED=false
fi

cd "$CURRENT_DIR"

# Cleanup
# rm -rf "$TEMP_DIR"

JSON+="],"
JSON+="\"all_tests_passed\": $ALL_TESTS_PASSED"
JSON+="}"
# Write the JSON to the report file
echo "$JSON" > report.json