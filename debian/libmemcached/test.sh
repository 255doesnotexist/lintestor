#!/bin/bash

# Define the package details
PACKAGE_NAME="libmemcached11"
PACKAGE_TYPE="Caching Library"
REPORT_FILE="report.json"

# Function to check if libmemcached is installed
is_libmemcached_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install libmemcached package
install_libmemcached_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to test libmemcached functionality
test_libmemcached_functionality() {
    local temp_dir=$(mktemp -d)
    local test_file="${temp_dir}/test_libmemcached"
    local memcached_server="localhost:11211"

    # Write a simple C program to test libmemcached functionality
    cat <<EOF > "$test_file"
#include <stdio.h>
#include <memcached/memcached.h>

int main() {
    memcached_st *memc;
    memcached_server_st *servers = memcached_servers_new(memcached_server, strlen(memcached_server), 0, 0);
    memcached_server_list_free(servers);
    return 0;
}
EOF

    # Compile the C program with libmemcached support
    gcc "$test_file" -o "${test_file%.*}" -lmemcached -o "${test_file%.*}"

    # Check if the executable was created and runs without error
    if [[ -x "${test_file%.*}" && "$("${test_file%.*}")" == "libmemcached test completed successfully." ]]; then
        echo "libmemcached is functioning correctly."
        return 0
    else
        echo "Failed to compile or run libmemcached test program."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
    local test_name="libmemcached Functionality Test"
    local test_passed=false

    # Check libmemcached functionality
    if test_libmemcached_functionality; then
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

# Check if libmemcached is installed
if is_libmemcached_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the libmemcached package
    if install_libmemcached_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check libmemcached functionality by compiling and running a simple C program
if test_libmemcached_functionality; then
    echo "libmem