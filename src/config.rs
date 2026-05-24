use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Distro {
    #[serde(rename = "ubuntu-22.04")]
    Ubuntu2204,
    #[serde(rename = "ubuntu-24.04")]
    Ubuntu2404,
    #[serde(rename = "debian-12")]
    Debian12,
    #[serde(rename = "fedora-39")]
    Fedora39,
    #[serde(rename = "fedora-40")]
    Fedora40,
    #[serde(rename = "rhel-9")]
    Rhel9,
    #[serde(rename = "archlinux")]
    Archlinux,
    #[serde(rename = "opensuse-tumbleweed")]
    OpensuseTumbleweed,
    #[serde(rename = "opensuse-leap")]
    OpensuseLeap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub distro: Distro,
    pub created: String,
    pub state: String,
    pub rootfs_path: String,
    pub memory_limit: Option<String>,
    pub cpu_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub environments: Vec<Environment>,
    pub default_env: Option<String>,
}

impl Config {
    pub fn default() -> Self {
        Self {
            environments: Vec::new(),
            default_env: None,
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        let path = PathBuf::from("/tmp/uni-runtime/config.json");
        Ok(path)
    }

    pub fn envs_dir() -> Result<PathBuf> {
        let dir = PathBuf::from("/tmp/uni-runtime/envs");
        Ok(dir)
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let mut file = File::open(&path).context("Cannot open config file")?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .context("Cannot read config file")?;
        
        let result: Result<Self, serde_json::Error> = serde_json::from_str(&content);
        
        match result {
            Ok(config) => Ok(config),
            Err(e) => {
                let json: serde_json::Value = serde_json::from_str(&content)?;
                let mut config = Self::default();
                
                if let Some(envs) = json.get("environments").and_then(|e| e.as_array()) {
                    for env in envs {
                        let name = env.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
                        let distro_str = env.get("distro").and_then(|d| d.as_str()).unwrap_or_default();
                        let created = env.get("created").and_then(|c| c.as_str()).unwrap_or_default().to_string();
                        let state = env.get("state").and_then(|s| s.as_str()).unwrap_or("stopped").to_string();
                        
                        if let Some(distro) = Distro::from_str(distro_str) {
                            let envs_dir = Self::envs_dir()?;
                            let rootfs_path = envs_dir.join(&name).to_string_lossy().to_string();
                            
                            config.environments.push(Environment {
                                name,
                                distro,
                                created,
                                state,
                                rootfs_path,
                                memory_limit: None,
                                cpu_limit: None,
                            });
                        }
                    }
                }
                
                if let Some(default) = json.get("default_env").and_then(|d| d.as_str()) {
                    config.default_env = Some(default.to_string());
                }
                
                Ok(config)
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Cannot create config directory")?;
        }
        let mut file = File::create(&path).context("Cannot create config file")?;
        let content = serde_json::to_string_pretty(self).context("Cannot serialize config")?;
        file.write_all(content.as_bytes())
            .context("Cannot write config file")?;
        Ok(())
    }

    pub fn get_env(&self, name: &str) -> Option<&Environment> {
        self.environments.iter().find(|e| e.name == name)
    }

    pub fn get_env_mut(&mut self, name: &str) -> Option<&mut Environment> {
        self.environments.iter_mut().find(|e| e.name == name)
    }

    pub fn add_env(&mut self, env: Environment) {
        self.environments.push(env);
    }

    pub fn remove_env(&mut self, name: &str) {
        self.environments.retain(|e| e.name != name);
        if self.default_env.as_deref() == Some(name) {
            self.default_env = None;
        }
    }

    pub fn get_default_env(&self) -> Option<&Environment> {
        self.default_env
            .as_ref()
            .and_then(|name| self.get_env(name))
    }
}

impl Distro {
    pub fn as_str(&self) -> &'static str {
        match self {
            Distro::Ubuntu2204 => "ubuntu-22.04",
            Distro::Ubuntu2404 => "ubuntu-24.04",
            Distro::Debian12 => "debian-12",
            Distro::Fedora39 => "fedora-39",
            Distro::Fedora40 => "fedora-40",
            Distro::Rhel9 => "rhel-9",
            Distro::Archlinux => "archlinux",
            Distro::OpensuseTumbleweed => "opensuse-tumbleweed",
            Distro::OpensuseLeap => "opensuse-leap",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ubuntu-22.04" => Some(Distro::Ubuntu2204),
            "ubuntu-24.04" => Some(Distro::Ubuntu2404),
            "debian-12" => Some(Distro::Debian12),
            "fedora-39" => Some(Distro::Fedora39),
            "fedora-40" => Some(Distro::Fedora40),
            "rhel-9" => Some(Distro::Rhel9),
            "archlinux" => Some(Distro::Archlinux),
            "opensuse-tumbleweed" => Some(Distro::OpensuseTumbleweed),
            "opensuse-leap" => Some(Distro::OpensuseLeap),
            _ => None,
        }
    }

    pub fn docker_image(&self) -> &'static str {
        match self {
            Distro::Ubuntu2204 => "ubuntu:22.04",
            Distro::Ubuntu2404 => "ubuntu:24.04",
            Distro::Debian12 => "debian:12",
            Distro::Fedora39 => "fedora:39",
            Distro::Fedora40 => "fedora:40",
            Distro::Rhel9 => "registry.access.redhat.com/ubi9:latest",
            Distro::Archlinux => "archlinux:latest",
            Distro::OpensuseTumbleweed => "opensuse/tumbleweed:latest",
            Distro::OpensuseLeap => "opensuse/leap:latest",
        }
    }

    pub fn pkg_manager(&self) -> &'static str {
        match self {
            Distro::Ubuntu2204 | Distro::Ubuntu2404 | Distro::Debian12 => "apt",
            Distro::Fedora39 | Distro::Fedora40 | Distro::Rhel9 => "dnf",
            Distro::Archlinux => "pacman",
            Distro::OpensuseTumbleweed | Distro::OpensuseLeap => "zypper",
        }
    }

    pub fn release_name(&self) -> &'static str {
        match self {
            Distro::Ubuntu2204 => "jammy",
            Distro::Ubuntu2404 => "noble",
            Distro::Debian12 => "bookworm",
            Distro::Fedora39 => "39",
            Distro::Fedora40 => "40",
            Distro::Rhel9 => "9",
            Distro::Archlinux => "rolling",
            Distro::OpensuseTumbleweed => "tumbleweed",
            Distro::OpensuseLeap => "leap",
        }
    }
}
