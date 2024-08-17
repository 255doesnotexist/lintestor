#!/bin/bash

# the following environmental variables should be controlled directly from TestEnvManager
# USER="root"
# PASSWORD="root"
# PORT=2222
# ADDRESS="localhost"

# echo $USER
# echo $PASSWORD
# echo $PORT
# echo $ADDRESS

# Function to check if sshpass is installed
check_sshpass() {
    if ! command -v sshpass &> /dev/null; then
        echo "sshpass is not installed. Please install it to proceed."
        exit 1
    fi
}

# Function to check if SSH connection is possible
check_ssh_connection() {
    if sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 "$USER@$ADDRESS" -p "$PORT" "exit" &> /dev/null; then
        echo "Able to establish SSH connection."
        return 0
    else
        return 1
    fi
}

# Function to wait for environment to start and become accessible via SSH
wait_for_connection() {
    local retries=20
    local delay=5

    for ((i=0; i<$retries; i++)); do
        if check_ssh_connection; then
            echo "Environment is accessible via SSH."
            return 0
        fi
        echo "Environment not connected yet. Retrying in $delay seconds..."
        sleep $delay
    done

    echo "Failed to connect to environment via SSH."
    exit 1
}

# Main script execution starts here

# Check if sshpass is installed
check_sshpass

# Check if SSH connection is possible
if check_ssh_connection; then
    echo "Already connected to environment via SSH. Exiting..."
    exit 0
fi


# Wait for environment to become accessible via SSH
wait_for_connection

exit 0
