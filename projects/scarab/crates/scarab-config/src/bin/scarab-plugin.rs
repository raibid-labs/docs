//! Scarab Plugin CLI
//!
//! Command-line interface for managing Scarab plugins

use scarab_config::prelude::*;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "search" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin search <query>");
                return Ok(());
            }
            cmd_search(&args[2..]).await?;
        }
        "install" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin install <name> [version]");
                return Ok(());
            }
            let version = args.get(3).map(|s| s.as_str());
            cmd_install(&args[2], version).await?;
        }
        "update" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin update <name>");
                return Ok(());
            }
            cmd_update(&args[2]).await?;
        }
        "remove" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin remove <name>");
                return Ok(());
            }
            cmd_remove(&args[2]).await?;
        }
        "list" => {
            cmd_list().await?;
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin info <name>");
                return Ok(());
            }
            cmd_info(&args[2]).await?;
        }
        "sync" => {
            cmd_sync().await?;
        }
        "check-updates" => {
            cmd_check_updates().await?;
        }
        "enable" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin enable <name>");
                return Ok(());
            }
            cmd_enable(&args[2]).await?;
        }
        "disable" => {
            if args.len() < 3 {
                eprintln!("Usage: scarab-plugin disable <name>");
                return Ok(());
            }
            cmd_disable(&args[2]).await?;
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"Scarab Plugin Manager

USAGE:
    scarab-plugin <COMMAND> [OPTIONS]

COMMANDS:
    search <query>           Search for plugins by name, description, or author
    install <name> [version] Install a plugin from the registry
    update <name>            Update an installed plugin to the latest version
    remove <name>            Remove an installed plugin
    list                     List all installed plugins
    info <name>              Show detailed information about a plugin
    sync                     Synchronize with remote registry
    check-updates            Check for updates to installed plugins
    enable <name>            Enable a plugin
    disable <name>           Disable a plugin
    help                     Show this help message

EXAMPLES:
    # Search for notification plugins
    scarab-plugin search notification

    # Install a plugin
    scarab-plugin install auto-notify

    # Install specific version
    scarab-plugin install auto-notify 1.2.0

    # Update all plugins
    scarab-plugin check-updates

    # Show plugin details
    scarab-plugin info auto-notify

For more information, visit: https://docs.scarab.dev/plugins
"#
    );
}

async fn cmd_search(args: &[String]) -> anyhow::Result<()> {
    let query = args.join(" ");
    let mut manager = RegistryManager::new()?;

    // Sync if cache is stale
    if manager.cache.is_stale() {
        println!("Registry cache is stale, syncing...");
        manager.sync().await?;
    }

    let filter = PluginFilter {
        query: Some(query.clone()),
        limit: Some(20),
        ..Default::default()
    };

    let results = manager.search(&filter)?;

    if results.is_empty() {
        println!("No plugins found matching '{}'", query);
        return Ok(());
    }

    println!("Found {} plugin(s):\n", results.len());

    for plugin in results {
        println!("  {} ({})", plugin.name, plugin.latest_version);
        println!("    {}", plugin.description);
        println!(
            "    Author: {} | Downloads: {} | Rating: {:.1}/5.0",
            plugin.author, plugin.stats.downloads, plugin.stats.rating
        );
        if !plugin.tags.is_empty() {
            println!("    Tags: {}", plugin.tags.join(", "));
        }
        println!();
    }

    Ok(())
}

async fn cmd_install(name: &str, version: Option<&str>) -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    println!("Installing plugin '{}'...", name);

    let installed = manager.install(name, version).await?;

    println!(
        "Successfully installed {} v{} to {}",
        installed.name,
        installed.version,
        installed.path.display()
    );

    Ok(())
}

async fn cmd_update(name: &str) -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    println!("Updating plugin '{}'...", name);

    let installed = manager.update(name).await?;

    println!(
        "Successfully updated {} to v{}",
        installed.name, installed.version
    );

    Ok(())
}

async fn cmd_remove(name: &str) -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    println!("Removing plugin '{}'...", name);
    manager.remove(name)?;
    println!("Successfully removed {}", name);

    Ok(())
}

async fn cmd_list() -> anyhow::Result<()> {
    let manager = RegistryManager::new()?;
    let installed = manager.list_installed()?;

    if installed.is_empty() {
        println!("No plugins installed");
        return Ok(());
    }

    println!("Installed plugins ({}):\n", installed.len());

    for plugin in installed {
        let status = if plugin.enabled { "enabled" } else { "disabled" };
        println!(
            "  {} v{} [{}]",
            plugin.name, plugin.version, status
        );
        println!("    Path: {}", plugin.path.display());
        println!();
    }

    Ok(())
}

async fn cmd_info(name: &str) -> anyhow::Result<()> {
    let manager = RegistryManager::new()?;

    // Try to get from registry
    if let Some(entry) = manager.get_plugin(name)? {
        println!("Plugin: {}", entry.name);
        println!("Description: {}", entry.description);
        println!("Author: {}", entry.author);
        println!("Latest Version: {}", entry.latest_version);
        println!("License: {}", entry.license);

        if let Some(homepage) = &entry.homepage {
            println!("Homepage: {}", homepage);
        }
        if let Some(repo) = &entry.repository {
            println!("Repository: {}", repo);
        }

        println!("\nStatistics:");
        println!("  Downloads: {}", entry.stats.downloads);
        println!("  Rating: {:.1}/5.0 ({} ratings)", entry.stats.rating, entry.stats.rating_count);

        if !entry.tags.is_empty() {
            println!("\nTags: {}", entry.tags.join(", "));
        }

        println!("\nAvailable Versions:");
        for version in entry.versions.iter().take(5) {
            let prerelease = if version.prerelease { " (prerelease)" } else { "" };
            println!("  {} - {} bytes{}", version.version, version.size, prerelease);
        }

        // Check if installed
        if let Ok(installed) = manager.installer.get_installed(name) {
            println!("\nInstalled: v{}", installed.version);
            println!("Status: {}", if installed.enabled { "enabled" } else { "disabled" });
        }
    } else {
        println!("Plugin '{}' not found in registry", name);
    }

    Ok(())
}

async fn cmd_sync() -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    println!("Synchronizing with registry...");
    manager.sync().await?;

    let manifest = manager.cache.manifest();
    println!(
        "Successfully synced {} plugins from {}",
        manifest.count(),
        manifest.metadata.url
    );

    Ok(())
}

async fn cmd_check_updates() -> anyhow::Result<()> {
    let manager = RegistryManager::new()?;

    println!("Checking for updates...");
    let updates = manager.check_updates()?;

    if updates.is_empty() {
        println!("All plugins are up to date");
        return Ok(());
    }

    println!("Updates available ({}):\n", updates.len());

    for (name, current, latest) in updates {
        println!("  {} {} -> {}", name, current, latest);
    }

    println!("\nRun 'scarab-plugin update <name>' to update a plugin");

    Ok(())
}

async fn cmd_enable(name: &str) -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    manager.installer.enable(name)?;
    println!("Enabled plugin '{}'", name);

    Ok(())
}

async fn cmd_disable(name: &str) -> anyhow::Result<()> {
    let mut manager = RegistryManager::new()?;

    manager.installer.disable(name)?;
    println!("Disabled plugin '{}'", name);

    Ok(())
}
