#!/bin/bash

PACKAGE_NAME="llvm"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="LLVM Compiler Infrastructure"
PACKAGE_TYPE="Compiler Toolchain"
PACKAGE_DESCRIPTION="The LLVM Compiler Infrastructure"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
