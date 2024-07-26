#!/bin/bash

PACKAGE_NAME="nodejs"
PACKAGE_TYPE="JavaScript Runtime"
REPORT_FILE="report.json"

is_nodejs_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

install_nodejs_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

test_nodejs_functionality() {
    local temp_dir=$(mktemp -d)
    echo "Temp dir: $temp_dir"

    local js_file="${temp_dir}/hello.js"

    cat <<EOF > "$js_file"
console.log('Hello, Node.js!');
EOF

    local output=$(node "$js_file")
    if [[ "$output" == "Hello, Node.js!" ]]; then
        echo "Node.js is functioning correctly."
        return 0
    else
        echo "Failed to run Node.js test program correctly."
        return 1
    fi
}

generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="Node.js Functionality Test"
    local test_passed=false

    if test_nodejs_functionality; then
        test_passed=true
    fi

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

    echo "$report_content" >$REPORT_FILE
}

if is_nodejs_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    if install_nodejs_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

if test_nodejs_functionality; then
    echo "Node.js is functioning correctly."
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Node.js is not functioning correctly."
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi
