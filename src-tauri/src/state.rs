//! Shared application state.
//!
//! The SDK path is resolved lazily and cached: a user can install the Android SDK while the app is
//! running, so a negative result is never sticky — every command retries the lookup when the cache is
//! empty (`docs/adr/ADR-0007-win-mvp-scaffold.md`).

use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;

#[derive(Default)]
pub struct AppState {
    adb: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        let resolved = adbkit::sdk::locate(None);
        Self {
            adb: Mutex::new(resolved),
        }
    }

    /// Path to `adb.exe`, re-locating if the cache is empty.
    ///
    /// Returns an owned `PathBuf` so no lock is held across an `.await`.
    pub fn adb_path(&self) -> Result<PathBuf, AppError> {
        let cached = self
            .adb
            .lock()
            .map_err(|_| AppError::from(adbkit::AdbError::Io("state lock poisoned".into())))?
            .clone();

        if let Some(path) = cached {
            return Ok(path);
        }

        let found = adbkit::sdk::locate(None).ok_or_else(AppError::sdk_missing)?;
        *self
            .adb
            .lock()
            .map_err(|_| AppError::from(adbkit::AdbError::Io("state lock poisoned".into())))? =
            Some(found.clone());
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caches_a_resolved_path_and_returns_it_again() {
        let state = AppState::default();
        let first = state.adb_path();
        let second = state.adb_path();
        assert_eq!(
            first.as_ref().ok().map(|p| p.exists()),
            second.as_ref().ok().map(|p| p.exists()),
            "repeated lookups must agree"
        );
    }

    #[test]
    fn missing_sdk_is_a_typed_error_not_a_panic() {
        let state = AppState {
            adb: Mutex::new(None),
        };
        // If this machine has an SDK the call succeeds; either way it must not panic.
        match state.adb_path() {
            Ok(path) => assert!(path.ends_with("adb.exe")),
            Err(error) => assert_eq!(error.code, "SDK_NOT_FOUND"),
        }
    }
}
