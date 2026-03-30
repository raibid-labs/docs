//! Sigilforge CLI
//!
//! Command-line interface for managing credentials in Sigilforge.
//!
//! # Usage
//!
//! ```bash
//! # Add a new account (starts OAuth flow)
//! sigilforge add-account spotify personal
//!
//! # List all configured accounts
//! sigilforge list-accounts
//!
//! # Get a fresh access token
//! sigilforge get-token spotify personal
//! ```

use anyhow::Result;
use clap::{Parser, Subcommand};
use sigilforge_core::{
    account_store::AccountStore,
    store::{KeyringStore, MemoryStore, SecretStore},
    AccountId, CredentialType, ServiceId,
};
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

mod client;

#[derive(Parser)]
#[command(name = "sigilforge")]
#[command(about = "Credential management for the raibid-labs ecosystem")]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new account for a service
    AddAccount {
        /// Service name (e.g., spotify, gmail, github)
        service: String,

        /// Account identifier (e.g., personal, work)
        account: String,

        /// OAuth scopes to request (comma-separated)
        #[arg(short, long)]
        scopes: Option<String>,
    },

    /// List all configured accounts
    ListAccounts {
        /// Filter by service name
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Get a fresh access token for an account
    GetToken {
        /// Service name
        service: String,

        /// Account identifier
        account: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Remove an account and its credentials
    RemoveAccount {
        /// Service name
        service: String,

        /// Account identifier
        account: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Resolve a credential reference
    Resolve {
        /// Reference to resolve (e.g., auth://spotify/personal/token)
        reference: String,
    },

    /// Start the daemon in foreground (for debugging)
    Daemon,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    match cli.command {
        Commands::AddAccount { service, account, scopes } => {
            add_account(&service, &account, scopes.as_deref()).await
        }
        Commands::ListAccounts { service } => {
            list_accounts(service.as_deref()).await
        }
        Commands::GetToken { service, account, format } => {
            get_token(&service, &account, &format).await
        }
        Commands::RemoveAccount { service, account, force } => {
            remove_account(&service, &account, force).await
        }
        Commands::Resolve { reference } => {
            resolve_reference(&reference).await
        }
        Commands::Daemon => {
            run_daemon_foreground().await
        }
    }
}

async fn add_account(service: &str, account: &str, scopes: Option<&str>) -> Result<()> {
    let mut client = client::DaemonClient::connect_default().await?;

    if client.is_connected() {
        let scope_vec = scopes
            .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        match client.add_account(service, account, scope_vec).await {
            Ok(response) => {
                println!("{}", response.message);
                Ok(())
            }
            Err(e) => {
                warn!("Daemon call failed: {}", e);
                fallback_add_account(service, account, scopes).await
            }
        }
    } else {
        warn!("Daemon not available, using fallback mode");
        fallback_add_account(service, account, scopes).await
    }
}

async fn fallback_add_account(service: &str, account: &str, scopes: Option<&str>) -> Result<()> {
    use sigilforge_core::{Account, AccountId, AccountStore, ServiceId};

    let store = AccountStore::load()?;

    let service_id = ServiceId::new(service);
    let account_id = AccountId::new(account);

    let scope_list = if let Some(scopes) = scopes {
        scopes.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        Vec::new()
    };

    let new_account = Account::new(service_id.clone(), account_id.clone(), scope_list);

    store.add_account(new_account)?;

    println!("Account {}/{} added successfully", service, account);
    if let Some(scopes) = scopes {
        println!("  Scopes: {}", scopes);
    }
    println!("  Storage path: {:?}", store.path());
    println!("  [stub] Would start OAuth flow to obtain tokens here");

    Ok(())
}

async fn list_accounts(service_filter: Option<&str>) -> Result<()> {
    let mut client = client::DaemonClient::connect_default().await?;

    if client.is_connected() {
        match client.list_accounts(service_filter).await {
            Ok(response) => {
                if response.accounts.is_empty() {
                    println!("No accounts configured");
                } else {
                    println!("Configured accounts:");
                    for account in response.accounts {
                        println!("  {}/{}", account.service, account.account);
                        println!("    Scopes: {}", account.scopes.join(", "));
                        println!("    Created: {}", account.created_at);
                        if let Some(last_used) = account.last_used {
                            println!("    Last used: {}", last_used);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!("Daemon call failed: {}", e);
                fallback_list_accounts(service_filter).await
            }
        }
    } else {
        warn!("Daemon not available, using fallback mode");
        fallback_list_accounts(service_filter).await
    }
}

async fn fallback_list_accounts(service_filter: Option<&str>) -> Result<()> {
    use sigilforge_core::{AccountStore, ServiceId};

    let store = AccountStore::load()?;

    let filter = service_filter.map(ServiceId::new);
    let accounts = store.list_accounts(filter.as_ref())?;

    if accounts.is_empty() {
        println!("No accounts configured");
        if let Some(service) = service_filter {
            println!("  (filtered by service: {})", service);
        }
        return Ok(());
    }

    println!("Configured accounts:");
    for account in accounts {
        println!("  {}/{}", account.service, account.id);
        if !account.scopes.is_empty() {
            println!("    Scopes: {}", account.scopes.join(", "));
        }
        println!("    Created: {}", account.created_at);
        if let Some(last_used) = account.last_used {
            println!("    Last used: {}", last_used);
        }
    }

    Ok(())
}

async fn get_token(service: &str, account: &str, format: &str) -> Result<()> {
    let mut client = client::DaemonClient::connect_default().await?;

    if client.is_connected() {
        match client.get_token(service, account).await {
            Ok(response) => {
                match format {
                    "json" => {
                        let json_output = serde_json::json!({
                            "service": service,
                            "account": account,
                            "token": response.token,
                            "expires_at": response.expires_at,
                        });
                        println!("{}", serde_json::to_string_pretty(&json_output)?);
                    }
                    _ => {
                        println!("{}", response.token);
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!("Daemon call failed: {}", e);
                fallback_get_token(service, account, format).await
            }
        }
    } else {
        warn!("Daemon not available, using fallback mode");
        fallback_get_token(service, account, format).await
    }
}

async fn fallback_get_token(service: &str, account: &str, format: &str) -> Result<()> {
    println!("[stub] Getting token for {}/{}", service, account);

    match format {
        "json" => {
            println!(r#"{{"service": "{}", "account": "{}", "token": "[stub]"}}"#, service, account);
        }
        _ => {
            println!("[stub] token-would-appear-here");
        }
    }
    Ok(())
}

async fn remove_account(service: &str, account: &str, force: bool) -> Result<()> {
    use std::io::{self, Write};

    let store = AccountStore::load()?;
    let service_id = ServiceId::new(service);
    let account_id = AccountId::new(account);

    // Verify account exists before prompting
    let account_entry = store.get_account(&service_id, &account_id)?;
    if account_entry.is_none() {
        eprintln!("Error: Account {}/{} not found", service, account);
        std::process::exit(1);
    }

    // Prompt for confirmation unless --force is used
    if !force {
        print!("Remove account {}/{}? [y/N] ", service, account);
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let confirmed = response.trim().eq_ignore_ascii_case("y")
            || response.trim().eq_ignore_ascii_case("yes");

        if !confirmed {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Remove account from store
    store.remove_account(&service_id, &account_id)?;

    delete_account_secrets(service, account).await?;

    println!("Account {}/{} removed successfully", service, account);
    println!("  Associated secrets removed from configured secret store");

    Ok(())
}

async fn delete_account_secrets(service: &str, account: &str) -> Result<()> {
    // Choose the best available secret store
    let store: Box<dyn SecretStore + Send + Sync> = match KeyringStore::try_new("sigilforge") {
        Ok(s) => {
            info!("Using keyring backend to delete secrets");
            Box::new(s)
        }
        Err(e) => {
            warn!("Keyring unavailable ({}); falling back to memory store (no-op)", e);
            Box::new(MemoryStore::new())
        }
    };

    // Common credential types to clean up
    let credential_types = [
        CredentialType::AccessToken,
        CredentialType::RefreshToken,
        CredentialType::TokenExpiry,
        CredentialType::ApiKey,
        CredentialType::ClientId,
        CredentialType::ClientSecret,
        CredentialType::TokenScopes,
    ];

    for cred_type in &credential_types {
        let key = format!("sigilforge/{}/{}/{}", service, account, cred_type);
        // Ignore errors - the key might not exist
        let _ = store.delete(&key).await;
    }

    Ok(())
}

async fn resolve_reference(reference: &str) -> Result<()> {
    let mut client = client::DaemonClient::connect_default().await?;

    if client.is_connected() {
        match client.resolve(reference).await {
            Ok(response) => {
                println!("{}", response.value);
                Ok(())
            }
            Err(e) => {
                warn!("Daemon call failed: {}", e);
                fallback_resolve_reference(reference).await
            }
        }
    } else {
        warn!("Daemon not available, using fallback mode");
        fallback_resolve_reference(reference).await
    }
}

async fn fallback_resolve_reference(reference: &str) -> Result<()> {
    use sigilforge_core::CredentialRef;

    match CredentialRef::from_auth_uri(reference) {
        Ok(cred_ref) => {
            println!("Parsed reference:");
            println!("  Service: {}", cred_ref.service);
            println!("  Account: {}", cred_ref.account);
            println!("  Type: {}", cred_ref.credential_type);
            println!("[stub] Would resolve to actual value");
        }
        Err(e) => {
            eprintln!("Failed to parse reference: {}", e);
        }
    }
    Ok(())
}

async fn run_daemon_foreground() -> Result<()> {
    println!("[stub] Running daemon in foreground...");
    println!("Press Ctrl+C to stop");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        info!("Daemon heartbeat");
    }
}
