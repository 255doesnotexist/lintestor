#!/bin/bash

PACKAGE_NAME="ocaml"
PACKAGE_VERSION=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
PACKAGE_PRETTY_NAME="Objective Caml"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="The Objective Caml compiler"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
