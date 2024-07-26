#!/bin/bash

PACKAGE_NAME="ocaml"
PACKAGE_TYPE="Programming Language"
REPORT_FILE="report.json"

is_ocaml_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

install_ocaml_package() {
    apt-get update
    apt-get install -y $PACKAGE_NAME ocamlbuild
    return $?
}

test_ocaml_functionality() {
    local temp_dir=$(mktemp -d)
    local ocaml_file="${temp_dir}/hello.ml"
    local executable="${temp_dir}/hello"

    cat <<EOF > "$ocaml_file"
let () = print_endline "Hello, OCaml!"
EOF

    cd "$temp_dir"
    ocamlc -o hello hello.ml

    if [[ -x "$executable" ]]; then
        echo "Executable created successfully."
    else
        echo "Failed to create executable."
    fi

    local output=$("$executable")
    echo "Output of the program: $output"

    if [[ "$output" == "Hello, OCaml!" ]]; then
        echo "OCaml is functioning correctly."
        return 0
    else
        echo "Failed to compile or run OCaml test program."
        return 1
    fi
}

generate_report() {
    local os_version=$(cat /proc/version)
    local kernel_version=$(uname -r)
    local package_version=$(dpkg -l | grep $PACKAGE_NAME | head -n 1 | awk '{print $3}')
    local test_name="OCaml Functionality Test"
    local test_passed=false

    if test_ocaml_functionality; then
        test_passed=true
    fi

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

    echo "$report_content" >$REPORT_FILE
}

if is_ocaml_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    if install_ocaml_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        exit 1
    fi
fi

if test_ocaml_functionality; then
    echo "OCaml is functioning correctly."
    generate_report
    echo "Report generated at $REPORT_FILE"
else
    echo "OCaml is not functioning correctly."
    generate_report
    echo "Report generated at $REPORT_FILE with failed test."
fi
