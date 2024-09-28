#!/bin/bash

PACKAGE_NAME="perl"
PACKAGE_VERSION=$(perl -v | grep -oP "v\K(\d+\.\d+\.\d+)")
PACKAGE_PRETTY_NAME="Perl"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="Larry Wall's Practical Extraction and Report Language"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
