#!/bin/bash

# Define the package details
PACKAGE_NAME="python3"
PACKAGE_SHOW_NAME="Python"
PACKAGE_TYPE="High-level Programming Language"
REPORT_FILE="report.json"

# Function to check if Python is installed
is_python_installed() {
    command -v python3 >/dev/null 2>&1
    return $?
}

# Function to install Python package
install_python_package() {
    apt-get update
    apt-get install -y python3 python3-pip
    return $?
}

# Function to test Python functionality
test_python_functionality() {
    local temp_dir=$(mktemp -d)
    local python_file="${temp_dir}/test.py"
    local output_file="${temp_dir}/python_output.txt"

    # Write a simple Python script to test functionality
    cat <<EOF > "$python_file"
import sys
import math

def test_basic_operations():
    assert 2 + 2 == 4, "Basic addition failed"
    assert 10 - 5 == 5, "Basic subtraction failed"
    assert 3 * 4 == 12, "Basic multiplication failed"
    assert 15 / 3 == 5, "Basic division failed"

def test_string_operations():
    assert "hello " + "world" == "hello world", "String concatenation failed"
    assert "python".upper() == "PYTHON", "String upper() method failed"

def test_list_operations():
    lst = [1, 2, 3]
    lst.append(4)
    assert lst == [1, 2, 3, 4], "List append failed"

def test_math_operations():
    assert math.sqrt(16) == 4, "Math square root failed"
    assert math.pi > 3.14 and math.pi < 3.15, "Math pi constant failed"

if __name__ == "__main__":
    test_basic_operations()
    test_string_operations()
    test_list_operations()
    test_math_operations()
    print("All tests passed successfully!")
EOF

    # Run the Python script
    python3 "$python_file" > "$output_file" 2>&1

    # Check if the script ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "All tests passed successfully!" ]]; then
        echo "Python is functioning correctly."
        return 0
    else
        echo "Failed to run Python test script."
        cat "$output_file"  # Print the actual output for debugging
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local python_version=$(python3 --version | awk '{print $2}')
    local test_name="Python Functionality Test"
    local test_passed=false

    # Check Python functionality
    if test_python_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$python_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
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

# Check if Python is installed
if is_python_installed; then
    echo "Python is installed."
else
    echo "Python is not installed. Attempting to install..."
    # Attempt to install the Python package
    if install_python_package; then
        echo "Python installed successfully."
    else
        echo "Failed to install Python."
        exit 1
    fi
fi

# Check Python functionality by running a test script
if test_python_functionality; then
    echo "Python is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Python is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script