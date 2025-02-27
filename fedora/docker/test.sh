#!/bin/bash

# Define the package details
PACKAGE_NAME="docker-ce"
PACKAGE_TYPE="Container Platform"
REPORT_FILE="report.json"

# Function to check if Docker service is active
is_docker_active() {
    sudo systemctl is-active --quiet docker
    return $?
}

# Function to check if a package is installed
is_package_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install Docker package
install_docker_package() {
    # Install required dependencies for adding the repository
    sudo dnf install -y dnf-utils 
    
    # Add Docker's official GPG key
    sudo dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo
    
    # Update package index and install Docker
    sudo dnf update -y
    sudo dnf install -y $PACKAGE_NAME
    return $?
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /etc/redhat-release)
    local kernel_version=$(uname -r)
    local package_version=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
    local test_name="Docker Service Test"
    local test_passed=false
    local distro="fedora"

    # Check if Docker service is running
    if is_docker_active; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "$distro",
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

# Check if the package is installed
if is_package_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the Docker package
    if install_docker_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check the initial state of Docker service
initial_state_active=$(is_docker_active; echo $?)

# Check if Docker service is running
if is_docker_active; then
    echo "Docker service is running."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Docker service is not running."
    # Try to start Docker service
    sudo systemctl start docker
    # Check again if Docker service is running
    if is_docker_active; then
        echo "Docker service started successfully."
        # Generate the report
        generate_report
        echo "Report generated at $REPORT_FILE"
    else
        echo "Failed to start Docker service."
        # Generate the report with test failed
        generate_report
        echo "Report generated at $REPORT_FILE with failed test."
    fi
fi

# Restore the initial state of Docker service
if [ "$initial_state_active" -eq 0 ]; then
    # If Docker was active initially, ensure it's still active
    sudo systemctl start docker
else
    # If Docker was not active initially, stop it
    sudo systemctl stop docker
fi

echo "Docker service state has been restored."

# End of the script
