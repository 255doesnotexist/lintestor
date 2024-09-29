#!/bin/bash

PACKAGE_NAME="openjdk-11-jdk"
PACKAGE_VERSION=$(java -version 2>&1 | awk -F '"' '/version/ {print $2}')
PACKAGE_PRETTY_NAME="OpenJDK Development Kit (JDK)"
PACKAGE_TYPE="Java Development Kit"
PACKAGE_DESCRIPTION="OpenJDK Development Kit (JDK)"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
