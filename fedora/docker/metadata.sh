#!/bin/bash

PACKAGE_NAME="docker-ce"
PACKAGE_VERSION=$(rpm -qa | grep $PACKAGE_NAME | head -n 1 | awk '{print $2}')
PACKAGE_PRETTY_NAME="Docker Engine - Community Edition"
PACKAGE_TYPE="Container Platform"
PACKAGE_DESCRIPTION="Docker Engine - Community Edition"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION