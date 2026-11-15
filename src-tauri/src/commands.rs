//! The entire IPC surface: setup guidance + the three device/server commands.
//!
//! These functions are thin (`src-tauri/AGENTS.md` §1): resolve state → call `adbkit` → map the error. No
//! parsing, no spawning, no business logic. Note that **no command takes a device serial**: the MVP has one
//! implicit device list and no per-device action, so `-s` is vacuous here and becomes mandatory with the
//! first mutating command.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use adbkit::sdk::SdkSource;
use adbkit::{Device, SetupReport};

use crate::error::AppError;
use crate::state::AppState;

/// The **only** URLs this app may hand to the system browser.
///
/// Hard-coded in Rust so there is exactly one allowlist; the frontend cannot widen it and the opener
/// capability is scoped to the same four strings. `developer.android.com` is the official docs host.
pub const ALLOWED_DOC_URLS: [&str; 4] = [
    "https://developer.android.com/studio",
    "https://developer.android.com/tools/adb",
    "https://developer.android.com/studio/run/managing-avds",
    "https://developer.android.com/studio/debug/dev-options",
];

/// Read-only detection of Studio / SDK / adb / emulator / AVDs, plus the `adb version` probe.
///
/// The probe is the only process the setup flow runs. `adb version` is client-local: it prints and exits
/// **without** starting the server (verified — `docs/windows-port/21-mvp-log.md` §7), so it is safe to run
/// before the user has approved anything.
#[tauri::command]
pub async fn setup_report(state: State<'_, AppState>) -> Result<SetupReport, AppError> {
    let custom = state.custom_root();
    let warning = state.warning();
    let mut report = adbkit::setup::detect(custom.as_deref(), warning);

    if let Some(location) = report.sdk.clone() {
        // A failed probe is not an error: the SDK row already says what is wrong.
        if let Ok(output) = adbkit::runner::run(&location.adb, &["version"]).await {
            if output.status.success() {
                report.adb_version =
                    adbkit::parser::parse_adb_version(&String::from_utf8_lossy(&output.stdout));
            }
        }
    }

    Ok(report)
}

/// Native folder picker → validate → persist → fresh report.
///
/// Cancelling is **not** an error: the dialog closes and the caller keeps the current report. An invalid
/// folder is an error with the upstream wording (`Sources/App/SettingsView.swift:81-82`).
#[tauri::command]
pub async fn choose_sdk_folder(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SetupReport, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose the Android SDK folder (it contains platform-tools)")
        .pick_folder(move |picked| {
            let _ = tx.send(picked);
        });

    let picked = rx.await.map_err(|_| AppError::dialog_closed())?;

    let Some(file_path) = picked else {
        // Cancelled: re-report the current truth instead of inventing an error.
        return setup_report_after_change(&state);
    };

    let dir = file_path
        .into_path()
        .map_err(|e| AppError::invalid_sdk(format!("That folder cannot be used: {e}")))?;

    // Same validator the locator uses, so an accepted folder can never fail discovery later.
    if adbkit::sdk::validate_root(&dir, SdkSource::Custom).is_none() {
        return Err(AppError::invalid_sdk(format!(
            r"No platform-tools\adb in {}.",
            dir.display()
        )));
    }

    state.set_custom_root(Some(dir))?;
    setup_report_after_change(&state)
}

/// Shared tail of the picker: re-run detection after the config changed.
fn setup_report_after_change(state: &AppState) -> Result<SetupReport, AppError> {
    let custom = state.custom_root();
    Ok(adbkit::setup::detect(custom.as_deref(), state.warning()))
}

/// `adb devices -l`, parsed. Refresh-only by design (override O1).
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
/// Human-approval list (`AGENTS.md` §7): the only caller is the UI button behind a confirmation dialog
/// (override O2). Never automatic, never from a test.
#[tauri::command]
pub async fn adb_stop(state: State<'_, AppState>) -> Result<(), AppError> {
    let adb = state.adb_path()?;
    adbkit::runner::run_server_cmd(&adb, "kill-server").await?;
    Ok(())
}

/// Open one of the four official documentation pages in the system browser.
///
/// The URL must match [`ALLOWED_DOC_URLS`] exactly. Keeping this in Rust (rather than calling the opener
/// plugin from the webview) means the allowlist cannot be bypassed by any DOM-level trick.
#[tauri::command]
pub async fn open_docs(app: AppHandle, url: String) -> Result<(), AppError> {
    use tauri_plugin_opener::OpenerExt;

    if !ALLOWED_DOC_URLS.contains(&url.as_str()) {
        return Err(AppError::url_not_allowed(url));
    }
    app.opener()
        .open_url(url.clone(), None::<&str>)
        .map_err(|e| AppError::open_failed(format!("Could not open {url}: {e}")))
}

/// `app_config_dir()` — exposed for tests and for the About/credits line, never for writing.
pub fn config_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map_err(|e| AppError::config_write(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::ALLOWED_DOC_URLS;

    #[test]
    fn the_allowlist_is_exactly_the_four_official_pages() {
        assert_eq!(ALLOWED_DOC_URLS.len(), 4);
        for url in ALLOWED_DOC_URLS {
            assert!(url.starts_with("https://developer.android.com/"), "{url}");
        }
    }

    #[test]
    fn lookalike_hosts_are_not_in_the_allowlist() {
        for bad in [
            "https://developer.android.com.evil.tld/studio",
            "http://developer.android.com/studio",
            "https://evil.tld/studio",
        ] {
            assert!(
                !ALLOWED_DOC_URLS.contains(&bad),
                "{bad} must not be allowed"
            );
        }
    }
}
