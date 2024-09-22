#!/bin/bash

PACKAGE_NAME="python3-scipy"
python_version=$(python3 --version | awk '{print $2}')
scipy_version=$(python3 -c "import scipy; print(scipy.__version__)")
PACKAGE_VERSION="$scipy_version (python $python_version)"
PACKAGE_PRETTY_NAME="SciPy"
PACKAGE_TYPE="Python Library"
PACKAGE_DESCRIPTION="Scientific Tools for Python"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
