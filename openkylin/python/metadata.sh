#!/bin/bash

PACKAGE_NAME="python3"
PACKAGE_VERSION=$(python3 --version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Python 3"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Interactive high-level object-oriented language (default python3 version)"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
