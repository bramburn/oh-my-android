//! All adb access for the Windows app.
//!
//! Invariant (root `AGENTS.md` §6.1): this crate is the **only** place that spawns a process.
//! `src-tauri` and the UI never do. See `AGENTS.md` in this directory.

pub mod parser;

pub use parser::{parse_devices, Device, DeviceState};

/// Everything that can go wrong talking to adb.
///
/// Serialized to the UI as `{ code, message }` (see `src-tauri/src/error.rs`), where `code` is the
/// variant name and `message` is a human-readable sentence with a hint where one is known.
#[derive(Debug, thiserror::Error)]
pub enum AdbError {
    /// `adb.exe` could not be started at all (missing file, access denied, ...).
    #[error("could not start adb: {0}")]
    Spawn(String),

    /// I/O failure while a child was running.
    #[error("adb I/O error: {0}")]
    Io(String),

    /// The child did not finish within the timeout and was killed.
    #[error("adb did not respond within {0:?}")]
    Timeout(std::time::Duration),

    /// adb ran and exited non-zero.
    #[error("adb failed: {stderr}")]
    NonZero { stderr: String },

    /// No Android SDK with `platform-tools\adb.exe` was found.
    #[error("Android SDK not found. Install Android Studio or set ANDROID_HOME, then Refresh.")]
    SdkMissing,
}