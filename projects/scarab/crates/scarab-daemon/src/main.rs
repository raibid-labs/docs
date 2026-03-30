use anyhow::Result;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use scarab_config::ConfigLoader;
use scarab_protocol::{SharedState, SHMEM_PATH};
use shared_memory::ShmemConf;
use std::io::Read;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use scarab_daemon::ipc::{ClientRegistry, IpcServer, PtyHandle};
use scarab_daemon::plugin_manager::PluginManager;
use scarab_daemon::session::SessionManager;
use scarab_daemon::vte;

use scarab_plugin_api::context::PluginSharedState;
use scarab_plugin_api::PluginContext;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Scarab Daemon...");

    // 0. Load Configuration (Fusabi-based)
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let fusabi_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.fsx");
    let toml_config_path = std::path::PathBuf::from(&home_dir)
        .join(".config/scarab/config.toml");

    let config = if fusabi_config_path.exists() {
        println!("Loading Fusabi config from: {}", fusabi_config_path.display());
        scarab_config::FusabiConfigLoader::from_file(&fusabi_config_path)?
    } else if toml_config_path.exists() {
        println!("⚠️  Loading legacy TOML config from: {}", toml_config_path.display());
        println!("💡 Consider migrating to Fusabi config: {}", fusabi_config_path.display());
        ConfigLoader::from_file(&toml_config_path)?
    } else {
        println!("No config found (tried .fsx and .toml), using defaults");
        println!("Create {} to customize your terminal", fusabi_config_path.display());
        scarab_config::ScarabConfig::default()
    };

    // 1. Initialize Session Manager
    let db_path = std::path::PathBuf::from(&home_dir).join(".local/share/scarab/sessions.db");

    let session_manager = std::sync::Arc::new(SessionManager::new(db_path)?);

    // Restore sessions from previous daemon runs
    session_manager.restore_sessions()?;
    println!(
        "Session Manager: Active ({} sessions)",
        session_manager.session_count()
    );

    // Create default session if none exist
    if session_manager.session_count() == 0 {
        let cols = config.terminal.columns;
        let rows = config.terminal.rows;
        let session_id = session_manager.create_session("default".to_string(), cols, rows)?;
        println!(
            "Created default session: {} ({}x{})",
            session_id, cols, rows
        );
    }

    // 2. Setup PTY System (legacy - will be per-session)
    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows: config.terminal.rows,
        cols: config.terminal.columns,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Use configured shell or fallback to bash
    let shell = &config.terminal.default_shell;
    let mut cmd = CommandBuilder::new(shell);

    // Set the working directory to user's home
    if let Ok(home) = std::env::var("HOME") {
        cmd.cwd(home);
    }

    // Set environment variable for Navigation Protocol
    cmd.env("SCARAB_NAV_SOCKET", "/tmp/scarab-nav.sock");

    let _child = pair.slave.spawn_command(cmd)?;
    println!(
        "Spawned shell: {} ({}x{})",
        shell, config.terminal.columns, config.terminal.rows
    );

    // Important: Release slave handle in parent process
    drop(pair.slave);

    let reader = pair.master.try_clone_reader()?;
    let reader = Arc::new(Mutex::new(reader));
    let writer = pair.master.take_writer()?;

    // 2. Initialize Shared Memory
    // Try to create new shared memory, or open existing if it already exists
    let shmem = match ShmemConf::new()
        .size(std::mem::size_of::<SharedState>())
        .os_id(SHMEM_PATH)
        .create()
    {
        Ok(shmem) => {
            println!("Created shared memory at: {}", SHMEM_PATH);
            shmem
        }
        Err(_) => {
            // Shared memory already exists, try to open it
            println!("Shared memory already exists, attempting to open...");
            match ShmemConf::new().os_id(SHMEM_PATH).open() {
                Ok(shmem) => {
                    println!("Opened existing shared memory at: {}", SHMEM_PATH);
                    shmem
                }
                Err(e) => {
                    eprintln!("Failed to open existing shared memory: {}", e);
                    eprintln!("Try cleaning up with: ipcrm -M $(ipcs -m | grep scarab | awk '{{print $1}}')");
                    return Err(e.into());
                }
            }
        }
    };

    // Initialize shared state with zeroed memory
    let shared_ptr = shmem.as_ptr() as *mut SharedState;
    unsafe {
        std::ptr::write_bytes(shared_ptr, 0, 1);
    }

    let sequence_counter = Arc::new(AtomicU64::new(0));

    // Initialize VTE parser
    let mut terminal_state = vte::TerminalState::new(shared_ptr, sequence_counter.clone());
    println!("VTE Parser: Active");
    println!("Scrollback buffer: 10,000 lines");

    // 3. Setup IPC Control Channel with channels for thread safety
    let (resize_tx, mut resize_rx) = mpsc::channel::<PtySize>(32);
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    let pty_handle = PtyHandle::new(input_tx, resize_tx);

    let client_registry = ClientRegistry::new();

    // Initialize Plugin Manager
    let plugin_state = Arc::new(parking_lot::Mutex::new(PluginSharedState::new(
        config.terminal.columns,
        config.terminal.rows,
    )));
    let plugin_ctx = Arc::new(PluginContext::new(
        Default::default(),
        plugin_state.clone(),
        "daemon",
    ));
    let mut plugin_manager = PluginManager::new(plugin_ctx, client_registry.clone());

    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_nav::NavigationPlugin::new()))
        .await
    {
        eprintln!("Failed to register NavigationPlugin: {}", e);
    }

    // Register Palette Plugin
    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_palette::PalettePlugin::new()))
        .await
    {
        eprintln!("Failed to register PalettePlugin: {}", e);
    }

    // Register Session Plugin
    if let Err(e) = plugin_manager
        .register_plugin(Box::new(scarab_session::SessionPlugin::new()))
        .await
    {
        eprintln!("Failed to register SessionPlugin: {}", e);
    }

    // Discover and load plugins
    if let Err(e) = plugin_manager.discover_and_load().await {
        eprintln!("Failed to load plugins: {}", e);
    }

    let plugin_manager = Arc::new(tokio::sync::Mutex::new(plugin_manager));

    let ipc_server = IpcServer::new(
        pty_handle.clone(),
        session_manager.clone(),
        client_registry.clone(),
        plugin_manager.clone(),
    )
    .await?;

    // Spawn IPC server task
    tokio::spawn(async move {
        if let Err(e) = ipc_server.accept_loop().await {
            eprintln!("IPC server error: {}", e);
        }
    });

    // Spawn PTY writer task to handle input from IPC
    let mut writer = writer;
    let pm_input = plugin_manager.clone();
    tokio::spawn(async move {
        use std::io::Write;
        while let Some(data) = input_rx.recv().await {
            // Dispatch input to plugins
            let processed_data = {
                let mut pm = pm_input.lock().await;
                match pm.dispatch_input(&data).await {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Plugin input error: {}", e);
                        data
                    }
                }
            };

            if processed_data.is_empty() {
                continue; // Input consumed by plugin
            }

            if let Err(e) = writer.write_all(&processed_data) {
                eprintln!("PTY write error: {}", e);
                break;
            }
            if let Err(e) = writer.flush() {
                eprintln!("PTY flush error: {}", e);
                break;
            }
        }
    });

    println!("Daemon initialized. Listening for input...");

    // 4. Main Loop with PTY reading and resize handling
    loop {
        tokio::select! {
            // Handle PTY output
            read_result = tokio::task::spawn_blocking({
                let reader_clone = Arc::clone(&reader);
                move || {
                    let mut buf = [0u8; 4096];
                    let mut reader_lock = reader_clone.lock().unwrap();
                    reader_lock.read(&mut buf).map(|n| (n, buf))
                }
            }) => {
                match read_result? {
                    Ok((n, buf)) if n > 0 => {
                        let data = &buf[..n];

                        // Debug output (can be disabled in production)
                        if cfg!(debug_assertions) {
                            print!("{}", String::from_utf8_lossy(data));
                        }

                        // Process output through plugins first
                        let data_str = String::from_utf8_lossy(data);

                        let processed_data = {
                            let mut pm = plugin_manager.lock().await;
                            match pm.dispatch_output(&data_str).await {
                                Ok(d) => d,
                                Err(e) => {
                                    eprintln!("Plugin dispatch error: {}", e);
                                    data_str.to_string()
                                }
                            }
                        };

                        // Process output through VTE parser
                        // This will:
                        // 1. Parse ANSI escape sequences
                        // 2. Update grid cells with proper colors and attributes
                        // 3. Handle cursor positioning
                        // 4. Manage scrollback buffer
                        // 5. Atomically update shared state
                        terminal_state.process_output(processed_data.as_bytes());
                    }
                    Ok(_) => break, // EOF
                    Err(e) => {
                        eprintln!("PTY Error: {}", e);
                        break;
                    }
                }
            }

            // Handle resize events from IPC
            Some(pty_size) = resize_rx.recv() => {
                println!("Resizing PTY to {}x{}", pty_size.cols, pty_size.rows);
                if let Err(e) = pair.master.resize(pty_size) {
                    eprintln!("Failed to resize PTY: {}", e);
                }
            }
        }
    }

    // Cleanup shared memory
    drop(shmem);
    println!("Daemon shutting down...");

    Ok(())
}
