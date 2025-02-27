#!/bin/bash

# Define package details
PACKAGE_NAME="erlang"

# Check if package is installed
is_package_installed() {
    rpm -q $PACKAGE_NAME > /dev/null 2>&1
    return $?
}

# Install Erlang package
install_erlang_package() {
    sudo dnf install -y $PACKAGE_NAME
    return $?
}

# Test Erlang functionality
test_erlang_functionality() {
    local initial_dir=$(pwd)
    local temp_dir=$(mktemp -d)
    local erlang_file="${temp_dir}/hello.erl"
    local module_name="hello"
    local erl_output

    # Create Erlang source file
    cat <<EOF > "$erlang_file"
-module($module_name).
-export([start/0]).

start() ->
    io:format("Hello, Erlang!~n").
EOF

    cd "$temp_dir"
    erl -compile $module_name
    if [[ -f "${module_name}.beam" ]]; then
        erl_output=$(erl -noshell -s $module_name start -s init stop)
        cd "$initial_dir"  # Return to initial directory
        if [[ "$erl_output" == "Hello, Erlang!" ]]; then
            return 0
        fi
    fi
    cd "$initial_dir"  # Return to initial directory
    return 1
}

# Main function logic
main() {
    # Check if package is installed
    if is_package_installed; then
        echo "Package $PACKAGE_NAME is installed."
    else
        echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
        if install_erlang_package; then
            echo "Package $PACKAGE_NAME installed successfully."
        else
            echo "Failed to install package $PACKAGE_NAME."
            return 1
        fi
    fi

    PACKAGE_VERSION=$(rpm -q $PACKAGE_NAME | awk '{print $2}')
    # Test Erlang functionality
    if test_erlang_functionality; then
        echo "Erlang is functioning correctly."
        return 0
    else
        echo "Erlang is not functioning correctly."
        return 1
    fi
}

# Execute main function
main
