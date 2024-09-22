#!/bin/bash

PACKAGE_NAME="zookeeper"
PACKAGE_VERSION=$(/opt/zookeeper/bin/zkServer.sh version 2>&1 | grep -oP 'version \K[0-9.]+' | head -1)
PACKAGE_PRETTY_NAME="ZooKeeper"
PACKAGE_TYPE="Distributed Coordination Service"
PACKAGE_DESCRIPTION="A high-performance coordination service for distributed applications"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
