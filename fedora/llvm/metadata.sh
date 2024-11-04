#!/bin/bash

PACKAGE_NAME="llvm"
PACKAGE_VERSION=$(rpm -q --queryformat "%{VERSION}\n" $PACKAGE_NAME)
PACKAGE_PRETTY_NAME="LLVM Compiler Infrastructure"
PACKAGE_TYPE="Compiler Toolchain"
PACKAGE_DESCRIPTION="The LLVM Compiler Infrastructure"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION