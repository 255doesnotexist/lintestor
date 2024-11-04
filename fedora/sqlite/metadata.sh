#!/bin/bash

PACKAGE_NAME="sqlite"
PACKAGE_VERSION=$(sqlite3 --version | awk '{print $1}')
PACKAGE_PRETTY_NAME="SQLite"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="Command-line interface for SQLite"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION