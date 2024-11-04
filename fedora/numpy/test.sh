#!/bin/bash

# Define the package details
PACKAGE_NAME="python3-numpy"

# Function to check if Python 3 is installed
is_python3_installed() {
    python3 --version > /dev/null 2>&1
    return $?
}

# Function to check if NumPy is installed
is_numpy_installed() {
    python3 -c "import numpy" 2>/dev/null
    return $?
}

# Function to install Python 3 package
install_python3_package() {
    dnf install -y python3 python3-pip
    return $?
}

# Function to install NumPy
install_numpy() {
    dnf install -y $PACKAGE_NAME
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
    python3 "$python_file" > "$output_file"

    # Check if the script ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "[1 2 3]" ]]; then
        echo "Numpy is functioning correctly."
        return 0
    else
        echo "Failed to run Numpy test script."
        return 1
    fi
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

# Check if NumPy is installed
if is_numpy_installed; then
    echo "NumPy is installed."
else
    echo "NumPy is not installed. Attempting to install..."
    # Attempt to install NumPy
    if install_numpy; then
        echo "NumPy installed successfully."
    else
        echo "Failed to install NumPy."
        exit 1
    fi
fi

PACKAGE_VERSION="$(python3 --version) ($(python3 -c "import numpy; print(numpy.__version__)"))"
echo "Python and NumPy versions: $PACKAGE_VERSION"

# Check Numpy functionality by compiling and running a simple Python script
if test_numpy_functionality; then
    echo "Numpy is functioning correctly."
    exit 0
else
    echo "Numpy is not functioning correctly."
    exit 1
fi

# End of the script