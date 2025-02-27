#!/bin/bash

# Define the directory and the script to execute
SCRIPT_DIR="../fedora/"  # Replace with the directory containing boot.sh
STARTUP_SCRIPT="boot.sh"
USER="fedora"
PASSWORD="linux"
PORT=2223
ADDRESS="localhost"

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
        echo "Successfully connected to QEMU via SSH."
        return 0
    else
        return 1
    fi
}

# Function to start QEMU in the background
start_qemu() {
    # Change to the directory containing the boot script and execute it in a new bash session
    bash -c "cd $SCRIPT_DIR && nohup ./$STARTUP_SCRIPT > qemu.log 2>&1 &" &
}

# Function to wait for QEMU to start and become accessible via SSH
wait_for_qemu() {
    local retries=2000 # 不知道为什么这个 fedora 开机很慢
    local delay=5

    for ((i=0; i<$retries; i++)); do
        if check_ssh_connection; then
            echo "QEMU has started and is accessible via SSH."
            return 0
        fi
        echo "QEMU not accessible via SSH yet. Retrying in $delay seconds..."
        sleep $delay
    done

    echo "Failed to connect to QEMU via SSH after multiple attempts."
    exit 1
}

# Main script execution starts here

# Check if sshpass is installed
check_sshpass

# Check if SSH connection is possible
if check_ssh_connection; then
    echo "Already connected to QEMU via SSH. Exiting..."
    exit 0
fi

# Start QEMU in the background
start_qemu

# Wait for QEMU to start and become accessible via SSH
wait_for_qemu

# The QEMU process should be running by now. Exit this script but keep QEMU running.
exit 0
