#!/bin/bash

PACKAGE_NAME="golang"
PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Go Programming Language"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Go programming language compiler, linker, and libraries"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION