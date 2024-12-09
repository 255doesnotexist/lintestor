#!/bin/bash

# Call init.sh to ensure dependencies are installed
./init.sh

# Check for existing SSH keys or generate new ones
echo "Checking for existing SSH keys..."
SSH_KEY_FILE="$HOME/.ssh/id_rsa.pub"
if [ ! -f "$SSH_KEY_FILE" ]; then
    echo "No SSH key found, generating one..."
    ssh-keygen -t rsa -b 2048 -f "$HOME/.ssh/id_rsa" -N "" -q
    echo "New SSH key generated at $HOME/.ssh/id_rsa"
fi

# Read the public key into a variable
SSH_PUBLIC_KEY=$(cat "$SSH_KEY_FILE")
echo "Using public key: $SSH_PUBLIC_KEY"

# Create expect script for auto login and configure SSH
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

# Keep the session alive
interact
EXPECT

# Export the public key to the environment for expect to use
export SSH_PUBLIC_KEY="$SSH_PUBLIC_KEY"

# Make expect script executable
chmod +x auto_config.exp

# Start the system using expect script
./auto_config.exp
