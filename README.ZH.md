# Uni-Runtime

一个支持多发行版包管理器的隔离运行环境。

[![Build Status](https://img.shields.io/github/actions/workflow/status/mars369-shan/uni-runtime/rust.yml)](https://github.com/mars369-shan/uni-runtime/actions)
[![License](https://img.shields.io/github/license/mars369-shan/uni-runtime)](https://github.com/mars369-shan/uni-runtime/blob/main/LICENSE)
[![Version](https://img.shields.io/github/v/release/mars369-shan/uni-runtime)](https://github.com/mars369-shan/uni-runtime/releases)

## 功能特性

- ✅ 支持多个 Linux 发行版环境
- ✅ 原生包管理器支持（apt、dnf、pacman、zypper、yay）
- ✅ 轻量级容器隔离（proot）
- ✅ 默认环境配置
- ✅ 快照功能（备份/恢复）
- ✅ 跨平台支持（Linux、macOS、Windows）

## 支持的发行版

| 发行版 | 包管理器 | 标识符 |
|--------|----------|--------|
| Ubuntu 22.04 | apt | ubuntu-22.04 |
| Ubuntu 24.04 | apt | ubuntu-24.04 |
| Debian 12 | apt | debian-12 |
| Fedora 39 | dnf | fedora-39 |
| Fedora 40 | dnf | fedora-40 |
| Arch Linux | pacman | archlinux |
| openSUSE Tumbleweed | zypper | opensuse-tumbleweed |
| openSUSE Leap | zypper | opensuse-leap |

## 安装

### Linux/macOS

```bash
curl -sSL https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.sh | bash
```

### Windows

在 PowerShell 中以管理员身份运行：

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/mars369-shan/uni-runtime/main/install.ps1'))
```

### 从源代码构建

```bash
git clone https://github.com/mars369-shan/uni-runtime.git
cd uni-runtime
cargo build --release
```

## 使用示例

### 创建环境

```bash
# 创建 Ubuntu 环境
uni-runtime create my-ubuntu --distro ubuntu-22.04

# 创建 Arch Linux 环境
uni-runtime create my-arch --distro archlinux
```

### 使用包管理器

```bash
# Ubuntu/Debian
uni-runtime exec my-ubuntu -- apt update
uni-runtime exec my-ubuntu -- apt install nginx

# Arch Linux
uni-runtime exec my-arch -- pacman -Syu
uni-runtime exec my-arch -- yay -S google-chrome

# Fedora
uni-runtime exec my-fedora -- dnf install nginx
```

### 设置默认环境

```bash
# 设置现有环境为默认
uni-runtime set my-ubuntu

# 使用默认环境（无需指定环境名）
uni-runtime exec -- apt update
uni-runtime run
```

### 命令列表

```bash
uni-runtime create <name> --distro <distro>  # 创建环境
uni-runtime list                              # 列出环境
uni-runtime delete <name>                     # 删除环境
uni-runtime exec <name> -- <command>          # 执行命令
uni-runtime run <name>                        # 启动交互式 shell
uni-runtime set <name>                        # 设置默认环境
uni-runtime unset                             # 取消默认环境
uni-runtime info <name>                       # 查看环境信息
uni-runtime snapshot <name> --name <snapshot> # 创建快照
uni-runtime restore <name> <snapshot>        # 恢复快照
```

## 项目结构

```
uni-runtime/
├── src/
│   ├── main.rs          # 主程序入口
│   ├── cli.rs           # 命令行接口
│   ├── config.rs        # 配置管理
│   └── runner.rs        # 运行时管理
├── install.sh           # Linux/macOS 安装脚本
├── install.ps1          # Windows 安装脚本
├── install-all.sh       # 统一安装器
├── Cargo.toml           # Rust 依赖配置
├── LICENSE              # MIT 许可证
├── README.md            # 项目说明
└── CONTRIBUTING.md      # 贡献指南
```

## 开发

### 前置条件

- Rust 1.70+
- Docker（可选）
- proot（可选）

### 构建

```bash
cargo build --release
cargo test
```

## 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 贡献

欢迎贡献代码！详见 [CONTRIBUTING.md](CONTRIBUTING.md)
