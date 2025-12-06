#!/bin/sh
# Duperr installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/thatjuan/duperr/main/install.sh | sh

set -e

REPO="thatjuan/duperr"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64) PLATFORM="linux-x86_64" ;;
                aarch64|arm64) PLATFORM="linux-aarch64" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                x86_64) PLATFORM="macos-x86_64" ;;
                arm64) PLATFORM="macos-aarch64" ;;
                *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $OS"
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | \
        grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    echo "Installing duperr..."

    detect_platform
    VERSION="${VERSION:-$(get_latest_version)}"

    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi

    echo "Platform: $PLATFORM"
    echo "Version: $VERSION"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download and extract
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/duperr-$PLATFORM.tar.gz"
    echo "Downloading from: $DOWNLOAD_URL"

    curl -fsSL "$DOWNLOAD_URL" | tar -xz -C "$INSTALL_DIR"
    chmod +x "$INSTALL_DIR/duperr"

    echo ""
    echo "✓ duperr installed to $INSTALL_DIR/duperr"
    echo ""

    # Check if in PATH
    if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
        echo "Add $INSTALL_DIR to your PATH:"
        echo ""
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    echo "Run 'duperr --help' to get started"
}

main
