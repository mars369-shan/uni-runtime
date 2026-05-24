#!/bin/bash

set -e

echo "=== Uni-Runtime Universal Installer ==="
echo ""

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "Linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macOS"
    elif [[ "$OSTYPE" == "cygwin" ]]; then
        echo "Windows (Cygwin)"
    elif [[ "$OSTYPE" == "msys" ]]; then
        echo "Windows (MSYS)"
    elif [[ "$OSTYPE" == "win32" ]]; then
        echo "Windows (Native)"
    else
        echo "Unknown"
    fi
}

OS=$(detect_os)

echo "Detected operating system: $OS"
echo ""

case $OS in
    "Linux")
        echo "Downloading Linux installer..."
        if command -v curl &> /dev/null; then
            curl -sSL https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.sh | bash
        elif command -v wget &> /dev/null; then
            wget -qO- https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.sh | bash
        else
            echo "Error: curl or wget required"
            exit 1
        fi
        ;;
    
    "macOS")
        echo "Downloading macOS installer..."
        curl -sSL https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.sh | bash
        ;;
    
    "Windows"*)
        echo "Windows system detected"
        echo ""
        echo "Please run the following command in PowerShell as Administrator:"
        echo ""
        echo "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.ps1'))"
        echo ""
        echo "Or download and run the installer manually:"
        echo "https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.ps1"
        ;;
    
    *)
        echo "Error: Unsupported operating system: $OS"
        exit 1
        ;;
esac

echo ""
echo "=== Installer script executed ==="
