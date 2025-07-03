#!/bin/bash

# ASS LSP Server Installation Script
# Copyright (c) 2024 wiedymi
# MIT License

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default installation method
INSTALL_METHOD=""
INSTALL_DIR="$HOME/.local/bin"

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if Rust is installed
check_rust() {
    if ! command_exists cargo; then
        print_error "Rust and Cargo are required but not installed."
        print_info "Please install Rust from https://rustup.rs/"
        print_info "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    print_success "Rust and Cargo found"
}

# Install from crates.io
install_from_crates() {
    print_info "Installing ASS LSP from crates.io..."

    if cargo install ass-lsp; then
        print_success "ASS LSP installed successfully from crates.io"
        return 0
    else
        print_error "Failed to install from crates.io"
        return 1
    fi
}

# Install from source
install_from_source() {
    print_info "Installing ASS LSP from source..."

    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the ass-lsp directory."
        exit 1
    fi

    # Build the project
    print_info "Building ASS LSP..."
    if cargo build --release; then
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi

    # Create installation directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Copy binary to installation directory
    if cp target/release/ass-lsp "$INSTALL_DIR/"; then
        print_success "ASS LSP installed to $INSTALL_DIR"
    else
        print_error "Failed to copy binary to $INSTALL_DIR"
        exit 1
    fi
}

# Check if binary is in PATH
check_path() {
    if command_exists ass-lsp; then
        print_success "ASS LSP is available in PATH"
        print_info "Version: $(ass-lsp --version 2>/dev/null || echo 'Unknown')"
    else
        print_warning "ASS LSP is not in PATH"
        print_info "You may need to add $INSTALL_DIR to your PATH"
        print_info "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        print_info "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# Display usage
usage() {
    cat << EOF
ASS LSP Installation Script

Usage: $0 [OPTIONS]

Options:
    -m, --method METHOD     Installation method (crates|source)
    -d, --dir DIRECTORY     Installation directory (default: ~/.local/bin)
    -h, --help              Show this help message

Installation Methods:
    crates                  Install from crates.io (requires published package)
    source                  Install from source (requires local source code)

Examples:
    $0 -m crates            Install from crates.io
    $0 -m source            Install from source
    $0 -m source -d /usr/local/bin    Install from source to /usr/local/bin

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--method)
            INSTALL_METHOD="$2"
            shift 2
            ;;
        -d|--dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main installation logic
main() {
    print_info "ASS LSP Server Installation Script"
    print_info "================================="

    # Check prerequisites
    check_rust

    # Determine installation method if not specified
    if [ -z "$INSTALL_METHOD" ]; then
        if [ -f "Cargo.toml" ]; then
            print_info "Found Cargo.toml, defaulting to source installation"
            INSTALL_METHOD="source"
        else
            print_info "No Cargo.toml found, defaulting to crates.io installation"
            INSTALL_METHOD="crates"
        fi
    fi

    # Install based on method
    case "$INSTALL_METHOD" in
        crates)
            install_from_crates
            ;;
        source)
            install_from_source
            ;;
        *)
            print_error "Invalid installation method: $INSTALL_METHOD"
            print_info "Valid methods: crates, source"
            exit 1
            ;;
    esac

    # Check if installation was successful
    check_path

    print_success "Installation completed!"
    print_info ""
    print_info "Next steps:"
    print_info "1. Make sure $INSTALL_DIR is in your PATH"
    print_info "2. Configure your editor to use the ASS LSP server"
    print_info "3. Open an .ass or .ssa file to test the language server"
    print_info ""
    print_info "For editor configuration, see: https://github.com/wiedymi/ass-lsp#usage"
}

# Run main function
main "$@"
