#!/bin/bash

PACKAGE_NAME="varnish"
PACKAGE_VERSION=$(varnishd -V 2>&1 | grep -oP 'varnish-\K\d+\.\d+\.\d+' | head -1)
PACKAGE_PRETTY_NAME="Varnish Cache"
PACKAGE_TYPE="Caching Server"
PACKAGE_DESCRIPTION="High-performance HTTP accelerator"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION