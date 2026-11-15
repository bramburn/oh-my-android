//! Tauri application wiring.
//!
//! Startup order matters: the config is loaded **before** the window exists, so the first `setup_report`
//! call already knows about a saved SDK folder and any config warning.

pub mod commands;
pub mod config;
pub mod error;
pub mod state;

use tauri::Manager;

use config::Config;
use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            let (config, warning) = Config::load(&dir);
            app.manage(AppState::new(dir, config, warning));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::setup_report,
            commands::choose_sdk_folder,
            commands::list_devices,
            commands::adb_start,
            commands::adb_stop,
            commands::open_docs
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Android Panel");
}

#[cfg(test)]
mod tests {
    #[test]
    fn config_lives_under_the_app_identifier() {
        // Tauri derives `app_config_dir` from the bundle identifier in `tauri.conf.json`; keeping the two
        // in sync is what makes the documented path true.
        let raw = include_str!("../tauri.conf.json");
        assert!(raw.contains("\"identifier\": \"dev.bramburn.androidpanel\""));
    }
}
