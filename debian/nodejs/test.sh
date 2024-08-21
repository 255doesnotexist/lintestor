#!/bin/bash

PACKAGE_NAME="nodejs"

is_nodejs_installed() {
    dpkg -l | grep -qw $PACKAGE_NAME
    return $?
}

install_nodejs_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

test_nodejs_functionality() {
    local temp_dir=$(mktemp -d)
    echo "Temp dir: $temp_dir"

    local js_file="${temp_dir}/hello.js"

    cat <<EOF > "$js_file"
console.log('Hello, Node.js!');
EOF

    local output=$(node "$js_file")
    if [[ "$output" == "Hello, Node.js!" ]]; then
        echo "Node.js is functioning correctly."
        return 0
    else
        echo "Failed to run Node.js test program correctly."
        return 1
    fi
}

if is_nodejs_installed; then
    echo "Package $PACKAGE_NAME is installed."
else
    echo "Package $PACKAGE_NAME is not installed. Attempting to install..."
    if install_nodejs_package; then
        echo "Package $PACKAGE_NAME installed successfully."
    else
        echo "Failed to install package $PACKAGE_NAME."
        return 1
    fi
fi

if test_nodejs_functionality; then
    echo "Node.js is functioning correctly."
    return 0
else
    echo "Node.js is not functioning correctly."
    return 1
fi
