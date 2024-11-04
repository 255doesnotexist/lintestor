#!/bin/bash

# Define the package details
PACKAGE_NAME="perl"

# Function to check if Perl is installed
is_perl_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Function to install Perl package
install_perl_package() {
    dnf install -y $PACKAGE_NAME

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
        return 1
    fi
fi

PACKAGE_VERSION=$(perl -v | grep -oP "v\K(\d+\.\d+\.\d+)")

# Check Perl functionality by running a simple Perl script
if test_perl_functionality; then
    echo "Perl is functioning correctly."
    return 0
else
    echo "Perl is not functioning correctly."
    return 1
fi

# End of the script