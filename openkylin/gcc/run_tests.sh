#!/bin/bash

# Check if the test executable exists and is executable
if [ -x "$1" ]; then
    TEST_RESULT=$($1)
    return $?
else
    return 1
fi