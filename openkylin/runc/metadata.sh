#!/bin/bash

PACKAGE_NAME="runc"
PACKAGE_VERSION=$(runc --version | awk '/runc version/{print $3}')
PACKAGE_PRETTY_NAME="runc"
PACKAGE_TYPE="Container Runtime"
PACKAGE_DESCRIPTION="CLI tool for spawning and running containers according to the OCI specification"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
