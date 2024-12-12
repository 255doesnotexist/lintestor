#!/bin/bash

qemu-system-riscv64 -nographic \
    -machine virt \
    -m 4G \
    -smp 4 \
    -drive file=fedora-disk-gcc.raw,format=raw,if=virtio \
    -bios /usr/lib/riscv64-linux-gnu/opensbi/generic/fw_jump.bin \
    -kernel /usr/lib/u-boot/qemu-riscv64_smode/uboot.elf \
    -netdev user,id=net0,hostfwd=tcp::2223-:22 \
    -device virtio-net-device,netdev=net0
    