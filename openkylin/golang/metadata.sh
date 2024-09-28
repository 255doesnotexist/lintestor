#!/bin/bash

PACKAGE_NAME="golang-go"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Go Programming Language"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Go programming language compiler, linker, and libraries"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
