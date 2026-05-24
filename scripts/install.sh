#!/bin/bash

set -e

INSTALL_DIR="${HOME}/.local/bin"
REPO_URL="https://github.com/yourusername/uni-runtime"
VERSION="${VERSION:-latest}"

echo "=== Uni-Runtime 一键安装脚本 ==="
echo "安装目录: ${INSTALL_DIR}"
echo

if [ "$(uname)" = "Darwin" ]; then
    PLATFORM="macos"
elif [ "$(uname)" = "Linux" ]; then
    PLATFORM="linux"
else
    echo "错误: 不支持的平台"
    exit 1
fi

ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64)  ARCH_NAME="amd64" ;;
    aarch64|arm64)  ARCH_NAME="arm64" ;;
    *)  echo "错误: 不支持的架构 ${ARCH}"; exit 1 ;;
esac

echo "检测到平台: ${PLATFORM} (${ARCH})"
echo

if [ ! -d "${INSTALL_DIR}" ]; then
    echo "创建安装目录: ${INSTALL_DIR}"
    mkdir -p "${INSTALL_DIR}"
fi

if command -v docker &> /dev/null; then
    echo "检测到 Docker，将使用 Docker 构建..."
    cd /tmp
    rm -rf uni-runtime-build
    git clone --depth 1 "${REPO_URL}" uni-runtime-build
    cd uni-runtime-build
    docker build -t uni-runtime-builder .
    docker create --name uni-runtime-builder uni-runtime-builder
    docker cp uni-runtime-builder:/app/target/release/uni-runtime "${INSTALL_DIR}/uni-runtime"
    docker rm uni-runtime-builder
    echo "✓ Docker 构建完成"
else
    echo "检测到 Rust 工具链，将从源码编译..."
    if ! command -v cargo &> /dev/null; then
        echo "错误: 未检测到 cargo，请先安装 Rust"
        echo "安装命令: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    cd /tmp
    rm -rf uni-runtime-build
    git clone --depth 1 "${REPO_URL}" uni-runtime-build
    cd uni-runtime-build
    cargo build --release
    cp target/release/uni-runtime "${INSTALL_DIR}/uni-runtime"
    echo "✓ 编译完成"
fi

chmod +x "${INSTALL_DIR}/uni-runtime"

CONFIG_DIR="${HOME}/.config/uni-runtime"
DATA_DIR="${HOME}/.local/share/uni-runtime"
mkdir -p "${CONFIG_DIR}" "${DATA_DIR}"

if [ -f "${INSTALL_DIR}/uni-runtime" ]; then
    echo
    echo "=== 安装完成 ==="
    echo
    echo "使用方法:"
    echo "  ${INSTALL_DIR}/uni-runtime --help"
    echo
    echo "建议将以下内容添加到您的 shell 配置文件 (~/.bashrc 或 ~/.zshrc):"
    echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
    echo
    echo "或者直接运行:"
    echo "  ${INSTALL_DIR}/uni-runtime list"
else
    echo "错误: 安装失败"
    exit 1
fi
