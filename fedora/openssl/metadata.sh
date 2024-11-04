#!/bin/bash

PACKAGE_NAME="openssl"
PACKAGE_VERSION=$(openssl version | awk '{print $2}')
PACKAGE_PRETTY_NAME="OpenSSL"
PACKAGE_TYPE="Cryptography Library"
PACKAGE_DESCRIPTION="Secure Sockets Layer toolkit - cryptographic utility"

echo "$PACKAGE_VERSION"
echo "$PACKAGE_PRETTY_NAME"
echo "$PACKAGE_TYPE"
echo "$PACKAGE_DESCRIPTION"