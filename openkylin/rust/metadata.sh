#!/bin/bash

PACKAGE_NAME="rustc"
PACKAGE_VERSION=$(rustc --version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Rust"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Rust compiler"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
