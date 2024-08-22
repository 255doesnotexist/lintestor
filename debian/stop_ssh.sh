#!/bin/bash

# Define the SSH connection parameters
# USER="root"
# PASSWORD="root"
# PORT=2222
# ADDRESS="localhost"

# Function to check if sshpass is installed
check_sshpass() {
    if ! command -v sshpass &> /dev/null; then
        echo "sshpass is not installed. Please install it to proceed."
        exit 1
    fi
}

# Function to stop environment via SSH
stop_environment_ssh() {
    local retries=3
    local delay=5

    for ((i=0; i<$retries; i++)); do
        if sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 "$USER@$ADDRESS" -p "$PORT" "halt" &> /dev/null; then
            echo "Successfully sent halt command to environment via SSH."
            return 0
        fi
        echo "Failed to send halt command. Retrying in $delay seconds..."
        sleep $delay
    done

    echo "Failed to stop environment via SSH after multiple attempts."
    return 1
}


# Main script execution starts here

# Check if sshpass is installed
check_sshpass

stop_environment_ssh

# Exit the script
exit 0
