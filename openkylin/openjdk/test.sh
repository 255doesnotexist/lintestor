#!/bin/bash

# Define the package details
PACKAGE_NAME="openjdk-11-jdk"
PACKAGE_SHOW_NAME="OpenJDK"
PACKAGE_TYPE="Java Development Kit"
REPORT_FILE="report.json"

# Function to check if OpenJDK is installed
is_openjdk_installed() {
    # as real package name in `dpkg -l`'s output looks like "openjdk-11-jdk:riscv64" 
    # cannot be word matched by pattern "openjdk-11-jdk" 
    # so removed -w option from grep is a workaround
    # but as dpkg's -l [pattern] option could match package pattern correctly,
    # it is better to just use `dpkg -l` to check package status
    # if the return value is 0, that means a package is already installed
    # or the package is not installed, the return value is 1
    dpkg -l $PACKAGE_NAME
    return $?
}

# Function to install OpenJDK package
install_openjdk_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME

    return $?
}

# Function to test OpenJDK functionality
test_openjdk_functionality() {
    local temp_dir=$(mktemp -d)
    local java_file="${temp_dir}/TestJava.java"
    local class_file="${temp_dir}/TestJava.class"
    local output_file="${temp_dir}/java_output.txt"

    # Write a simple Java program to test OpenJDK functionality
    cat <<EOF > "$java_file"
public class TestJava {
    public static void main(String[] args) {
        System.out.println("OpenJDK is working!");
    }
}
EOF

    # Compile the Java file
    javac "$java_file"

    # Check if compilation was successful
    if [ ! -f "$class_file" ]; then
        echo "Failed to compile Java test file."
        return 1
    fi

    # Run the compiled Java program
    java -cp "$temp_dir" TestJava > "$output_file"

    # Check if the program ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "OpenJDK is working!" ]]; then
        echo "OpenJDK is functioning correctly."
        return 0
    else
        echo "Failed to run OpenJDK test program."
        return 1
    fi
}

# Main script execution starts here

# Check if OpenJDK is installed
if is_openjdk_installed; then
    echo "OpenJDK is installed."
else
    echo "OpenJDK is not installed. Attempting to install..."
    # Attempt to install the OpenJDK package
    if install_openjdk_package; then
        echo "OpenJDK installed successfully."
    else
        echo "Failed to install OpenJDK."
        return 1
    fi
fi

PACKAGE_VERSION=$(java -version 2>&1 | awk -F '"' '/version/ {print $2}')

# Check OpenJDK functionality by compiling and running a simple Java program
if test_openjdk_functionality; then
    echo "OpenJDK is functioning correctly."
    return 0
else
    echo "OpenJDK is not functioning correctly."
    return 1
fi

# End of the script