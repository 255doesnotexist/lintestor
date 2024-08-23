#!/bin/bash

# Define the package details
PACKAGE_NAME="ruby"

# Function to check if Ruby is installed
is_ruby_installed() {
    command -v ruby >/dev/null 2>&1
    return $?
}

# Function to install Ruby package
install_ruby_package() {
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y $PACKAGE_NAME
    return $?
}

# Function to test Ruby functionality
test_ruby_functionality() {
    local temp_dir=$(mktemp -d)
    local ruby_file="${temp_dir}/test.rb"
    local output_file="${temp_dir}/ruby_output.txt"

    # Write a simple Ruby script to test functionality
    cat <<EOF > "$ruby_file"
# Test basic arithmetic
raise "Basic arithmetic failed" unless 2 + 2 == 4

# Test string manipulation
raise "String manipulation failed" unless "hello".capitalize == "Hello"

# Test array operations
arr = [1, 2, 3]
arr << 4
raise "Array operations failed" unless arr == [1, 2, 3, 4]

# Test hash operations
hash = { "a" => 1, "b" => 2 }
hash["c"] = 3
raise "Hash operations failed" unless hash == { "a" => 1, "b" => 2, "c" => 3 }

# Test basic file I/O
File.write("./test_file.txt", "Hello, Ruby!")
content = File.read("./test_file.txt")
raise "File I/O failed" unless content == "Hello, Ruby!"

puts "All tests passed successfully!"
EOF

    # Run the Ruby script
    ruby "$ruby_file" > "$output_file" 2>&1

    # Check if the script ran successfully and the output is as expected
    if [[ -f "$output_file" && "$(cat "$output_file")" == "All tests passed successfully!" ]]; then
        echo "Ruby is functioning correctly."
        return 0
    else
        echo "Failed to run Ruby test script."
        cat "$output_file"  # Print the actual output for debugging
        return 1
    fi
}

# Main script execution starts here

# Check if Ruby is installed
if is_ruby_installed; then
    echo "Ruby is installed."
else
    echo "Ruby is not installed. Attempting to install..."
    # Attempt to install the Ruby package
    if install_ruby_package; then
        echo "Ruby installed successfully."
    else
        echo "Failed to install Ruby."
        return 1
    fi
fi

PACKAGE_VERSION=$(ruby --version | awk '{print $2}')

# Check Ruby functionality by running a test script
if test_ruby_functionality; then
    echo "Ruby is functioning correctly."
    exit_status=0
else
    echo "Ruby is not functioning correctly."
    exit_status=1
fi

rm test_file.txt
return $exit_status
# End of the script