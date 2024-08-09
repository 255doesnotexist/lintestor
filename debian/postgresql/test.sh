#!/bin/bash

# Define the package details
PACKAGE_NAME="postgresql"
PACKAGE_SHOW_NAME="PostgreSQL"
PACKAGE_TYPE="Relational Database Management System"
REPORT_FILE="report.json"

# Function to check if PostgreSQL is installed
is_postgresql_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install PostgreSQL package
install_postgresql_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME postgresql-contrib
    systemctl start postgresql
    systemctl enable postgresql
    return $?
}

# Function to test PostgreSQL functionality
test_postgresql_functionality() {
    local test_db="test_db"
    local test_user="test_user"
    local test_password="test_password"

    # Create a test user and database
    sudo -u postgres psql -c "CREATE USER $test_user WITH PASSWORD '$test_password';"
    sudo -u postgres createdb -O $test_user $test_db

    # Run a test query
    local query_result=$(PGPASSWORD=$test_password psql -h localhost -U $test_user -d $test_db -t -c "SELECT 1 AS result;")

    # Clean up
    sudo -u postgres dropdb $test_db
    sudo -u postgres psql -c "DROP USER $test_user;"

    # Check if the query was successful
    if [ "$(echo $query_result | tr -d ' ')" = "1" ]; then
        echo "PostgreSQL is functioning correctly."
        return 0
    else
        echo "Failed to run PostgreSQL test query."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local postgresql_version=$(psql --version | awk '{print $3}')
    local test_name="PostgreSQL Functionality Test"
    local test_passed=false

    # Check PostgreSQL functionality
    if test_postgresql_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$postgresql_version",
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

# Check if PostgreSQL is installed
if is_postgresql_installed; then
    echo "PostgreSQL is installed."
else
    echo "PostgreSQL is not installed. Attempting to install..."
    # Attempt to install the PostgreSQL package
    if install_postgresql_package; then
        echo "PostgreSQL installed successfully."
    else
        echo "Failed to install PostgreSQL."
        exit 1
    fi
fi

# Check PostgreSQL functionality by running a simple query
if test_postgresql_functionality; then
    echo "PostgreSQL is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "PostgreSQL is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script