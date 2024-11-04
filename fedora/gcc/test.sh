#!/bin/bash

# Define the package details
PACKAGE_NAME_MAKE="make"
PACKAGE_NAME_GCC="gcc"

# Function to check if a package is installed
is_package_installed() {
    rpm -q $1 > /dev/null 2>&1
    return $?
}

# Function to install a package
install_package() {
    dnf install -y $1
    return $?
}

# Function to check make version and functionality
test_make_functionality() {
    MAKE_VERSION=$(make --version | head -n1)
    if [[ -n $MAKE_VERSION ]]; then
        echo "$MAKE_VERSION"
        return 0
    else
        return 1
    fi
}

# Function to check gcc version and functionality
test_gcc_functionality() {
    GCC_VERSION=$(gcc --version | head -n1)
    if [[ -n $GCC_VERSION ]]; then
        echo "$GCC_VERSION"
        return 0
    else
        return 1
    fi
}

# Function to compile and run the test program
compile_and_run_test_program() {
    make clean && make
    if [ $? -eq 0 ]; then
        ./test_program
        return $?
    else
        return 1
    fi
}

# Function to check if the program output is as expected
check_program_output() {
    OUTPUT=$(./test_program)
    EXPECTED_OUTPUT="This program tests the availability of make and gcc.
If you can see this message, both make and gcc are working!

Testing add function: 10 + 5 = 15
Testing subtract function: 10 - 5 = 5"

    if [ "$OUTPUT" = "$EXPECTED_OUTPUT" ]; then
        echo "Program output is correct."
        return 0
    else
        echo "Program output is incorrect."
        echo "Expected output:"
        echo "$EXPECTED_OUTPUT"
        echo "Actual output:"
        echo "$OUTPUT"
        return 1
    fi
}

# Main function to run all tests
main() {
    local result=0

    # Check and install make if necessary
    if is_package_installed $PACKAGE_NAME_MAKE; then
        echo "Package $PACKAGE_NAME_MAKE is installed."
    else
        echo "Package $PACKAGE_NAME_MAKE is not installed. Attempting to install..."
        if install_package $PACKAGE_NAME_MAKE; then
            echo "Package $PACKAGE_NAME_MAKE installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME_MAKE."
            return 1
        fi
    fi

    # Check and install gcc if necessary
    if is_package_installed $PACKAGE_NAME_GCC; then
        echo "Package $PACKAGE_NAME_GCC is installed."
    else
        echo "Package $PACKAGE_NAME_GCC is not installed. Attempting to install..."
        if install_package $PACKAGE_NAME_GCC; then
            echo "Package $PACKAGE_NAME_GCC installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME_GCC."
            return 1
        fi
    fi

    # Check make functionality
    if test_make_functionality; then
        echo "make is functioning correctly."
    else
        echo "make is not functioning correctly."
        result=1
    fi

    # Check gcc functionality
    if test_gcc_functionality; then
        echo "gcc is functioning correctly."
    else
        echo "gcc is not functioning correctly."
        result=1
    fi

    # Compile and run the test program
    echo "Compiling and running the test program..."
    if compile_and_run_test_program; then
        echo "Test program compiled and ran successfully."
    else
        echo "Failed to compile or run the test program."
        result=1
    fi

    # Check if the program output is correct
    if check_program_output; then
        echo "All tests passed successfully."
    else
        echo "Program output check failed."
        result=1
    fi

    # Clean up compiled files
    echo "Cleaning up..."
    make clean
    
    if [ $result -eq 0 ]; then
        echo "All tests completed successfully."
    else
        echo "Some tests failed."
    fi

    return $result
}

# Run the main function and capture its return value
main
RESULT=$?

# Return the result (this will be an exit if the script is executed, or a return if sourced)
return $RESULT