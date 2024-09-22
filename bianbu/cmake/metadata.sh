#!/bin/bash

PACKAGE_NAME="cmake"
PACKAGE_VERSION=$(cmake --version | head -n1 | cut -d' ' -f3)
PACKAGE_PRETTY_NAME="CMake"
PACKAGE_TYPE="Build System"
PACKAGE_DESCRIPTION="Cross-platform make"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
