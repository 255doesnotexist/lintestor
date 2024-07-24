#!/bin/bash

# Define the package details
PACKAGE_NAME="ocaml"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

# Function to check if OCaml is installed
is_ocaml_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

# Function to install OCaml package
install_ocaml_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to compile a simple OCaml program and test functionality
test_ocaml_functionality() {
    local temp_dir=$(mktemp -d)
    local ocaml_file="${temp_dir}/hello.ml"
    local executable="${temp_dir}/hello"

    # Write a simple OCaml program to test compilation
    cat <<EOF > "$ocaml_file"
let print_hello () =
  Printf.printf "Hello, OCaml!" ();

print_hello ()
EOF

    # Compile the OCaml program
    ocamlbuild "$ocaml_file"

    # Check if the executable was created and runs without error
    if [[ -x "$executable" && "$("${executable}")" == "Hello, OCaml!" ]]; then
        echo "OCaml is functioning correctly."
        return 0
    else
        echo "Failed to compile or run OCaml test program."
        return 1
    fi
}

# Function to generate the report.json
generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="OCaml Functionality Test"
    local test_passed=false

    # Check OCaml functionality
    if test_ocaml_functionality; then
        test_passed=true
    fi

    # Prepare the report content
    local report_content=$(cat <<EOF
{
    "distro": "debian",
    "os_version": "$os_version",
    "kernel_version": "$kernel_version",
    "package_name": "$PACKAGE_NAME",
    "package_type": "$PACKAGE_TYPE",
    "package_version": "$package_version",
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

# Check if OCaml is installed
if is_ocaml_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    # Attempt to install the OCaml package
    if install_ocaml_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

# Check OCaml functionality by compiling and running a simple OCaml program
if test_ocaml_functionality; then
    echo "OCaml is functioning correctly."
    # Generate the report
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "OCaml is not functioning correctly."
    # Generate the report with test failed
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi

# End of the script
