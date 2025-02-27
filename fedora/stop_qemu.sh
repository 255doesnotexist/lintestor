#!/bin/bash

# Define the SSH connection parameters
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

# Function to stop QEMU via SSH
stop_qemu_ssh() {
    local retries=3
    local delay=5

    for ((i=0; i<$retries; i++)); do
        if sshpass -p "$PASSWORD" ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 "$USER@$ADDRESS" -p "$PORT" "sudo halt" &> /dev/null; then
            echo "Successfully sent halt command to QEMU via SSH."
            return 0
        fi
        echo "Failed to send halt command. Retrying in $delay seconds..."
        sleep $delay
    done

    echo "Failed to stop QEMU via SSH after multiple attempts."
    return 1
}

# Function to kill QEMU processes
kill_qemu_processes() {
    echo "Killing QEMU processes on the local machine."
    pkill -f qemu-system-riscv64
}

# Main script execution starts here

# Check if sshpass is installed
check_sshpass

# Attempt to stop QEMU via SSH
if ! stop_qemu_ssh; then
    # If SSH halt command fails, force kill QEMU processes
    kill_qemu_processes
fi

# Exit the script
exit 0
