#!/bin/bash

PACKAGE_NAME="clang"
PACKAGE_VERSION=$(clang --version | grep -oP "version\W?\K.*")
PACKAGE_PRETTY_NAME="Clang C/C++ Compiler"
PACKAGE_TYPE="Compiler Toolchain"
PACKAGE_DESCRIPTION="Clang C/C++ compiler, based on LLVM"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
