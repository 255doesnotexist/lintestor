#!/bin/bash

# PACKAGE_VERSION needs to be specified as a global variable, either manually or through commands
# It could be defined anywhere in the script (I think); just make sure you defined it in every script
PACKAGE_VERSION="1.0.0-justatest"
# PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')

echo "This is an example dummy test script..."

# Do your stuff here
sleep 1

# Do NOT use "exit" at all times; use "return" instead, 
# otherwise the PACKAGE_VERSION could not be fetched properly in report.json

# note that when used in sub-functions, "return" may only be able to jump out of the function,
# not exiting from the script itself. If a main function is used, consider adding
# related checks there to ensure that the script will exit on error.
return 0