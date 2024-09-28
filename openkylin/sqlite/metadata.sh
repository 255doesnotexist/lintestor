#!/bin/bash

PACKAGE_NAME="sqlite3"
PACKAGE_VERSION=$(sqlite3 --version | awk '{print $1}')
PACKAGE_PRETTY_NAME="SQLite 3"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="Command-line interface for SQLite 3"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
