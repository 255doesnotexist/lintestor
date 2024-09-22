#!/bin/bash

PACKAGE_NAME="redis-server"
PACKAGE_VERSION=$(redis-server --version | awk '{print $3}' | cut -d '=' -f2)
PACKAGE_PRETTY_NAME="Redis"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="Persistent key-value database, server side"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
