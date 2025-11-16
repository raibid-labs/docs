// Mobile-specific entry points for iOS and Android

#[tauri::mobile_entry_point]
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            crate::commands::greet,
            crate::commands::get_platform_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
