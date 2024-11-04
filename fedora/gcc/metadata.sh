#!/bin/bash

PACKAGE_NAME="gcc"
PACKAGE_VERSION=$(gcc --version | head -n1 | awk '{print $4}')
PACKAGE_PRETTY_NAME="GNU Compiler Collection"
PACKAGE_TYPE="Compiler Toolchain"
PACKAGE_DESCRIPTION="The GNU Compiler Collection"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION