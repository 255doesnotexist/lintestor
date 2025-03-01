#!/bin/bash

# Define the package details
PACKAGE_NAME="lighttpd"

# Function to check if Lighttpd is installed
is_lighttpd_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install Lighttpd package
install_lighttpd_package() {
    echo "Installing $PACKAGE_NAME..."
    sudo dnf install -y $PACKAGE_NAME
    return $?
}

# Function to check if Lighttpd service is running
is_lighttpd_running() {
    sudo systemctl is-active --quiet lighttpd.service
    return $?
}

# Function to start Lighttpd service
start_lighttpd_service() {
    echo "Starting lighttpd service..."
    sudo systemctl start lighttpd.service
    return $?
}

# Function to check and fix lighttpd configuration
check_lighttpd_config() {
    echo "Checking lighttpd configuration..."
    
    # Check if config file exists
    if [ ! -f "/etc/lighttpd/lighttpd.conf" ]; then
        echo "Configuration file not found. Creating default configuration..."
        sudo mkdir -p /etc/lighttpd
        sudo touch /etc/lighttpd/lighttpd.conf
    fi
    
    # Ensure server.document-root is set
    if ! grep -q "server.document-root" /etc/lighttpd/lighttpd.conf; then
        echo "Setting document root..."
        echo 'server.document-root = "/var/www/html"' | sudo tee -a /etc/lighttpd/lighttpd.conf > /dev/null
    fi
    
    # Ensure server.port is set
    if ! grep -q "server.port" /etc/lighttpd/lighttpd.conf; then
        echo "Setting server port..."
        echo 'server.port = 80' | sudo tee -a /etc/lighttpd/lighttpd.conf > /dev/null
    fi
    
    # Create document root if it doesn't exist
    if [ ! -d "/var/www/html" ]; then
        echo "Creating document root directory..."
        sudo mkdir -p /var/www/html
        sudo chmod 755 /var/www/html
    fi
    
    # Create a test index.html file
    echo "Creating test index.html..."
    echo "<html><body><h1>Lighttpd Test Page</h1></body></html>" | sudo tee /var/www/html/index.html > /dev/null
    sudo chmod 644 /var/www/html/index.html
    
    # Check configuration syntax
    echo "Validating configuration syntax..."
    if ! sudo lighttpd -t -f /etc/lighttpd/lighttpd.conf; then
        echo "Configuration syntax error."
        return 1
    fi
    
    return 0
}

# Function to test Lighttpd service status
test_lighttpd_service() {
    echo "Testing lighttpd HTTP response..."
    local max_attempts=3
    local attempt=1
    local success=false
    
    while [ $attempt -le $max_attempts ]; do
        echo "Attempt $attempt of $max_attempts..."
        local curl_output=$(curl -s -m 5 http://localhost)
        local curl_status=$?
        local curl_response=$(curl -s -o /dev/null -w "%{http_code}" http://localhost)
        
        if [[ $curl_status -eq 0 && $curl_response -eq 200 ]]; then
            echo "Success! HTTP status code: $curl_response"
            success=true
            break
        else
            echo "Failed. HTTP status code: $curl_response, curl exit code: $curl_status"
            # Wait before retrying
            sleep 2
            attempt=$((attempt + 1))
        fi
    done
    
    if $success; then
        return 0
    else
        # Diagnostic information if service is not responding
        echo "Diagnostic information:"
        echo "---- Firewall status ----"
        sudo firewall-cmd --list-all || echo "Could not get firewall status"
        
        echo "---- Lighttpd service status ----"
        sudo systemctl status lighttpd.service || echo "Could not get service status"
        
        echo "---- Log files ----"
        sudo journalctl -u lighttpd.service --no-pager -n 20 || echo "Could not get logs"
        
        return 1
    fi
}

# Main script execution starts here
main() {
    echo "Starting lighttpd test..."
    
    # Check if Lighttpd is installed
    if is_lighttpd_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        # Attempt to install the Lighttpd package
        if install_lighttpd_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            return 1
        fi
    fi
    
    PACKAGE_VERSION=$(sudo rpm -q --queryformat "%{VERSION}" $PACKAGE_NAME)
    echo "Lighttpd version: $PACKAGE_VERSION"
    
    # Check and fix configuration
    if ! check_lighttpd_config; then
        echo "Failed to configure lighttpd properly."
        return 1
    fi
    
    # Check if Lighttpd service is running
    if is_lighttpd_running; then
        echo "Lighttpd service is already running."
    else
        echo "Lighttpd service is not running. Attempting to start..."
        if start_lighttpd_service; then
            echo "Lighttpd service started successfully."
            # Give it a moment to fully start
            sleep 2
        else
            echo "Failed to start Lighttpd service."
            return 1
        fi
    fi
    
    # Check Lighttpd service status by connecting to the default port
    if test_lighttpd_service; then
        echo "Lighttpd service is active and responding."
        return 0
    else
        echo "Lighttpd service is active but not responding."
        return 1
    fi
}

# Call the main function
main