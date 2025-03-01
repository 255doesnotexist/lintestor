#!/bin/bash

# Log function, output log with timestamp
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Install PostgreSQL
install_postgresql() {
    log "Installing PostgreSQL server and client..."
    sudo dnf install -y postgresql postgresql-server postgresql-contrib
    if [ $? -ne 0 ]; then
        log "Error: PostgreSQL installation failed"
        return 1
    fi
    log "PostgreSQL installation successful"
    return 0
}

# Check if PostgreSQL data directory exists and has content
check_data_directory() {
    local PGDATA="/var/lib/pgsql/data"
    if [ -d "$PGDATA" ] && [ "$(sudo ls -A $PGDATA 2>/dev/null)" ]; then
        return 0
    else
        return 1
    fi
}

# Initialize or use existing PostgreSQL database
setup_postgresql() {
    log "Setting up PostgreSQL database..."
    local PGDATA="/var/lib/pgsql/data"
    
    # Check if data directory exists and has content
    if check_data_directory; then
        log "Data directory already exists and has content"
        
        # If initdb_postgresql.log exists, check its content whether it indicates success
        if sudo test -f /var/lib/pgsql/initdb_postgresql.log; then
            if sudo grep -q "Success" /var/lib/pgsql/initdb_postgresql.log; then
                log "Previous initialization successful, using existing data directory"
                return 0
            fi
        fi
        
        # If no successful initialization log found, try to clean up and re-initialize
        log "No successful initialization log found, cleaning up and re-initializing..."
        sudo rm -rf "${PGDATA:?}"/*
    fi
    
    # Initialize database
    log "Initializing PostgreSQL database..."
    if sudo postgresql-setup --initdb; then
        log "PostgreSQL database initialization successful"
        return 0
    else
        # Even if command fails, check log whether it indicates success
        if sudo test -f /var/lib/pgsql/initdb_postgresql.log && sudo grep -q "Success" /var/lib/pgsql/initdb_postgresql.log; then
            log "Although command returned error, log indicates initialization successful"
            return 0
        else
            log "Error: PostgreSQL database initialization failed"
            if sudo test -f /var/lib/pgsql/initdb_postgresql.log; then
                log "Initialization log content:"
                sudo cat /var/lib/pgsql/initdb_postgresql.log | head -10
            fi
            return 1
        fi
    fi
}

# Configure PostgreSQL to allow local connections
configure_postgresql() {
    log "Configuring PostgreSQL connection authentication..."
    local PG_HBA_CONF="/var/lib/pgsql/data/pg_hba.conf"
    
    if ! sudo test -f "$PG_HBA_CONF"; then
        log "Error: pg_hba.conf file does not exist"
        return 1
    fi
    
    # Back up original configuration
    sudo cp -f "$PG_HBA_CONF" "${PG_HBA_CONF}.bak"
    
    # Modify authentication method, but keep postgres user using peer authentication
    log "Updating pg_hba.conf authentication configuration..."
    
    # Create new pg_hba.conf file, explicitly specifying postgres user using peer authentication
    cat << EOF | sudo tee "$PG_HBA_CONF" > /dev/null
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# "local" is for Unix domain socket connections only
local   all             postgres                                peer
local   all             all                                     md5

# IPv4 local connections:
host    all             all             127.0.0.1/32            md5

# IPv6 local connections:
host    all             all             ::1/128                 md5
EOF
    
    return 0
}

# Start and enable PostgreSQL service
start_postgresql() {
    log "Starting PostgreSQL service..."
    
    # Attempt to start or restart service
    sudo systemctl enable postgresql
    if ! sudo systemctl restart postgresql; then
        log "Failed to restart PostgreSQL service through systemd, trying to use pg_ctl directly..."
        
        # Attempt to start directly using pg_ctl
        sudo -u postgres /usr/bin/pg_ctl -D /var/lib/pgsql/data start
        if [ $? -ne 0 ]; then
            log "Error: Failed to start PostgreSQL service using pg_ctl"
            return 1
        fi
    fi
    
    # Wait for service to start
    sleep 5
    
    # Check if service is responding
    if ! pg_isready -q; then
        log "Error: PostgreSQL service is not responding"
        log "Service status details:"
        sudo systemctl status postgresql
        return 1
    fi
    
    log "PostgreSQL service has started and is responding"
    return 0
}

# Execute PostgreSQL command without password prompt
pg_exec() {
    sudo -u postgres psql --no-password -c "$1"
}

# Create test database
create_test_db() {
    local test_db="$1"
    local test_user="$2"
    
    # Use sudo -u postgres to ensure peer authentication is used, no password prompt
    sudo -u postgres createdb -O "$test_user" "$test_db" 2>/dev/null
    return $?
}

# Drop test database
drop_test_db() {
    local test_db="$1"
    
    # Use sudo -u postgres to ensure peer authentication is used, no password prompt
    sudo -u postgres dropdb "$test_db" 2>/dev/null
    return $?
}

# Test PostgreSQL functionality
test_postgresql() {
    log "Testing PostgreSQL functionality..."
    local test_db="test_db"
    local test_user="test_user"
    local test_password="test_password"
    
    # Wait for PostgreSQL to be ready to accept connections
    for i in {1..5}; do
        if pg_isready -q; then
            break
        fi
        log "Waiting for PostgreSQL to be ready to accept connections... ($i/5)"
        sleep 2
    done
    
    if ! pg_isready -q; then
        log "Error: PostgreSQL is not ready to accept connections"
        return 1
    fi
    
    # Create test user - use pg_exec function to avoid password prompt
    log "Creating test user..."
    pg_exec "DROP USER IF EXISTS $test_user;"
    pg_exec "CREATE USER $test_user WITH PASSWORD '$test_password';"
    if [ $? -ne 0 ]; then
        log "Error: Failed to create test user"
        return 1
    fi
    
    # Create test database
    log "Creating test database..."
    drop_test_db "$test_db"
    if ! create_test_db "$test_db" "$test_user"; then
        log "Error: Failed to create test database"
        # Clean up user
        pg_exec "DROP USER IF EXISTS $test_user;"
        return 1
    fi
    
    # Run test query
    log "Running test query..."
    query_output=$(PGPASSWORD=$test_password psql -h localhost -U $test_user -d $test_db -t -c "SELECT 1 AS result;" 2>&1)
    query_status=$?
    
    # Clean up test data
    log "Cleaning up test data..."
    drop_test_db "$test_db"
    pg_exec "DROP USER IF EXISTS $test_user;"
    
    # Verify query result
    if [ $query_status -eq 0 ] && [ "$(echo $query_output | tr -d ' ')" = "1" ]; then
        log "PostgreSQL test query successful: $query_output"
        return 0
    else
        log "Error: PostgreSQL test query failed"
        log "Query output: $query_output"
        log "Query return status: $query_status"
        return 1
    fi
}

# Main function
main() {
    log "Starting PostgreSQL test script..."
    
    # Get PostgreSQL installation status
    if rpm -q postgresql postgresql-server > /dev/null 2>&1; then
        log "PostgreSQL is installed"
    else
        log "PostgreSQL is not installed, installing..."
        if ! install_postgresql; then
            log "PostgreSQL installation failed"
            return 1
        fi
    fi
    
    # Get PostgreSQL version
    PACKAGE_VERSION=$(rpm -q --queryformat "%{VERSION}" postgresql 2>/dev/null)
    log "PostgreSQL version: $PACKAGE_VERSION"
    
    # Set up PostgreSQL database
    if ! setup_postgresql; then
        log "PostgreSQL setup failed"
        return 1
    fi
    
    # Configure PostgreSQL
    if ! configure_postgresql; then
        log "PostgreSQL configuration failed"
        return 1
    fi
    
    # Start PostgreSQL service
    if ! start_postgresql; then
        log "PostgreSQL start failed"
        return 1
    fi
    
    # Test PostgreSQL functionality
    if ! test_postgresql; then
        log "PostgreSQL functionality test failed"
        return 1
    fi
    
    log "PostgreSQL test successful"
    return 0
}

# Execute main function
main
