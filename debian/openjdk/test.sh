#!/bin/bash

# Define the package details
PACKAGE_NAME="openjdk-*-jdk"
PACKAGE_SHOW_NAME="OpenJDK"
PACKAGE_TYPE="Java Development Kit"
REPORT_FILE="report.json"

# Function to check if OpenJDK is installed
is_openjdk_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install OpenJDK package
install_openjdk_package() {
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

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local java_version=$(java -version 2>&1 | awk -F '"' '/version/ {print $2}')
    local test_name="OpenJDK Functionality Test"
    local test_passed=false

    # Check OpenJDK functionality
    if test_openjdk_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$java_version",
    "package_name": "$PACKAGE_SHOW_NAME",
    "package_type": "$PACKAGE_TYPE",
    "test_results": [
        {
            "test_name": "$test_name",
            "passed": $test_passed
        }
    ],
    "all_tests_passed": $test_passed
}
EOF
)

    # Write the report to the file
    echo "$report_content" >$REPORT_FILE
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
        exit 1
    fi
fi

# Check OpenJDK functionality by compiling and running a simple Java program
if test_openjdk_functionality; then
    echo "OpenJDK is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "OpenJDK is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script