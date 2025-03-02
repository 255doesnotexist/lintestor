#!/bin/bash

# Function to check if a package is installed
check_package() {
    if ! dpkg -l "$1" >/dev/null 2>&1; then
        if [ "$UPDATE_DONE" != "yes" ]; then
            echo "Updating package lists..."
            sudo apt update
            UPDATE_DONE="yes"
        fi
        echo "Installing $1..."
        sudo apt install -y "$1"
    else
        echo "$1 is already installed"
    fi
}

# Initialize update flag
UPDATE_DONE="no"

# Check and install required packages
echo "Installing dependencies..."
check_package qemu-system-riscv64
check_package wget
check_package cloud-image-utils  # For cloud-init configuration
check_package expect

# Check if firmware files exist, download if not
if [ ! -f "RISCV_VIRT_CODE.fd" ]; then
    echo "Downloading RISC-V UEFI code firmware..."
    wget http://repo.openeuler.org/openEuler-24.03-LTS-SP1/virtual_machine_img/riscv64/RISCV_VIRT_CODE.fd
fi

if [ ! -f "RISCV_VIRT_VARS.fd" ]; then
    echo "Downloading RISC-V UEFI variables firmware..."
    wget http://repo.openeuler.org/openEuler-24.03-LTS-SP1/virtual_machine_img/riscv64/RISCV_VIRT_VARS.fd
fi

# Check if cloud image exists
IMAGE_URL="https://dl.fedoraproject.org/pub/alt/risc-v/release/41/Cloud/riscv64/images/Fedora-Cloud-Base-Generic-41.20250224-1026a2d0e311.riscv64.qcow2"
IMAGE_NAME="fedora-riscv64.qcow2"

if [ ! -f "$IMAGE_NAME" ]; then
    echo "Downloading Fedora RISC-V cloud image..."
    wget "$IMAGE_URL" -O "$IMAGE_NAME"
else
    echo "Image file already exists, skipping download"
fi

# Check if the VM has been initialized already
if [ ! -f ".initialized" ]; then
    # Create cloud-init configuration
    echo "Creating cloud-init configuration..."
    cat > user-data << 'USERDATA'
#cloud-config

ssh_pwauth: true

users:
  - default
  - name: root
    lock_passwd: false
disable_root: false

password: linux
chpasswd:
  expire: false

ssh_pwauth: true

# Enable SSH password authentication and root login
# ssh_authorized_keys: []

runcmd:
  - sed -i 's/^#*PermitRootLogin.*$/PermitRootLogin yes/' /etc/ssh/sshd_config
  - sed -i 's/^#*PasswordAuthentication.*$/PasswordAuthentication yes/' /etc/ssh/sshd_config
  - systemctl restart sshd
  - touch /etc/cloud/cloud-init.disabled
  - shutdown -h now  # Shutdown after completing cloud-init
USERDATA

    cat > meta-data << 'METADATA'
instance-id: fedora-riscv
local-hostname: fedora-riscv
METADATA

    echo "Creating cloud-init ISO..."
    cloud-localds cloud-init.iso user-data meta-data

    # Create initialization boot script with fixed QEMU parameters
    cat > init_boot.sh << 'INITBOOT'
#!/bin/bash
qemu-system-riscv64 -nographic \
    -machine virt \
    -smp 4 -m 8G \
    -drive file=RISCV_VIRT_CODE.fd,if=pflash,format=raw,readonly=on \
    -drive file=RISCV_VIRT_VARS.fd,if=pflash,format=raw \
    -drive file=fedora-riscv64.qcow2,format=qcow2,if=virtio \
    -drive file=cloud-init.iso,format=raw,if=virtio \
    -object rng-random,filename=/dev/urandom,id=rng0 \
    -device virtio-rng-device,rng=rng0 \
    -netdev user,id=net0,hostfwd=tcp::2223-:22 \
    -device virtio-net-device,netdev=net0
INITBOOT

    chmod +x init_boot.sh

    # Run the VM to initialize it
    echo "Starting QEMU VM for initial configuration..."
    echo "This will take some time as cloud-init configures the system."
    echo "Please wait for the VM to boot and for the cloud-init process to complete."
    echo "Once you see a login prompt, you can log in as 'fedora' with password 'linux'."
    echo "Then run 'sudo poweroff' to shut down the VM."
    echo ""
    # echo "Press Enter to continue..."
    # read

    # Run the initialization
    ./init_boot.sh

    # Mark as initialized
    touch .initialized
    
    # Cleanup cloud-init files
    echo "Cleaning up cloud-init files..."
    rm -f user-data meta-data cloud-init.iso init_boot.sh
else
    echo "VM has already been initialized."
fi

# Create normal boot script for future use with fixed QEMU parameters
cat > boot.sh << 'BOOT'
#!/bin/bash
qemu-system-riscv64 -nographic \
    -machine virt \
    -smp 4 -m 8G \
    -drive file=RISCV_VIRT_CODE.fd,if=pflash,format=raw,readonly=on \
    -drive file=RISCV_VIRT_VARS.fd,if=pflash,format=raw \
    -drive file=fedora-riscv64.qcow2,format=qcow2,if=virtio \
    -object rng-random,filename=/dev/urandom,id=rng0 \
    -device virtio-rng-device,rng=rng0 \
    -netdev user,id=net0,hostfwd=tcp::2223-:22 \
    -device virtio-net-device,netdev=net0
BOOT

chmod +x boot.sh

echo "Setup completed!"
echo "You can now start the VM using ./boot.sh"
echo "To SSH into the VM: ssh -p 2223 fedora@localhost"
echo "Default password: linux"