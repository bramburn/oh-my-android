//! The only error shape that crosses the IPC boundary.
//!
//! `code` is stable and machine-readable; `message` is a sentence the UI can show as-is
//! (`docs/windows-port/12-ipc-contract.md` §3).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// The folder the user picked is not an SDK root (no `platform-toolsdb.exe`).
    pub fn invalid_sdk(message: impl Into<String>) -> Self {
        Self::new("SDK_INVALID", message)
    }

    /// Writing the config file failed.
    pub fn config_write(detail: impl Into<String>) -> Self {
        Self::new(
            "CONFIG_WRITE_FAILED",
            format!(
                "Could not save your settings: {}. The choice will work for this session only.",
                detail.into()
            ),
        )
    }

    /// The folder-picker dialog went away without answering.
    pub fn dialog_closed() -> Self {
        Self::new(
            "DIALOG_CLOSED",
            "The folder dialog closed unexpectedly. Try again.",
        )
    }

    /// A URL that is not on the documentation allowlist.
    pub fn url_not_allowed(url: impl Into<String>) -> Self {
        Self::new(
            "URL_NOT_ALLOWED",
            format!("Refusing to open an unknown link: {}", url.into()),
        )
    }

    /// The system browser could not be launched.
    pub fn open_failed(message: impl Into<String>) -> Self {
        Self::new("OPEN_FAILED", message)
    }

    pub fn sdk_missing() -> Self {
        Self::new(
            "SDK_NOT_FOUND",
            "Android SDK not found. Install Android Studio or set ANDROID_HOME, then Refresh.",
        )
    }

    /// adb ran and exited non-zero. stderr is the useful half; stdout is usually empty.
    pub fn from_output(output: std::process::Output) -> Self {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Self::new(
            "ADB_FAILED",
            if detail.is_empty() {
                format!("adb exited with {}", output.status)
            } else {
                detail
            },
        )
    }
}

impl From<adbkit::AdbError> for AppError {
    fn from(error: adbkit::AdbError) -> Self {
        use adbkit::AdbError;
        match error {
            AdbError::Spawn(_) => Self::new(
                "ADB_SPAWN_FAILED",
                "adb could not be started. Check that platform-tools is intact, then Refresh.",
            ),
            AdbError::Io(_) => Self::new("ADB_IO_ERROR", "Lost contact with adb. Try Refresh."),
            AdbError::Timeout(timeout) => Self::new(
                "ADB_TIMEOUT",
                format!("adb did not respond within {}s.", timeout.as_secs()),
            ),
            AdbError::NonZero { stderr } => Self::new("ADB_FAILED", stderr),
            AdbError::SdkMissing => Self::sdk_missing(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_camel_case_keys() {
        let json = serde_json::to_string(&AppError::sdk_missing()).unwrap();
        assert!(
            json.starts_with(r#"{"code":"SDK_NOT_FOUND","message":"#),
            "{json}"
        );
    }

    #[test]
    fn timeout_message_mentions_seconds() {
        let err = AppError::from(adbkit::AdbError::Timeout(std::time::Duration::from_secs(
            20,
        )));
        assert_eq!(err.code, "ADB_TIMEOUT");
        assert!(err.message.contains("20s"), "{}", err.message);
    }

    #[test]
    fn spawn_errors_do_not_leak_paths_to_the_ui() {
        let err = AppError::from(adbkit::AdbError::Spawn("C:\\x\\adb.exe: denied".into()));
        assert!(!err.message.contains("C:\\x"), "{}", err.message);
    }
}
