#!/bin/bash

# Check if cmake is installed, if not, install it
if ! command -v cmake &> /dev/null; then
    echo "CMake is not installed. Installing..."
    if [ -x "$(command -v dnf)" ]; then
        dnf install -y cmake
    elif [ -x "$(command -v yum)" ]; then
        yum install -y cmake
    else
        echo "Unable to install CMake. Please install it manually."
        return 1
    fi
fi

PACKAGE_VERSION=$(cmake --version | head -n1 | cut -d' ' -f3)
CURRENT_DIR=$(pwd)

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
        return 0
    else
        return 1
    fi
else
    return 1
fi

cd "$CURRENT_DIR"

# Cleanup
# rm -rf "$TEMP_DIR"