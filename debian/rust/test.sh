#!/bin/bash

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

set -euo pipefail
original_dir=$(pwd)
log "Current directory: $original_dir"

temp_dir=$(mktemp -d)
log "Created temporary directory: $temp_dir"
cd "$temp_dir"

# Define the package details
PACKAGE_NAME="rust"

# Function to check if Rust is installed and properly configured
is_rust_installed() {
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        log "Rust is installed."
        
        # Check if a default toolchain is configured
        if rustc --version >/dev/null 2>&1 && cargo --version >/dev/null 2>&1; then
            log "Rust toolchain is properly configured."
            return 0
        else
            log "Rust is installed but no default toolchain is configured."
            return 1
        fi
    else
        log "Rust is not installed."
        return 1
    fi
}

# Function to install Rust
install_rust() {
    log "Attempting to install Rust..."
    if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
        log "Failed to install Rust."
        return 1
    fi
    
    # Load rustup into current shell
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Set default toolchain
    log "Setting up default toolchain..."
    if ! rustup default stable; then
        log "Failed to set default toolchain."
        return 1
    fi
    
    log "Rust installed successfully."
    return 0
}

# Function to configure Rust if already installed but not set up
configure_rust() {
    log "Configuring Rust toolchain..."
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if ! rustup default stable; then
        log "Failed to set default toolchain."
        return 1
    fi
    
    log "Rust toolchain configured successfully."
    return 0
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check for curl
    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is not installed. Please install curl and try again."
        return 1
    fi

    # Check for gcc
    if ! command -v gcc >/dev/null 2>&1; then
        echo "gcc is not installed. Please install build-essential and try again."
        return 1
    fi

    log "System prerequisites check passed."
}

# Function to test Rust functionality
test_rust_functionality() {
    local temp_dir
    temp_dir=$(mktemp -d) || { log "Failed to create temporary directory."; return 1; }
    log "Created temporary directory: $temp_dir"

    cd "$temp_dir"

    # Create a new Rust project
    log "Creating a new Rust project..."
    if ! cargo new hello_world; then
        log "Failed to create Rust project."
        cd "$original_dir"
        return 1
    fi

    cd hello_world

    # Build the project
    log "Building the Rust project..."
    if ! cargo build; then
        log "Failed to build Rust project."
        cd "$original_dir"
        return 1
    fi

    # Run the project
    log "Running the Rust project..."
    output=$(cargo run 2>&1) || {
        log "Failed to run Rust project. Output:"
        log "$output"
        cd "$original_dir"
        return 1
    }

    log "Project output: $output"

    # Clean up
    cd "$original_dir"
    rm -rf "$temp_dir"
    log "Cleaned up temporary directory."

    # Check if the output is as expected
    if [[ "$output" == *"Hello, world!"* ]]; then
        log "Rust test passed successfully."
        return 0
    else
        log "Unexpected output from Rust project."
        return 1
    fi
}

# Main script execution
main() {
    log "Starting Rust test script..."

    if ! check_prerequisites; then
        return 1
    fi

    # Check if Rust is installed and configured
    if ! is_rust_installed; then
        if command -v rustup >/dev/null 2>&1; then
            # Rust is partially installed (rustup exists) but needs configuration
            log "Rust is partially installed. Configuring..."
            if ! configure_rust; then
                if ! install_rust; then
                    return 1
                fi
            fi
        else
            # Rust is not installed at all
            if ! install_rust; then
                return 1
            fi
        fi
    fi

    # Get Rust and Cargo versions
    local rust_version=$(rustc --version 2>/dev/null | awk '{print $2}') || rust_version="Unknown"
    local cargo_version=$(cargo --version 2>/dev/null | awk '{print $2}') || cargo_version="Unknown"
    PACKAGE_VERSION="$rust_version ($cargo_version)"
    log "Using Rust version: $rust_version, Cargo version: $cargo_version"

    if test_rust_functionality; then
        log "Rust is functioning correctly."
        return 0
    else
        log "Rust is not functioning correctly."
        return 1
    fi
}

# Run the main function
main