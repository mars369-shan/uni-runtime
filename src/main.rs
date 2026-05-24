mod cli;
mod config;
mod runner;

use clap::Parser;
use cli::{Cli, Commands};
use config::{Config, Distro, Environment};
use runner::Runner;
use anyhow::{Context, Result};


fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load().context("Failed to load config")?;

    match cli.command {
        Commands::Create { name, distro } => cmd_create(&mut config, &name, &distro)?,
        Commands::List => cmd_list(&config)?,
        Commands::Delete { name } => cmd_delete(&mut config, &name)?,
        Commands::Start { name } => cmd_start(&config, &name)?,
        Commands::Stop { name } => cmd_stop(&config, &name)?,
        Commands::Ps => cmd_ps(&config)?,
        Commands::SetDefault { name, create } => cmd_set_default(&mut config, name, create)?,
        Commands::UnsetDefault => cmd_unset_default(&mut config)?,
        Commands::Exec { name, command } => cmd_exec(&config, name, command)?,
        Commands::Run { name } => cmd_run(&config, name)?,
        Commands::Snapshot { name, snapshot_name } => cmd_snapshot(&config, &name, &snapshot_name)?,
        Commands::Restore { name, snapshot_name } => cmd_restore(&config, &name, &snapshot_name)?,
        Commands::Info { name } => cmd_info(&config, &name)?,
        Commands::Test => cmd_test()?,
    }

    Ok(())
}

fn cmd_create(config: &mut Config, name: &str, distro_str: &str) -> Result<()> {
    let distro = Distro::from_str(distro_str).ok_or_else(|| anyhow::anyhow!("Unsupported distribution: {}", distro_str))?;
    if config.get_env(name).is_some() {
        return Err(anyhow::anyhow!("Environment '{}' already exists", name));
    }

    let env = Runner::create_env(name, distro)?;
    config.add_env(env);
    config.save()?;

    println!("Environment '{}' created successfully!", name);
    Ok(())
}

fn cmd_list(config: &Config) -> Result<()> {
    println!("Environment list:");
    if config.environments.is_empty() {
        println!("(No environments created)");
        return Ok(());
    }
    for env in &config.environments {
        let marker = if config.default_env.as_deref() == Some(&env.name) {
            " *"
        } else {
            ""
        };
        println!("  {}{}", env.name, marker);
        println!("    Distribution: {}", env.distro.as_str());
        println!("    Created: {}", env.created);
        println!("    State: {}", env.state);
        println!("    Path: {}", env.rootfs_path);
    }
    Ok(())
}

fn cmd_delete(config: &mut Config, name: &str) -> Result<()> {
    let env = config.get_env(name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", name))?;
    
    Runner::delete_env(env)?;
    config.remove_env(name);
    config.save()?;
    
    println!("Environment '{}' deleted successfully!", name);
    Ok(())
}

fn cmd_start(config: &Config, name: &str) -> Result<()> {
    let env = config.get_env(name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", name))?;
    println!("Starting environment '{}'...", name);
    println!("Root directory: {}", env.rootfs_path);
    Ok(())
}

fn cmd_stop(config: &Config, name: &str) -> Result<()> {
    let _env = config.get_env(name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", name))?;
    println!("Stopping environment '{}'...", name);
    Ok(())
}

fn cmd_ps(_config: &Config) -> Result<()> {
    println!("Running environments:");
    Ok(())
}

fn cmd_set_default(config: &mut Config, name: Option<String>, create: Option<String>) -> Result<()> {
    if let Some(distro_str) = create {
        let distro = Distro::from_str(&distro_str).ok_or_else(|| anyhow::anyhow!("Unsupported distribution: {}", distro_str))?;
        let default_name = "default-".to_string() + &distro_str;
        if config.get_env(&default_name).is_none() {
            let env = Runner::create_env(&default_name, distro)?;
            config.add_env(env);
        }
        config.default_env = Some(default_name.clone());
        config.save()?;
        println!("Created and set default environment: {}", default_name);
    } else if let Some(target) = name {
        if config.get_env(&target).is_none() {
            return Err(anyhow::anyhow!("Environment '{}' not found", target));
        }
        config.default_env = Some(target.clone());
        config.save()?;
        println!("Set default environment: {}", target);
    } else {
        return Err(anyhow::anyhow!("Please specify environment name or use --create option"));
    }
    Ok(())
}

fn cmd_unset_default(config: &mut Config) -> Result<()> {
    if config.default_env.is_none() {
        println!("No default environment set");
        return Ok(());
    }
    config.default_env = None;
    config.save()?;
    println!("Default environment unset");
    Ok(())
}

fn get_target_env(config: &Config, name: Option<String>) -> Result<&Environment> {
    match name {
        Some(n) => config.get_env(&n).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", n)),
        None => config.get_default_env().ok_or_else(|| anyhow::anyhow!("No default environment set, please specify environment name or set a default first")),
    }
}

fn cmd_exec(config: &Config, name: Option<String>, command: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err(anyhow::anyhow!("Please specify command (use -- to separate)"));
    }
    let env = get_target_env(config, name)?;
    Runner::exec(env, &command)?;
    Ok(())
}

fn cmd_run(config: &Config, name: Option<String>) -> Result<()> {
    let env = get_target_env(config, name)?;
    Runner::run(env)?;
    Ok(())
}

fn cmd_snapshot(config: &Config, name: &str, snapshot_name: &str) -> Result<()> {
    let env = config.get_env(name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", name))?;
    println!("Creating snapshot '{}' for environment '{}'...", snapshot_name, name);
    println!("Source directory: {}", env.rootfs_path);
    Ok(())
}

fn cmd_restore(config: &Config, name: &str, snapshot_name: &str) -> Result<()> {
    let env = config.get_env(name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", name))?;
    println!("Restoring snapshot '{}' to environment '{}'...", snapshot_name, name);
    println!("Target directory: {}", env.rootfs_path);
    Ok(())
}

fn cmd_info(config: &Config, name: &str) -> Result<()> {
    let env_name = if name == "default" {
        match config.get_default_env() {
            Some(env) => &env.name,
            None => {
                println!("No default environment set");
                return Ok(());
            }
        }
    } else {
        name
    };

    let env = config.get_env(env_name).ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", env_name))?;
    println!("Environment info:");
    println!("  Name: {}", env.name);
    println!("  Distribution: {}", env.distro.as_str());
    println!("  Package Manager: {}", env.distro.pkg_manager());
    println!("  Created: {}", env.created);
    println!("  State: {}", env.state);
    println!("  Root path: {}", env.rootfs_path);
    if let Some(mem) = &env.memory_limit {
        println!("  Memory limit: {}", mem);
    }
    if let Some(cpu) = env.cpu_limit {
        println!("  CPU limit: {} cores", cpu);
    }
    if config.default_env.as_deref() == Some(&env.name) {
        println!("  (Current default environment)");
    }
    Ok(())
}

fn cmd_test() -> Result<()> {
    println!("=== Uni-Runtime Feature Test ===");
    println!();

    let test_env_name = "test-env";
    
    if let Ok(config) = Config::load() {
        if config.get_env(test_env_name).is_some() {
            println!("Test environment already exists, skipping creation");
        } else {
            println!("1. Testing environment creation...");
            let mut config_mut = Config::load()?;
            let env = Runner::create_env(test_env_name, Distro::Ubuntu2204)?;
            config_mut.add_env(env);
            config_mut.save()?;
            println!("   ✓ Environment created successfully");
        }
    }

    println!("2. Testing environment list...");
    let config = Config::load()?;
    if !config.environments.is_empty() {
        println!("   ✓ Environment list works correctly");
        for env in &config.environments {
            println!("     - {} ({})", env.name, env.distro.as_str());
        }
    } else {
        println!("   ✗ Environment list is empty");
    }

    println!("3. Testing environment info...");
    if let Some(env) = config.get_env(test_env_name) {
        println!("   ✓ Environment info retrieved successfully");
        println!("     Name: {}", env.name);
        println!("     Distribution: {}", env.distro.as_str());
        println!("     Package Manager: {}", env.distro.pkg_manager());
    } else {
        println!("   ✗ Environment not found");
    }

    println!("4. Testing command execution...");
    if let Some(env) = config.get_env(test_env_name) {
        match Runner::exec(env, &vec!["echo".to_string(), "Hello from test environment".to_string()]) {
            Ok(_) => println!("   ✓ Command executed successfully"),
            Err(e) => println!("   ✗ Command execution failed: {}", e),
        }
    }

    println!();
    println!("=== Test Complete ===");
    Ok(())
}
