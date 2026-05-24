# Uni-Runtime

An isolated runtime environment supporting multiple Linux distribution package managers.

[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/uni-runtime/rust.yml)](https://github.com/yourusername/uni-runtime/actions)
[![License](https://img.shields.io/github/license/yourusername/uni-runtime)](https://github.com/yourusername/uni-runtime/blob/main/LICENSE)
[![Version](https://img.shields.io/github/v/release/yourusername/uni-runtime)](https://github.com/yourusername/uni-runtime/releases)

## Features

- ✅ Support multiple Linux distribution environments
- ✅ Native package manager support (apt, dnf, pacman, zypper, yay)
- ✅ Lightweight container isolation (proot)
- ✅ Default environment configuration
- ✅ Snapshot functionality (backup/restore)
- ✅ Cross-platform support (Linux, macOS, Windows)

## Supported Distributions

| Distribution | Package Manager | Identifier |
|--------------|-----------------|------------|
| Ubuntu 22.04 | apt | ubuntu-22.04 |
| Ubuntu 24.04 | apt | ubuntu-24.04 |
| Debian 12 | apt | debian-12 |
| Fedora 39 | dnf | fedora-39 |
| Fedora 40 | dnf | fedora-40 |
| Arch Linux | pacman | archlinux |
| openSUSE Tumbleweed | zypper | opensuse-tumbleweed |
| openSUSE Leap | zypper | opensuse-leap |

## Installation

### Linux/macOS

```bash
curl -sSL https://raw.githubusercontent.com/yourusername/uni-runtime/main/install.sh | bash
```

### Windows

Run in PowerShell as Administrator:

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/yourusername/uni-runtime/main/install.ps1'))
```

### Build from Source

```bash
git clone https://github.com/yourusername/uni-runtime.git
cd uni-runtime
cargo build --release
```

## Usage Examples

### Create Environment

```bash
# Create Ubuntu environment
uni-runtime create my-ubuntu --distro ubuntu-22.04

# Create Arch Linux environment
uni-runtime create my-arch --distro archlinux
```

### Use Package Managers

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

### Set Default Environment

```bash
# Set existing environment as default
uni-runtime set my-ubuntu

# Use default environment (no need to specify environment name)
uni-runtime exec -- apt update
uni-runtime run
```

### Command List

```bash
uni-runtime create <name> --distro <distro>  # Create environment
uni-runtime list                              # List environments
uni-runtime delete <name>                     # Delete environment
uni-runtime exec <name> -- <command>          # Execute command
uni-runtime run <name>                        # Start interactive shell
uni-runtime set <name>                        # Set default environment
uni-runtime unset                             # Unset default environment
uni-runtime info <name>                       # Show environment info
uni-runtime snapshot <name> --name <snapshot> # Create snapshot
uni-runtime restore <name> <snapshot>        # Restore snapshot
```

## Project Structure

```
uni-runtime/
├── src/
│   ├── main.rs          # Main entry point
│   ├── cli.rs           # Command line interface
│   ├── config.rs        # Configuration management
│   └── runner.rs        # Runtime management
├── install.sh           # Linux/macOS installation script
├── install.ps1          # Windows installation script
├── install-all.sh       # Universal installer
├── Cargo.toml           # Rust dependencies
├── LICENSE              # MIT License
├── README.md            # Project documentation
└── CONTRIBUTING.md      # Contribution guide
```

## Development

### Prerequisites

- Rust 1.70+
- Docker (optional)
- proot (optional)

### Build

```bash
cargo build --release
cargo test
```

## License

MIT License - See [LICENSE](LICENSE) for details

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)
