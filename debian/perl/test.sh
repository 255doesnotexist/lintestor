#!/bin/bash

# Define the package details
PACKAGE_NAME="perl"
PACKAGE_SHOW_NAME="Perl"
PACKAGE_TYPE="Practical Extraction and Reporting Language"
REPORT_FILE="report.json"

# Function to check if Perl is installed
is_perl_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install Perl package
install_perl_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME

    return $?
}

# Function to test Perl functionality
test_perl_functionality() {
    local temp_dir=$(mktemp -d)
    local perl_file="${temp_dir}/test.pl"
    local output_file="${temp_dir}/perl_output.txt"

    # Write a simple Perl script to test functionality
    cat <<'EOF' > "$perl_file"
#!/usr/bin/perl
use strict;
use warnings;

my @array = (1, 2, 3, 4, 5);
my $sum = 0;
$sum += $_ for @array;
print "The sum is: $sum\n";
EOF

    # Run the Perl script
    perl "$perl_file" > "$output_file"

    # Check if the script ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "The sum is: 15" ]]; then
        echo "Perl is functioning correctly."
        return 0
    else
        echo "Failed to run Perl test script."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local perl_version=$(perl -v | grep -oP "This is perl \K(\d+\.\d+\.\d+)")
    local test_name="Perl Functionality Test"
    local test_passed=false

    # Check Perl functionality
    if test_perl_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_version": "$perl_version",
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

# Check if Perl is installed
if is_perl_installed; then
    echo "Perl is installed."
else
    echo "Perl is not installed. Attempting to install..."
    # Attempt to install the Perl package
    if install_perl_package; then
        echo "Perl installed successfully."
    else
        echo "Failed to install Perl."
        exit 1
    fi
fi

# Check Perl functionality by running a simple Perl script
if test_perl_functionality; then
    echo "Perl is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "Perl is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script