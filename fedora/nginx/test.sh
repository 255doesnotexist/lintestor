#!/bin/bash

# Define the package details
PACKAGE_NAME="nginx"

# Function to check if Nginx is installed
is_nginx_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install Nginx package
install_nginx_package() {
    echo "Installing $PACKAGE_NAME..."
    sudo dnf install -y $PACKAGE_NAME
    return $?
}

# Function to check if Nginx service is running
is_nginx_running() {
    sudo systemctl is-active --quiet nginx.service
    return $?
}

# Function to start Nginx service
start_nginx_service() {
    echo "Starting nginx service..."
    sudo systemctl start nginx.service
    return $?
}

# Function to check and fix nginx configuration
check_nginx_config() {
    echo "Checking nginx configuration..."
    
    # Check if main config file exists
    if [ ! -f "/etc/nginx/nginx.conf" ]; then
        echo "Main configuration file not found."
        return 1
    fi
    
    # Create a test index.html file
    echo "Creating test index.html..."
    sudo mkdir -p /usr/share/nginx/html
    echo "<html><body><h1>Nginx Test Page</h1></body></html>" | sudo tee /usr/share/nginx/html/index.html > /dev/null
    sudo chmod 644 /usr/share/nginx/html/index.html
    
    # Check configuration syntax
    echo "Validating configuration syntax..."
    if ! sudo nginx -t; then
        echo "Configuration syntax error."
        return 1
    fi
    
    return 0
}

# Function to check firewall status and open port if needed
check_firewall() {
    if command -v firewall-cmd &> /dev/null; then
        echo "Checking firewall status..."
        if ! sudo firewall-cmd --quiet --list-services | grep -q "http"; then
            echo "Opening HTTP port in firewall..."
            sudo firewall-cmd --add-service=http --permanent
            sudo firewall-cmd --reload
        fi
    fi
}

# Function to test Nginx HTTP response
test_nginx_http() {
    echo "Testing nginx HTTP response..."
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
            echo "Failed. HTTP status code: $curl_response, curl return code: $curl_status"
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
        echo "---- SELinux status ----"
        sestatus || echo "Could not get SELinux status"
        
        echo "---- Firewall status ----"
        sudo firewall-cmd --list-all || echo "Could not get firewall status"
        
        echo "---- Nginx service status ----"
        sudo systemctl status nginx.service || echo "Could not get service status"
        
        echo "---- Nginx server ports ----"
        sudo netstat -tulpn | grep nginx || sudo ss -tulpn | grep nginx || echo "Could not get nginx ports"
        
        echo "---- Log files ----"
        sudo tail -20 /var/log/nginx/error.log || echo "Could not read error log"
        
        return 1
    fi
}

# Main script execution starts here
main() {
    echo "Starting nginx test..."
    
    # Check if Nginx is installed
    if is_nginx_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        # Attempt to install the Nginx package
        if install_nginx_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            return 1
        fi
    fi
    
    # Get the correct package version
    PACKAGE_VERSION=$(rpm -q --queryformat "%{VERSION}" $PACKAGE_NAME)
    echo "Nginx version: $PACKAGE_VERSION"
    
    # Check and fix configuration
    if ! check_nginx_config; then
        echo "Failed to configure nginx properly."
        return 1
    fi
    
    # Check firewall settings
    check_firewall
    
    # Check if Nginx service is running
    if is_nginx_running; then
        echo "Nginx service is already running."
    else
        echo "Nginx service is not running. Attempting to start..."
        if start_nginx_service; then
            echo "Nginx service started successfully."
            # Give it a moment to fully start
            sleep 2
        else
            echo "Failed to start Nginx service."
            return 1
        fi
    fi
    
    # Test Nginx HTTP response
    if test_nginx_http; then
        echo "Nginx service is active and responding."
        return 0
    else
        echo "Nginx service is active but not responding."
        return 1
    fi
}

# Call the main function
main