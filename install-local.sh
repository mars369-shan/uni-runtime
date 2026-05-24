#!/bin/bash

set -e

echo "=== Uni-Runtime Local Installer ==="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Clone or update the repo
REPO_DIR="$HOME/.uni-runtime"
if [ -d "$REPO_DIR" ]; then
    echo "Updating repository..."
    cd "$REPO_DIR" && git pull
else
    echo "Cloning repository..."
    git clone https://github.com/mars369-shan/uni-runtime.git "$REPO_DIR"
    cd "$REPO_DIR"
fi

# Build
echo "Building Uni-Runtime..."
cargo build --release

# Install
echo "Installing to $HOME/.local/bin/..."
mkdir -p "$HOME/.local/bin"
cp target/release/uni-runtime "$HOME/.local/bin/"

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    if [ -f "$HOME/.bashrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
        echo "Added to .bashrc, please run: source ~/.bashrc"
    elif [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
        echo "Added to .zshrc, please run: source ~/.zshrc"
    fi
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  uni-runtime --help"
echo "  uni-runtime test"
