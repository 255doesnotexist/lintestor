#!/bin/bash

PACKAGE_NAME="ruby"
PACKAGE_VERSION=$(ruby --version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Ruby"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Interpreter of object-oriented scripting language Ruby"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
