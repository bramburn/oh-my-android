//! Tauri application wiring.

pub mod commands;
pub mod error;
pub mod state;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::adb_start,
            commands::adb_stop
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Android Panel");
}
