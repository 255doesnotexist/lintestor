#!/bin/bash

PACKAGE_NAME="nginx"
PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Nginx"
PACKAGE_TYPE="Web Server"
PACKAGE_DESCRIPTION="high-performance web server and a reverse proxy server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION