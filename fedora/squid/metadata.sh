#!/bin/bash

PACKAGE_NAME="squid"
PACKAGE_VERSION=$(squid -v | head -n1 | awk '{print $4}')
PACKAGE_PRETTY_NAME="Squid"
PACKAGE_TYPE="Proxy Server"
PACKAGE_DESCRIPTION="Caching Proxy for the Web"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION