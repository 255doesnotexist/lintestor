#!/bin/bash

PACKAGE_NAME="lighttpd"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Lighttpd"
PACKAGE_TYPE="Web Server"
PACKAGE_DESCRIPTION="Lighttpd - a secure, fast, compliant and very flexible web server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
