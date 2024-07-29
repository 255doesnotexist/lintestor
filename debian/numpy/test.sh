#!/bin/bash

# Define the package details
PACKAGE_NAME="python3-numpy"
PACKAGE_SHOW_NAME="numpy"
PACKAGE_TYPE="Python Scientific Library"
REPORT_FILE="report.json"

# Function to check if Python 3 is installed
is_python3_installed() {
    dpkg -l | grep -qw python3-full
    return $?
}

# Function to install Python 3 package
install_python3_package() {
    apt-get update
    apt-get install -y python3-full
    apt-get install -y python-is-python3 python3-pip
    apt-get install -y $PACKAGE_NAME

    return $?
}

# Function to test Numpy functionality
test_numpy_functionality() {
    local temp_dir=$(mktemp -d)
    local python_file="${temp_dir}/test_numpy.py"
    local output_file="${temp_dir}/numpy_output.txt"

    # Write a simple Python script to test Numpy functionality
    cat <<EOF > "$python_file"
import numpy as np

print(np.array([1, 2, 3]))
EOF

    # Run the Python script with Numpy
    python "$python_file" > "$output_file"

    # Check if the script ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "[1 2 3]" ]]; then
        echo "Numpy is functioning correctly."
        return 0
    else
        echo "Failed to run Numpy test script."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local python_version=$(python3 --version)
    local numpy_version=$(python3 -c "import numpy; print(numpy.__version__)")
    local test_name="Numpy Functionality Test"
    local test_passed=false

    # Check Numpy functionality
    if test_numpy_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$numpy_version ($python_version)",
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

# Check if Python 3 is installed
if is_python3_installed; then
    echo "Python 3 is installed."
else
    echo "Python 3 is not installed. Attempting to install..."
    # Attempt to install the Python 3 package
    if install_python3_package; then
        echo "Python 3 installed successfully."
    else
        echo "Failed to install Python 3."
        exit 1
    fi
fi

# Check Numpy functionality by compiling and running a simple Python script
if test_numpy_functionality; then
    echo "Numpy is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Numpy is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
