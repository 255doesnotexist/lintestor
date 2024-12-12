#!/bin/bash

# Function to check if a package is installed
check_package() {
    if ! dpkg -l "$1" >/dev/null 2>&1; then
        echo "Installing $1..."
        sudo apt install -y "$1"
    else
        echo "$1 is already installed"
    fi
}

# Check and install required packages
echo "Installing dependencies..."
sudo apt update
check_package qemu-system-riscv64
check_package u-boot-qemu
check_package wget
check_package gzip
check_package expect

# Check if image already exists
if [ ! -f "fedora-disk-gcc.raw" ]; then
    if [ ! -f "fedora-disk-gcc.raw.gz" ]; then
        echo "Downloading Fedora RISC-V image..."
        wget https://openkoji.iscas.ac.cn/pub/temp/fedora-disk-gcc.raw.gz
    fi
    
    echo "Decompressing image..."
    gzip -d fedora-disk-gcc.raw.gz
else
    echo "Image file already exists, skipping download and decompression"
fi


# Check for existing SSH keys or generate new ones
echo "Checking for existing SSH keys..."
SSH_KEY_FILE="$HOME/.ssh/id_rsa.pub"
if [ ! -f "$SSH_KEY_FILE" ]; then
    echo "No SSH key found, generating one..."
    ssh-keygen -t rsa -b 2048 -f "$HOME/.ssh/id_rsa" -N "" -q
    echo "New SSH key generated at $HOME/.ssh/id_rsa"
fi

# Read the public key into a variable
export SSH_PUBLIC_KEY=$(cat "$SSH_KEY_FILE")
echo "Using public key: $SSH_PUBLIC_KEY"

# Create expect script for auto login and configure SSH
cat > auto_config.exp << 'EXPECT'
#!/usr/bin/expect -f
set timeout 120

# Start QEMU and expect login prompt
spawn qemu-system-riscv64 -nographic \
    -machine virt \
    -m 4G \
    -smp 4 \
    -drive file=fedora-disk-gcc.raw,format=raw,if=virtio \
    -kernel /usr/lib/u-boot/qemu-riscv64_smode/uboot.elf \
    -netdev user,id=net0,hostfwd=tcp::2223-:22 \
    -device virtio-net-device,netdev=net0

# Wait for login prompt
expect "login:"
send "root\r"
expect "Password:"
send "riscv\r"

# Wait for shell prompt
expect "#"
# Configure sshd to allow public key authentication
send "mkdir -p /root/.ssh && chmod 700 /root/.ssh\r"
expect "#"
send "echo '$env(SSH_PUBLIC_KEY)' > /root/.ssh/authorized_keys\r"
expect "#"
send "chmod 600 /root/.ssh/authorized_keys\r"
expect "#"
# Restart sshd service
send "sed -i 's/^#*PubkeyAuthentication no/PubkeyAuthentication yes/' /etc/ssh/sshd_config\r"
expect "#"
send "systemctl restart sshd\r"
expect "#"
send "halt\r"
expect eof
EXPECT

# Make expect script executable
chmod +x auto_config.exp

# Start the system using expect script
./auto_config.exp

echo "Setup completed!"
