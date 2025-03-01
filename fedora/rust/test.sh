#!/bin/bash

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

error_exit() {
    log "ERROR: $1"
    exit 1
}

set -euo pipefail
original_dir=$(pwd)
log "Current directory: $original_dir"

temp_dir=$(mktemp -d)
log "Created temporary directory: $temp_dir"
cd "$temp_dir"

# Define the package details
PACKAGE_NAME="rust"

# Function to check if Rust is installed
is_rust_installed() {
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        log "Rust is installed."
        return 0
    else
        log "Rust is not installed."
        return 1
    fi
}

# Function to install Rust
install_rust() {
    log "Attempting to install Rust..."
    
    # Explicitly request the stable toolchain for RISC-V
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --target riscv64gc-unknown-linux-gnu
    
    # Source the environment
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
        log "Cargo environment sourced from $HOME/.cargo/env"
    else
        log "WARNING: $HOME/.cargo/env not found after installation"
    fi
    
    # Add to PATH for this session if not already there
    if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        log "Added $HOME/.cargo/bin to PATH for this session"
    fi
    
    # Verify installation and toolchain
    if command -v rustc >/dev/null 2>&1 && command -v cargo >/dev/null 2>&1; then
        log "Rust installed successfully."
        log "rustc version: $(rustc --version)"
        log "cargo version: $(cargo --version)"
        
        # Check if RISC-V target is installed
        if rustup target list --installed | grep -q "riscv64gc-unknown-linux-gnu"; then
            log "RISC-V target is properly installed"
        else
            log "Installing RISC-V target explicitly..."
            rustup target add riscv64gc-unknown-linux-gnu
        fi
        
        return 0
    else
        error_exit "Rust installation failed. rustc or cargo not found in PATH."
        return 1
    fi
}

# Function to check system prerequisites
check_prerequisites() {
    log "Checking system prerequisites..."

    # Check for curl
    if ! command -v curl >/dev/null 2>&1; then
        if command -v dnf >/dev/null 2>&1; then
            log "Installing curl via dnf..."
            sudo dnf install -y curl
        else
            error_exit "curl is not installed and couldn't be installed automatically."
        fi
    fi

    # Check for gcc (or a compatible compiler)
    if ! command -v gcc >/dev/null 2>&1; then
        if command -v dnf >/dev/null 2>&1; then
            log "Installing gcc and development tools via dnf..."
            sudo dnf install -y gcc make
        else
            error_exit "gcc is not installed and couldn't be installed automatically."
        fi
    fi

    log "System prerequisites check passed."
}

# Function to test Rust functionality
test_rust_functionality() {
    local test_dir
    test_dir=$(mktemp -d) || error_exit "Failed to create temporary directory."
    log "Created test directory: $test_dir"

    cd "$test_dir"

    # Create a new Rust project
    log "Creating a new Rust project..."
    if ! cargo new hello_world; then
        log "Failed to create Rust project. Cargo output:"
        cargo new hello_world --verbose
        rm -rf "$test_dir"
        return 1
    fi

    cd hello_world

    # Build the project
    log "Building the Rust project..."
    if ! cargo build; then
        log "Failed to build Rust project."
        cd "$original_dir"
        rm -rf "$test_dir"
        return 1
    fi

    # Run the project
    log "Running the Rust project..."
    output=$(cargo run 2>&1) || {
        log "Failed to run Rust project. Output:"
        echo "$output"
        cd "$original_dir"
        rm -rf "$test_dir"
        return 1
    }

    log "Project output: $output"

    # Clean up
    cd "$original_dir"
    rm -rf "$test_dir"
    log "Cleaned up test directory."

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

    check_prerequisites

    if ! is_rust_installed; then
        install_rust
    fi

    # Make sure PATH includes cargo
    if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
        log "Added $HOME/.cargo/bin to PATH"
    fi

    # Double-check Rust is accessible
    if ! command -v rustc >/dev/null 2>&1; then
        error_exit "rustc command not found after installation"
    fi
    
    if ! command -v cargo >/dev/null 2>&1; then
        error_exit "cargo command not found after installation"
    fi

    local rust_version=$(rustc --version | awk '{print $2}') || rust_version="Unknown"
    local cargo_version=$(cargo --version | awk '{print $2}') || cargo_version="Unknown"
    PACKAGE_VERSION="$rust_version ($cargo_version)"
    log "Rust version: $PACKAGE_VERSION"

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