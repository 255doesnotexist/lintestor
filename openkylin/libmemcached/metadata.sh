#!/bin/bash

PACKAGE_NAME="libmemcached11t64"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="libmemcached"
PACKAGE_TYPE="Caching Library"
PACKAGE_DESCRIPTION="Client library to memcached servers (transitional package)"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
