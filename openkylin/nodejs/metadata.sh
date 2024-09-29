#!/bin/bash

PACKAGE_NAME="nodejs"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Node.js"
PACKAGE_TYPE="Javascript Runtime"
PACKAGE_DESCRIPTION="Evented I/O for V8 JavaScript - runtime executable"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
