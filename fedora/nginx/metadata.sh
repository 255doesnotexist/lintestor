#!/bin/bash

PACKAGE_NAME="nginx"

# Check if package is installed first
if rpm -q $PACKAGE_NAME &>/dev/null; then
    # Use queryformat for reliable version extraction
    PACKAGE_VERSION=$(rpm -q --queryformat "%{VERSION}" $PACKAGE_NAME)
else
    PACKAGE_VERSION="not installed"
fi

PACKAGE_PRETTY_NAME="Nginx"
PACKAGE_TYPE="Web Server"
PACKAGE_DESCRIPTION="high-performance web server and a reverse proxy server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION