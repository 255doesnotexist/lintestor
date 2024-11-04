#!/bin/bash

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

set -euo pipefail
original_dir=$(pwd)
log "Current directory: $original_dir"

temp_dir=$(mktemp -d)
log "Created temporary directory: $temp_dir"
cd "$temp_dir"

# Define the package details
PACKAGE_NAME="python3-scipy"

# Function to check if Python 3 and SciPy are installed
is_scipy_installed() {
    if python3 -c "import scipy" >/dev/null 2>&1; then
        log "SciPy is installed."
        return 0
    else
        log "SciPy is not installed."
        return 1
    fi
}

# Function to install SciPy
install_scipy() {
    log "Attempting to install SciPy..."
    dnf install -y python3-scipy
    if [[ $? -ne 0 ]]; then
        echo "Failed to install SciPy."
        return 1
    fi
    log "SciPy installed successfully."
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check for Python 3
    if ! command -v python3 >/dev/null 2>&1; then
        echo "Python 3 is not installed. Please install Python 3 and try again."
        return 1
    fi

    log "System prerequisites check passed."
}

# Function to test SciPy functionality
test_scipy_functionality() {
    local temp_dir
    temp_dir=$(mktemp -d) || (echo "Failed to create temporary directory." && return 1)
    log "Created temporary directory: $temp_dir"

    cd "$temp_dir"

    # Create a simple SciPy test script
    cat <<EOF > test_scipy.py
import scipy
import numpy as np
from scipy import stats

# Generate some random data
data = np.random.randn(1000)

# Compute the mean and standard deviation
mean = np.mean(data)
std = np.std(data)

# Perform a one-sample t-test
t_statistic, p_value = stats.ttest_1samp(data, 0)

print(f"Mean: {mean:.4f}")
print(f"Standard Deviation: {std:.4f}")
print(f"T-statistic: {t_statistic:.4f}")
print(f"P-value: {p_value:.4f}")
EOF

    # Run the SciPy test script
    log "Running the SciPy test script..."
    if ! output=$(python3 test_scipy.py 2>&1); then
        log "Failed to run SciPy test script. Output:"
        log "$output"
        rm -rf "$temp_dir"
        return 1
    fi

    log "Script output:"
    log "$output"

    # Clean up
    cd ..
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    # After cleaning up:
    cd "$original_dir"
    log "Changed back to original directory: $original_dir"

    # Check if the output contains expected elements
    if [[ "$output" == *"Mean:"* && "$output" == *"Standard Deviation:"* && "$output" == *"T-statistic:"* && "$output" == *"P-value:"* ]]; then
        log "SciPy test passed successfully."
        return 0
    else
        log "Unexpected output from SciPy test script."
        return 1
    fi
}

# Main script execution
main() {
    log "Starting SciPy test script..."

    if ! check_prerequisites; then
        return 1
    fi

    if ! is_scipy_installed; then
        if ! install_scipy; then
            return 1
        fi
    fi

    local python_version=$(python3 --version | awk '{print $2}') || python_version="Unknown"
    local scipy_version=$(python3 -c "import scipy; print(scipy.__version__)") || scipy_version="Unknown"
    PACKAGE_VERSION="$scipy_version (python $python_version)"
    if test_scipy_functionality; then
        log "SciPy is functioning correctly."
        return 0
    else
        log "SciPy is not functioning correctly."
        return 1
    fi
}

# Run the main function
main