#!/bin/bash

# Save current directory
ORIGINAL_DIR=$(pwd)

# Create and enter fedora directory
echo "Setting up Fedora RISC-V environment..."
mkdir -p ../fedora
cd ../fedora || exit 1

# Create init.sh
echo "Creating init.sh..."
cat > init.sh << 'EOL'
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

echo "Setup completed!"
EOL

# Create boot.sh
echo "Creating boot.sh..."
cat > boot.sh << 'EOL'
#!/bin/bash

# Call init.sh to ensure dependencies are installed
./init.sh

# Create expect script for auto login and sshd configuration
cat > auto_config.exp << 'EXPECT'
#!/usr/bin/expect -f
set timeout -1

# Start QEMU and expect login prompt
spawn qemu-system-riscv64 -nographic \
    -machine virt \
    -m 4G \
    -smp 4 \
    -drive file=fedora-disk-gcc.raw,format=raw,if=virtio \
    -kernel /usr/lib/u-boot/qemu-riscv64_smode/uboot.elf \
    -netdev user,id=net0,hostfwd=tcp::2222-:22 \
    -device virtio-net-device,netdev=net0

# Wait for login prompt
expect "login:"
send "root\r"
expect "Password:"
send "riscv\r"

# Wait for shell prompt
expect "#"
# Configure sshd to allow password authentication
send "sed -i 's/#PasswordAuthentication yes/PasswordAuthentication yes/' /etc/ssh/sshd_config\r"
expect "#"
send "sed -i 's/PasswordAuthentication no/PasswordAuthentication yes/' /etc/ssh/sshd_config\r"
expect "#"
# Start sshd service
send "systemctl restart sshd\r"
expect "#"

# Keep the session alive
interact
EXPECT

# Make expect script executable
chmod +x auto_config.exp

# Start the system using expect script
./auto_config.exp
EOL

# Make scripts executable
chmod +x init.sh boot.sh

# Run initial setup
echo "Running initial setup..."
./init.sh

# Return to original directory
cd "$ORIGINAL_DIR" || exit 1

echo "Setup complete! You can now:"
echo "1. cd ../fedora"
echo "2. ./boot.sh to start the system"
echo "3. After system boots, use 'ssh -p 2222 root@localhost' to connect (password: riscv)"