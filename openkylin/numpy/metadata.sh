#!/bin/bash

PACKAGE_NAME="python3-numpy"
PACKAGE_VERSION="$(python3 --version) ($(python3 -c "import numpy; print(numpy.__version__)"))"
PACKAGE_PRETTY_NAME="NumPy"
PACKAGE_TYPE="Python Library"
PACKAGE_DESCRIPTION="Numerical Python adds a fast array facility to the Python language"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
