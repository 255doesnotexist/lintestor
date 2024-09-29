#!/bin/bash

PACKAGE_NAME="nginx"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Nginx"
PACKAGE_TYPE="Web Server"
PACKAGE_DESCRIPTION="high-performance web server and a reverse proxy server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
