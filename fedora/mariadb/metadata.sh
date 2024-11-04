#!/bin/bash

PACKAGE_NAME="mariadb-server"
PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
PACKAGE_PRETTY_NAME="MariaDB database server"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="MariaDB database server binaries"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION