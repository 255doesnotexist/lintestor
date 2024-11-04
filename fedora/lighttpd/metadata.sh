#!/bin/bash

PACKAGE_NAME="lighttpd"
PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Lighttpd"
PACKAGE_TYPE="Web Server"
PACKAGE_DESCRIPTION="Lighttpd - a secure, fast, compliant and very flexible web server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION