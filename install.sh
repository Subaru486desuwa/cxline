#!/bin/bash
set -e

echo "==> Installing cxline..."

# Check Rust toolchain
if ! command -v cargo &>/dev/null; then
    echo "Error: Rust not installed. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check tmux, install if missing (macOS)
if ! command -v tmux &>/dev/null; then
    echo "==> tmux not found, installing..."
    if command -v brew &>/dev/null; then
        brew install tmux
    elif command -v apt-get &>/dev/null; then
        sudo apt-get install -y tmux
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y tmux
    else
        echo "Error: Cannot auto-install tmux. Please install it manually."
        exit 1
    fi
fi

# Build and install cxline
echo "==> Building cxline (release)..."
cargo install --path . --force

# Run setup (writes tmux.conf + shell alias)
echo "==> Configuring..."
cxline setup

echo ""
echo "Done! Open a new terminal and type 'cx' to start."
