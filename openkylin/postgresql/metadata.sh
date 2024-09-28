#!/bin/bash

PACKAGE_NAME="postgresql"
PACKAGE_VERSION=$(psql --version | awk '{print $3}')
PACKAGE_PRETTY_NAME="PostgreSQL"
PACKAGE_TYPE="Database"
PACKAGE_DESCRIPTION="Object-relational SQL database, version 16 server"

echo $PACKAGE_VERSION
echo $PACKAGE_PRETTY_NAME
echo $PACKAGE_TYPE
echo $PACKAGE_DESCRIPTION
