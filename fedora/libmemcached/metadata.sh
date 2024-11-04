#!/bin/bash

PACKAGE_NAME="libmemcached"
PACKAGE_VERSION=$(rpm -q --qf "%{VERSION}-%{RELEASE}" $PACKAGE_NAME)
PACKAGE_PRETTY_NAME="libmemcached"
PACKAGE_TYPE="Caching Library"
PACKAGE_DESCRIPTION="Client library to memcached servers"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION