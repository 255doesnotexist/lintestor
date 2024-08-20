#!/bin/bash

# PACKAGE_VERSION needs to be specified as a global variable, either manually or through commands
# It could be defined anywhere in the script (I think); just make sure you defined it in every script
PACKAGE_VERSION="1.0.0-justatest"
# PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')

echo "This is an example dummy test script..."

# Do your stuff here
sleep 1

# Do NOT use "exit" throughout your script; use "return" instead, 
# otherwise the PACKAGE_VERSION could not be fetched properly in report.json
return 0
