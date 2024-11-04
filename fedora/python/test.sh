#!/bin/bash

# Define the package details
PACKAGE_NAME="python3"

# Function to check if Python is installed
is_python_installed() {
    command -v python3 >/dev/null 2>&1
    return $?
}

# Function to install Python package
install_python_package() {
    # Use dnf instead of apt-get for Fedora
    dnf install -y python3 python3-pip
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
        return 1
    fi
fi

PACKAGE_VERSION=$(python3 --version | awk '{print $2}')

# Check Python functionality by running a test script
if test_python_functionality; then
    echo "Python is functioning correctly."
    return 0
else
    echo "Python is not functioning correctly."
    return 1
fi

# End of the script