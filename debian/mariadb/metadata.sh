#!/bin/bash

PACKAGE_NAME="mariadb-server"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="MariaDB database server"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="MariaDB database server binaries"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
