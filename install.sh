#!/bin/bash

set -e

VERSION="0.1.0"
BINARY_NAME="uni-runtime"
TARGET_DIR="$HOME/.local/bin"

echo "=== Uni-Runtime Installer ==="
echo "Version: $VERSION"
echo ""

OS=$(uname -s)
ARCH=$(uname -m)

case $OS in
    Linux)
        OS="linux"
        ;;
    Darwin)
        OS="macos"
        ;;
    *)
        echo "Error: Unsupported OS: $OS"
        exit 1
        ;;
esac

case $ARCH in
    x86_64)
        ARCH="amd64"
        ;;
    aarch64|arm64)
        ARCH="arm64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "Detected system: $OS-$ARCH"

if [ "$OS" = "linux" ]; then
    echo "Checking dependencies..."
    
    if ! command -v docker &> /dev/null; then
        echo "Warning: Docker not installed, will use lightweight mode"
    fi
    
    if ! command -v proot &> /dev/null; then
        echo "Installing proot..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y proot
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y proot
        elif command -v pacman &> /dev/null; then
            sudo pacman -S --noconfirm proot
        elif command -v zypper &> /dev/null; then
            sudo zypper install -y proot
        else
            echo "Warning: Cannot install proot automatically, will use simple mode"
        fi
    fi
fi

if [ "$OS" = "darwin" ]; then
    echo "Checking dependencies..."
    
    if ! command -v brew &> /dev/null; then
        echo "Error: Homebrew not installed, please install Homebrew first"
        echo "Install: https://brew.sh/"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        echo "Warning: Docker not installed, will use lightweight mode"
    fi
    
    if ! command -v proot &> /dev/null; then
        echo "Installing proot..."
        brew install proot
    fi
fi

echo "Creating installation directory..."
mkdir -p "$TARGET_DIR"

echo "Downloading binary..."
DOWNLOAD_URL="https://github.com/mars369-shan/uni-runtime/releases/download/v${VERSION}/${BINARY_NAME}-${OS}-${ARCH}"

if command -v curl &> /dev/null; then
    curl -L "$DOWNLOAD_URL" -o "$TARGET_DIR/$BINARY_NAME"
elif command -v wget &> /dev/null; then
    wget "$DOWNLOAD_URL" -O "$TARGET_DIR/$BINARY_NAME"
else
    echo "Error: curl or wget not installed"
    exit 1
fi

echo "Setting execute permissions..."
chmod +x "$TARGET_DIR/$BINARY_NAME"

echo "Checking PATH..."
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    echo "Adding $TARGET_DIR to PATH..."
    if [ -f "$HOME/.bashrc" ]; then
        echo "export PATH=\"$TARGET_DIR:\$PATH\"" >> "$HOME/.bashrc"
        echo "Please run 'source ~/.bashrc' or restart terminal"
    elif [ -f "$HOME/.zshrc" ]; then
        echo "export PATH=\"$TARGET_DIR:\$PATH\"" >> "$HOME/.zshrc"
        echo "Please run 'source ~/.zshrc' or restart terminal"
    else
        echo "Warning: Cannot add to PATH automatically, please add $TARGET_DIR manually"
    fi
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Usage:"
echo "  uni-runtime create <env-name> --distro <distro>"
echo "  uni-runtime exec <env-name> -- <command>"
echo "  uni-runtime list"
echo ""
echo "Supported distributions:"
echo "  ubuntu-22.04, ubuntu-24.04, debian-12"
echo "  fedora-39, fedora-40, rhel-9"
echo "  archlinux, opensuse-tumbleweed, opensuse-leap"
