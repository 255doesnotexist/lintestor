#!/bin/bash

# Define the package details
PACKAGE_NAME="postgresql"

# Function to check if PostgreSQL is installed
is_postgresql_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install PostgreSQL package
install_postgresql_package() {
    sudo dnf install -y $PACKAGE_NAME postgresql-contrib
    sudo systemctl enable postgresql
    sudo systemctl start postgresql
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
        return 1
    fi
fi

PACKAGE_VERSION=$(sudo psql --version | awk '{print $3}')

# Check PostgreSQL functionality by running a simple query
if test_postgresql_functionality; then
    echo "PostgreSQL is functioning correctly."
    return 0
else
    echo "PostgreSQL is not functioning correctly."
    return 1
fi

# End of the script
