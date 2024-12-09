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
