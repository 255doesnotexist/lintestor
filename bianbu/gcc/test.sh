#!/bin/bash
PACKAGE_VERSION=$(gcc --version | head -n1 | cut -d' ' -f4)
make test
rm -rf ./temp