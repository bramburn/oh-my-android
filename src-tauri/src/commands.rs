//! The entire IPC surface for the MVP: list devices, start the adb server, stop the adb server.
//!
//! These functions are intentionally thin (`AGENTS.md` §4 rule 4): resolve state → call `adbkit` → map the
//! error. No parsing, no spawning, no business logic lives here.

use tauri::State;

use adbkit::Device;

use crate::error::AppError;
use crate::state::AppState;

/// `adb devices -l`, parsed.
///
/// Refresh-only by design for the MVP (override O1 in `docs/windows-port/21-mvp-log.md`).
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<Device>, AppError> {
    let adb = state.adb_path()?;
    let output = adbkit::runner::run(&adb, &["devices", "-l"]).await?;
    if !output.status.success() {
        return Err(AppError::from_output(output));
    }
    Ok(adbkit::parser::parse_devices(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

/// `adb start-server`. All stdio null — see `adbkit::runner::run_server_cmd`.
#[tauri::command]
pub async fn adb_start(state: State<'_, AppState>) -> Result<(), AppError> {
    let adb = state.adb_path()?;
    adbkit::runner::run_server_cmd(&adb, "start-server").await?;
    Ok(())
}

/// `adb kill-server`.
///
/// This is on the human-approval list (`AGENTS.md` §7): the only caller is the UI button, behind a
/// confirmation dialog (override O2). It is never invoked automatically and never from a test.
#[tauri::command]
pub async fn adb_stop(state: State<'_, AppState>) -> Result<(), AppError> {
    let adb = state.adb_path()?;
    adbkit::runner::run_server_cmd(&adb, "kill-server").await?;
    Ok(())
}
