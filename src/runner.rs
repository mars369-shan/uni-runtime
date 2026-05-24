use crate::config::{Config, Distro, Environment};
use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::io::Write;

fn which_binary(binary: &str) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let full_path = Path::new(dir).join(binary);
            if full_path.exists() && full_path.is_file() {
                return Some(full_path);
            }
        }
    }
    None
}

pub struct Runner;

impl Runner {
    pub fn create_env(name: &str, distro: Distro) -> Result<Environment> {
        println!("Creating environment: {} ({})", name, distro.as_str());
        let envs_dir = Config::envs_dir()?;
        let env_root = envs_dir.join(name);

        if env_root.exists() {
            return Err(anyhow::anyhow!("Environment '{}' already exists", name));
        }

        let result = match Self::create_env_from_docker(name, &distro) {
            Ok(_) => {
                println!("Environment created successfully!");
                Ok(())
            }
            Err(e) => {
                println!("Docker method failed: {}", e);
                println!("Trying to create lightweight environment with proot...");
                Self::create_env_with_proot(name, &distro)
            }
        };

        result?;

        let env = Environment {
            name: name.to_string(),
            distro: distro.clone(),
            created: chrono::Utc::now().to_rfc3339(),
            state: "stopped".to_string(),
            rootfs_path: env_root.to_string_lossy().to_string(),
            memory_limit: None,
            cpu_limit: None,
        };

        Ok(env)
    }

    fn create_env_from_docker(name: &str, distro: &Distro) -> Result<()> {
        let envs_dir = Config::envs_dir()?;
        let env_root = envs_dir.join(name);
        std::fs::create_dir_all(&env_root).context("Cannot create environment directory")?;

        let docker_image = distro.docker_image();
        let container_name = format!("uni-runtime-{}", name);
        println!("Extracting filesystem from Docker image: {}", docker_image);

        let pull_output = Command::new("docker")
            .args(["pull", docker_image])
            .output()
            .context("Failed to pull Docker image")?;
        if !pull_output.status.success() {
            return Err(anyhow::anyhow!("docker pull failed: {}", String::from_utf8_lossy(&pull_output.stderr)));
        }
        println!("Image pulled successfully");

        let create_output = Command::new("docker")
            .args(["create", "--name", &container_name, docker_image])
            .output()
            .context("Failed to create container")?;
        if !create_output.status.success() {
            return Err(anyhow::anyhow!("docker create failed: {}", String::from_utf8_lossy(&create_output.stderr)));
        }
        println!("Container created: {}", container_name);

        let source_path = format!("{}:/", container_name);
        let cp_output = Command::new("docker")
            .args(["cp", source_path.as_str(), env_root.to_str().unwrap()])
            .output()
            .context("Failed to copy from container")?;
        if !cp_output.status.success() {
            let _ = Command::new("docker").args(["rm", &container_name]).output();
            return Err(anyhow::anyhow!("docker cp failed: {}", String::from_utf8_lossy(&cp_output.stderr)));
        }
        println!("Filesystem copied successfully");

        let rm_output = Command::new("docker")
            .args(["rm", &container_name])
            .output()
            .context("Failed to remove container")?;
        if !rm_output.status.success() {
            println!("Warning: Failed to remove temporary container");
        }

        Self::setup_env(&env_root, distro)?;

        Ok(())
    }

    fn create_env_with_proot(name: &str, distro: &Distro) -> Result<()> {
        let envs_dir = Config::envs_dir()?;
        let env_root = envs_dir.join(name);
        std::fs::create_dir_all(&env_root).context("Cannot create environment directory")?;

        let _pkg_mgr = distro.pkg_manager();
        let os_release = match distro {
            Distro::Archlinux => "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\nVERSION_ID=\"rolling\"\n",
            Distro::Ubuntu2204 => "NAME=\"Ubuntu\"\nPRETTY_NAME=\"Ubuntu 22.04 LTS\"\nID=ubuntu\nVERSION_ID=\"22.04\"\n",
            Distro::Ubuntu2404 => "NAME=\"Ubuntu\"\nPRETTY_NAME=\"Ubuntu 24.04 LTS\"\nID=ubuntu\nVERSION_ID=\"24.04\"\n",
            Distro::Debian12 => "NAME=\"Debian GNU/Linux\"\nPRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"\nID=debian\nVERSION_ID=\"12\"\n",
            Distro::Fedora39 => "NAME=\"Fedora Linux\"\nPRETTY_NAME=\"Fedora Linux 39\"\nID=fedora\nVERSION_ID=\"39\"\n",
            Distro::Fedora40 => "NAME=\"Fedora Linux\"\nPRETTY_NAME=\"Fedora Linux 40\"\nID=fedora\nVERSION_ID=\"40\"\n",
            Distro::OpensuseTumbleweed => "NAME=\"openSUSE Tumbleweed\"\nPRETTY_NAME=\"openSUSE Tumbleweed\"\nID=opensuse-tumbleweed\n",
            Distro::OpensuseLeap => "NAME=\"openSUSE Leap\"\nPRETTY_NAME=\"openSUSE Leap\"\nID=opensuse-leap\n",
            Distro::Rhel9 => "NAME=\"Red Hat Enterprise Linux\"\nPRETTY_NAME=\"Red Hat Enterprise Linux 9\"\nID=rhel\nVERSION_ID=\"9\"\n",
        };

        let dirs = ["bin", "etc", "usr/bin", "usr/lib", "var", "home", "tmp", "proc", "sys", "dev"];
        for dir in dirs.iter() {
            std::fs::create_dir_all(env_root.join(dir))?;
        }

        if let Some(bash) = which_binary("bash") {
            std::fs::copy(&bash, env_root.join("bin/bash"))?;
            std::fs::set_permissions(env_root.join("bin/bash"), std::fs::Permissions::from_mode(0o755))?;
        } else if let Some(sh) = which_binary("sh") {
            std::fs::copy(&sh, env_root.join("bin/sh"))?;
            std::fs::set_permissions(env_root.join("bin/sh"), std::fs::Permissions::from_mode(0o755))?;
            std::os::unix::fs::symlink("sh", env_root.join("bin/bash")).ok();
        }

        std::fs::write(env_root.join("etc/hostname"), name)?;
        std::fs::write(env_root.join("etc/hosts"), format!("127.0.0.1 localhost\n127.0.0.1 {}\n", name))?;
        std::fs::write(env_root.join("etc/os-release"), os_release)?;

        Self::create_pkg_manager_stub(&env_root, distro)?;

        println!("Lightweight environment created successfully!");
        Ok(())
    }

    fn create_pkg_manager_stub(env_root: &Path, distro: &Distro) -> Result<()> {
        let pkg_mgr_script = match distro {
            Distro::Archlinux => r#"#!/bin/bash
set -e
if [ ! -f /var/lib/pacman/db.lck ]; then
    touch /var/lib/pacman/db.lck
fi
exec /usr/bin/pacman.real "$@"
"#,
            Distro::Ubuntu2204 | Distro::Ubuntu2404 | Distro::Debian12 => r#"#!/bin/bash
exec /usr/bin/apt.real "$@"
"#,
            Distro::Fedora39 | Distro::Fedora40 | Distro::Rhel9 => r#"#!/bin/bash
exec /usr/bin/dnf.real "$@"
"#,
            Distro::OpensuseTumbleweed | Distro::OpensuseLeap => r#"#!/bin/bash
exec /usr/bin/zypper.real "$@"
"#,
        };

        let pkg_mgr_path = env_root.join("usr/bin").join(distro.pkg_manager());
        std::fs::create_dir_all(pkg_mgr_path.parent().unwrap())?;
        std::fs::write(&pkg_mgr_path, pkg_mgr_script)?;
        std::fs::set_permissions(&pkg_mgr_path, std::fs::Permissions::from_mode(0o755))?;

        Ok(())
    }

    fn setup_env(env_root: &Path, distro: &Distro) -> Result<()> {
        match distro {
            Distro::Archlinux => Self::setup_archlinux(env_root),
            Distro::Ubuntu2204 | Distro::Ubuntu2404 => Self::setup_ubuntu(env_root),
            Distro::Debian12 => Self::setup_debian(env_root),
            Distro::Fedora39 | Distro::Fedora40 => Self::setup_fedora(env_root),
            _ => Ok(()),
        }
    }

    fn setup_archlinux(rootfs: &Path) -> Result<()> {
        let pacman_conf = rootfs.join("etc/pacman.conf");
        if pacman_conf.exists() {
            let mut content = std::fs::read_to_string(&pacman_conf)?;
            content = content.replace("#Color", "Color");
            content = content.replace("#ParallelDownloads = 5", "ParallelDownloads = 10");
            std::fs::write(&pacman_conf, content)?;
            println!("Configured pacman.conf");
        }

        let keyring_dir = rootfs.join("etc/pacman.d/gnupg");
        if !keyring_dir.exists() {
            std::fs::create_dir_all(&keyring_dir)?;
        }

        Ok(())
    }

    fn setup_ubuntu(rootfs: &Path) -> Result<()> {
        let apt_sources = rootfs.join("etc/apt/sources.list");
        if apt_sources.exists() {
            let content = std::fs::read_to_string(&apt_sources)?;
            if content.is_empty() {
                std::fs::write(&apt_sources, "deb http://archive.ubuntu.com/ubuntu/ jammy main restricted universe multiverse\ndeb http://archive.ubuntu.com/ubuntu/ jammy-updates main restricted universe multiverse\n")?;
            }
        }
        Ok(())
    }

    fn setup_debian(rootfs: &Path) -> Result<()> {
        let apt_sources = rootfs.join("etc/apt/sources.list");
        if apt_sources.exists() {
            let content = std::fs::read_to_string(&apt_sources)?;
            if content.is_empty() {
                std::fs::write(&apt_sources, "deb http://deb.debian.org/debian/ bookworm main\ndeb http://deb.debian.org/debian/ bookworm-updates main\n")?;
            }
        }
        Ok(())
    }

    fn setup_fedora(rootfs: &Path) -> Result<()> {
        let dnf_conf = rootfs.join("etc/dnf/dnf.conf");
        if dnf_conf.exists() {
            let mut content = std::fs::read_to_string(&dnf_conf)?;
            if !content.contains("fastestmirror") {
                content.push_str("\nfastestmirror=True\n");
                std::fs::write(&dnf_conf, content)?;
            }
        }
        Ok(())
    }

    pub fn exec(env: &Environment, command: &[String]) -> Result<()> {
        let env_root = PathBuf::from(&env.rootfs_path);

        if !env_root.exists() {
            return Err(anyhow::anyhow!("Environment directory does not exist: {}", env.rootfs_path));
        }

        if command.is_empty() {
            return Err(anyhow::anyhow!("Please specify a command to execute"));
        }

        let cmd_str = command.join(" ");
        let use_proot = which_binary("proot").is_some();

        let status = if use_proot {
            let env_root_str = env.rootfs_path.as_str();
            let full_cmd = if cmd_str.contains("sudo") {
                let path_env = format!("{}/usr/bin:{}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", env_root_str, env_root_str);
                format!("export PATH={}; {}", path_env, cmd_str)
            } else {
                cmd_str
            };
            
            Command::new("proot")
                .args([
                    "-0",
                    "-R", env.rootfs_path.as_str(),
                    "-b", "/etc/resolv.conf:/etc/resolv.conf",
                    "-b", "/proc:/proc",
                    "-b", "/sys:/sys",
                    "-b", "/dev:/dev",
                    "-w", "/root",
                    "--", "/bin/sh", "-c", &full_cmd
                ])
                .status()
                .context("Failed to execute command")?
        } else {
            let path_env = format!("PATH={}/usr/bin:{}/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin", env.rootfs_path, env.rootfs_path);
            
            Command::new("/bin/sh")
                .args(["-c", &cmd_str])
                .current_dir(&env_root)
                .env("PATH", path_env)
                .env("LD_LIBRARY_PATH", format!("{}/usr/lib:{}/lib", env.rootfs_path, env.rootfs_path))
                .status()
                .context("Failed to execute command")?
        };

        if !status.success() {
            return Err(anyhow::anyhow!("Command failed with exit code: {}", status.code().unwrap_or(-1)));
        }

        Ok(())
    }

    pub fn run(env: &Environment) -> Result<()> {
        let env_root = PathBuf::from(&env.rootfs_path);

        if !env_root.exists() {
            return Err(anyhow::anyhow!("Environment directory does not exist: {}", env.rootfs_path));
        }

        println!("Entering interactive shell for environment '{}'...", env.name);
        println!("Distribution: {}", env.distro.as_str());
        println!("Type 'exit' to quit");

        let use_proot = which_binary("proot").is_some();

        if use_proot {
            let status = Command::new("proot")
                .args([
                    "-R", env.rootfs_path.as_str(),
                    "-b", "/etc/resolv.conf:/etc/resolv.conf",
                    "-b", "/proc:/proc",
                    "-b", "/sys:/sys",
                    "-b", "/dev:/dev",
                    "--", "/bin/bash"
                ])
                .status()
                .context("Failed to start shell")?;

            if !status.success() {
                return Err(anyhow::anyhow!("Shell exited with code: {}", status.code().unwrap_or(-1)));
            }
        } else {
            Self::run_simple_shell(env)?;
        }

        Ok(())
    }

    fn run_simple_shell(env: &Environment) -> Result<()> {
        loop {
            print!("{}@uni-runtime:~$ ", env.name);
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "exit" {
                break;
            }

            if input.is_empty() {
                continue;
            }

            let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
            if let Err(e) = Self::exec(env, &args) {
                eprintln!("Error: {}", e);
            }
        }

        Ok(())
    }

    pub fn delete_env(env: &Environment) -> Result<()> {
        let env_root = PathBuf::from(&env.rootfs_path);
        if env_root.exists() {
            std::fs::remove_dir_all(&env_root).context("Failed to delete environment directory")?;
        }
        Ok(())
    }

    pub fn list_envs() -> Result<Vec<Environment>> {
        let config = Config::load()?;
        Ok(config.environments)
    }
}
