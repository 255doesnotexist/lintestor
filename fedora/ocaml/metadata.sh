#!/bin/bash

PACKAGE_NAME="ocaml"
PACKAGE_VERSION=$(rpm -qi $PACKAGE_NAME | grep Version | awk '{print $2}')
PACKAGE_PRETTY_NAME="Objective Caml"
PACKAGE_TYPE="Programming Language"
PACKAGE_DESCRIPTION="The Objective Caml compiler"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION