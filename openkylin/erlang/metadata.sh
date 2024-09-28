#!/bin/bash

PACKAGE_NAME="erlang"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Erlang Programming Language"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="A programming language used to build massively scalable soft real-time systems with requirements on high availability"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
