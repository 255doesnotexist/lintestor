#!/bin/bash

PACKAGE_NAME="haproxy"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | awk '{print $3}')
PACKAGE_PRETTY_NAME="HAProxy"
PACKAGE_TYPE="Load Balancer"
PACKAGE_DESCRIPTION="Reliable, high performance TCP/HTTP load balancer"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
