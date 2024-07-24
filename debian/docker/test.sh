#!/bin/bash

# Define the package details
PACKAGE_NAME="docker-ce"
PACKAGE_TYPE="Container Platform"
REPORT_FILE="report.json"

# Function to check if Docker service is active
is_docker_active() {
    systemctl is-active --quiet docker
    return $?
}

# Function to check if a package is installed
is_package_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Docker package
install_docker_package() {
    apt install gpg curl lsb-release apt-transport-https ca-certificates software-properties-common -y

    # Add Docker's official GPG key
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

    # Add Docker repository
    echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/debian \
  $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

    # Update package index and install Docker
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
    local test_name="Docker Service Test"
    local test_passed=false
    local distro="debian"

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
initial_state_active=$(is_docker_active; echo$?)

# Check if Docker service is running
if is_docker_active; then
    echo "Docker service is running."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Docker service is not running."
    # Try to start Docker service
    systemctl start docker
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
    systemctl start docker
else
    # If Docker was not active initially, stop it
    systemctl stop docker
fi

echo "Docker service state has been restored."

# End of the script
