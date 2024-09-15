#!/bin/bash

# Define the package details
PACKAGE_NAME="golang-go"

# Function to check if Go is installed
is_go_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Go package
install_go_package() {
    export DEBIAN_FRONTEND=noninteractive
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
        return 1
    fi
fi

PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
# Check Go functionality by compiling a simple Go program
if test_go_functionality; then
    echo "Go is functioning correctly."
    return 0
else
    echo "Go is not functioning correctly."
    return 1
fi

# End of the script
