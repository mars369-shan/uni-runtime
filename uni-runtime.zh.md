# uni-runtime 技术方案文档

## 1. 项目概述

### 1.1 项目名称
`uni-runtime` —— 一种基于运行时隔离和自包含依赖的 Linux 跨发行版包管理器。

### 1.2 项目目标
构建一个**从底层解决 Linux 软件依赖与兼容性问题**的包管理系统。核心思想是**不依赖宿主系统的库**，而是将应用及其所有依赖打包成一个独立的、与系统隔离的运行时单元，从而实现“一次打包，到处运行”，彻底摆脱传统的“依赖地狱”。

### 1.3 设计原则
- **自包含**：应用所需的所有依赖（包括底层库如 glibc）都打包在内或明确声明依赖某个稳定的运行时。
- **沙箱隔离**：每个应用运行在独立的沙箱环境中，无法访问宿主系统或其他应用的敏感资源，提升安全性。
- **原子更新与回滚**：使用文件系统版本控制技术，支持完整的原子更新和即时回滚。
- **跨发行版**：不依赖任何发行版的包管理器和目录结构，可在任意 Linux 发行版上运行。
- **开发者友好**：提供简单的打包工具和清晰的 manifest 规范，方便将现有应用迁移到该格式。

---

## 2. 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        用户层                                │
│   CLI 客户端 (uni-runtime)                                   │
│   - uni create   创建新环境                                   │
│   - uni exec     在环境中执行命令                              │
│   - uni run      在环境中运行交互式shell                        │
│   - uni list     列出环境                                    │
│   - uni delete   删除环境                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      管理层 (守护进程)                        │
│   - 接收命令（create, exec, run, list, delete）               │
│   - 管理本地环境数据库                                         │
│   - 调用 OSTree 进行版本管理                                   │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
        ┌──────────┐                   ┌──────────┐
        │ 本地仓库  │                   │ OSTree   │
        │ (SQLite) │                   │ 存储层   │
        └──────────┘                   └──────────┘
                                            │
                                            ▼
                              ┌─────────────────────────┐
                              │   发行版镜像文件           │
                              │   (tarball)              │
                              └─────────────────────────┘
                                            │
                                            ▼
┌─────────────────────────────────────────────────────────────┐
│                       运行时层                                │
│   - systemd-nspawn / Bubblewrap 隔离                         │
│   - 挂载发行版根文件系统                                       │
│   - 配置 Namespaces / Cgroups / Seccomp                      │
│   - 网络配置 (NAT/Bridge)                                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                       内核层                                  │
│   Linux Kernel (Namespaces, Cgroups, AppArmor/SELinux)      │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 核心技术组件详解

### 3.1 多发行版环境模板

支持创建以下类型的运行环境，每个环境包含对应发行版的原生包管理器：

| 模板 ID | 发行版 | 包管理器 | 支持的包格式 | AUR 助手 |
|---------|--------|----------|--------------|----------|
| `ubuntu-22.04` | Ubuntu 22.04 | apt | .deb | - |
| `ubuntu-24.04` | Ubuntu 24.04 | apt | .deb | - |
| `debian-12` | Debian 12 | apt | .deb | - |
| `fedora-39` | Fedora 39 | dnf | .rpm | - |
| `fedora-40` | Fedora 40 | dnf | .rpm | - |
| `rhel-9` | RHEL 9 | dnf/yum | .rpm | - |
| `archlinux` | Arch Linux | pacman | .pkg.tar.zst | yay |
| `opensuse-tumbleweed` | openSUSE TW | zypper | .rpm | - |
| `opensuse-leap` | openSUSE Leap | zypper | .rpm | - |

### 3.2 包管理器集成

**apt (Debian/Ubuntu)**
```bash
uni exec <env> -- apt update
uni exec <env> -- apt install <package>
uni exec <env> -- apt install ./package.deb
```

**dnf/yum (Fedora/RHEL)**
```bash
uni exec <env> -- dnf install <package>
uni exec <env> -- dnf install ./package.rpm
```

**pacman (Arch Linux)**
```bash
uni exec <env> -- pacman -Syu <package>
```

**yay (Arch Linux AUR)**
```bash
uni exec <env> -- yay -S <aur-package>
```

**zypper (openSUSE)**
```bash
uni exec <env> -- zypper install <package>
```

### 3.3 隔离技术

使用 **systemd-nspawn** 或 **bubblewrap + proot** 作为底层隔离技术：

```bash
# systemd-nspawn 示例
systemd-nspawn --machine=<env-name> \
  --bind=/var/lib/uniruntime/envs/<env-name>/rootfs \
  --bind-ro=/etc/resolv.conf \
  --private-network \
  --network-veth \
  bash
```

**隔离策略**：
- **文件系统隔离**：每个环境拥有独立的根文件系统，与宿主系统隔离。
- **网络隔离**：支持 NAT、桥接、host-only 三种模式。
- **进程隔离**：独立 PID 命名空间。
- **用户/组隔离**：映射当前用户到环境内的用户。

### 3.4 存储与版本控制：OSTree

- **作用**：管理环境文件系统的多个版本，实现**原子更新**和**回滚**。
- **工作流程**：
  1. 创建环境时，从发行版镜像创建初始部署。
  2. 更新环境时，创建新的 OSTree commit。
  3. 支持快照功能，可随时恢复到之前的状态。

### 3.5 守护进程（Daemon）

- **名称**：`uniruntimed`
- **功能**：
  - 监听来自 CLI 的命令。
  - 管理本地环境数据库（SQLite）。
  - 创建和管理隔离环境。
  - 配置网络和资源限制。
- **启动方式**：systemd 用户服务（可选系统级服务）。

### 3.6 CLI 客户端

命令行接口设计：
```bash
# 环境管理命令
uni create <env-name> --distro <distro>   # 创建新环境
uni list                                  # 列出所有环境
uni delete <env-name>                     # 删除环境
uni start <env-name>                      # 启动环境
uni stop <env-name>                       # 停止环境
uni ps                                    # 查看运行中的环境

# 默认环境设置
uni set default <env-name>                # 设置默认环境
uni set default --create <distro>         # 创建并设置为默认环境（如 --ubuntu-22.04）
uni unset default                         # 取消默认环境设置

# 执行命令
uni exec <env-name> -- <command>          # 在环境中执行命令
uni exec -- <command>                     # 使用默认环境执行命令
uni run <env-name>                        # 在环境中运行交互式 shell
uni run                                   # 在默认环境中运行交互式 shell

# 快照管理
uni snapshot <env-name> --name <name>     # 创建快照
uni restore <env-name> <snapshot-name>    # 恢复快照

# 环境信息
uni info <env-name>                       # 查看环境详情
```

**默认环境使用示例**：
```bash
# 创建并设置默认环境
uni set default --ubuntu-22.04

# 查看当前默认环境
uni info default

# 使用默认环境执行命令（无需指定环境名称）
uni exec -- apt update && apt install nginx
uni run                                  # 进入默认环境的交互式 shell
```

---

## 4. 关键技术选型

| 组件 | 技术选择 | 理由 |
|------|----------|------|
| 编程语言 | Rust | 高性能、内存安全、无 GC，非常适合系统级工具。 |
| 包镜像格式 | SquashFS + zstd 压缩 | 高压缩比，随机读取快，内核原生支持。 |
| 版本控制 | libostree (C 库，通过 FFI 调用) | 成熟稳定，被 Flatpak 和 Fedora Silverblue 使用。 |
| 沙箱创建 | Bubblewrap | 轻量、安全、无 root 依赖（需 setuid 或用户命名空间）。 |
| 安全加固 | AppArmor / Seccomp | 提供细粒度的访问控制和系统调用过滤。 |
| 服务管理 | systemd --user | 统一管理用户态服务。 |
| 数据库 | SQLite (通过 rusqlite) | 轻量、无需单独服务、可靠。 |
| 序列化 | Serde + YAML/JSON | 处理 manifest 和索引文件。 |
| HTTP 客户端 | reqwest | 异步、支持 TLS。 |
| 异步运行时 | tokio | 高性能异步 I/O。 |
| 日志 | tracing | 结构化日志，便于调试。 |
| 错误处理 | anyhow + thiserror | 便捷且类型安全。 |

---

## 5. 开发路线图

### 阶段一：核心功能 – 4 周
**目标**：实现基础的多发行版环境创建和运行功能。

- [ ] 实现基础 CLI 框架（`create`, `exec`, `run`, `list`, `delete`）。
- [ ] 集成 systemd-nspawn，实现环境隔离运行。
- [ ] 支持 Ubuntu/Debian 环境创建，集成 apt 包管理器。
- [ ] 实现环境生命周期管理（创建、启动、停止、删除）。
- [ ] 测试：在任意 Linux 发行版上创建 Ubuntu 环境并安装 .deb 包。

### 阶段二：多发行版支持 – 4 周
**目标**：支持主流发行版的原生包管理器。

- [ ] 支持 Fedora/RHEL 环境（dnf/yum）。
- [ ] 支持 Arch Linux 环境（pacman）。
- [ ] 支持 openSUSE 环境（zypper）。
- [ ] 集成 yay AUR 助手（Arch Linux）。
- [ ] 跨发行版测试（Ubuntu, Fedora, Arch, openSUSE）。

### 阶段三：高级功能 – 4 周
**目标**：实现快照、网络配置和资源限制。

- [ ] 实现快照和恢复功能。
- [ ] 实现磁盘配额限制。
- [ ] 网络模式配置（NAT/桥接/host-only）。
- [ ] 资源限制（CPU/内存）。

### 阶段四：生态完善 – 4 周
**目标**：完善用户体验和扩展功能。

- [ ] 支持从 Docker 镜像导入环境。
- [ ] 环境导入/导出功能。
- [ ] 图形化前端支持。
- [ ] 与 Docker/Flatpak 协同支持。
- [ ] 完善错误提示和用户文档。

---

## 6. 安全设计

- **命名空间隔离**：每个环境拥有独立的 PID、Network、Mount、IPC 命名空间。
- **资源限制**：支持 CPU、内存、磁盘配额限制。
- **无特权模式**：优先使用用户命名空间，避免 setuid。
- **网络隔离**：可选的完全网络隔离模式（host-only）。
- **审计日志**：守护进程记录所有环境创建、删除和命令执行操作。

---

## 7. 总结与后续扩展

本技术方案提供了一个支持多发行版包管理器的隔离运行环境。它解决的核心问题是在单一 Linux 系统上使用不同发行版的原生包管理器安装和运行软件。通过 systemd-nspawn + OSTree 的组合，实现了**轻量隔离、多发行版支持、原子更新**的运行环境管理机制。

**可能的扩展方向**：
- 支持跨架构模拟（如通过 QEMU 在 arm64 上运行 amd64 环境）。
- 实现环境模板共享和社区贡献。
- 集成容器运行时（如 runc）提供更强的隔离能力。
- 开发图形化环境管理工具。

通过遵循上述路线图和规范，开发团队可以按照模块逐步实现，并在每个阶段进行集成测试，最终产出实用的多发行版包管理器运行环境。

---

**文档版本**：2.0  
**最后更新**：2025-01-XX  
**维护者**：Your Team  
**许可证**：MIT 或 Apache-2.0