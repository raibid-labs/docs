// Shared command functions for both desktop and mobile

// Custom Tauri command for greeting
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Tauri on your platform!", name)
}

// Custom command to get platform info
#[tauri::command]
pub fn get_platform_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    format!("Running on {} ({})", os, arch)
}
