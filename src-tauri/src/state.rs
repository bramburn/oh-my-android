//! Shared application state.
//!
//! Holds the loaded [`Config`] and a lazily-resolved `adb.exe` cache. The cache is invalidated whenever the
//! user picks a different SDK folder, and a negative result is never sticky — a user can install the SDK
//! while the app is running and simply press Re-check.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;

/// Everything mutable that commands need. One mutex: these are tiny fields and every borrow is short.
struct Inner {
    config: Config,
    /// Resolved `adb.exe` for the *current* config. Cleared on any config change.
    adb: Option<PathBuf>,
    /// Non-fatal problem from loading the config (corrupt file), surfaced in `SetupReport`.
    warning: Option<String>,
}

pub struct AppState {
    inner: Mutex<Inner>,
    config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: PathBuf, config: Config, warning: Option<String>) -> Self {
        Self {
            inner: Mutex::new(Inner {
                config,
                adb: None,
                warning,
            }),
            config_dir,
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, AppError> {
        self.inner
            .lock()
            .map_err(|_| AppError::from(adbkit::AdbError::Io("state lock poisoned".into())))
    }

    /// The user's saved SDK root, if any.
    pub fn custom_root(&self) -> Option<PathBuf> {
        self.lock()
            .ok()
            .and_then(|inner| inner.config.sdk.custom_root.clone())
            .map(PathBuf::from)
    }

    /// Warning to show about the config file, if it had to be reset.
    pub fn warning(&self) -> Option<String> {
        self.lock().ok().and_then(|inner| inner.warning.clone())
    }

    /// Path to `adb.exe`: saved choice first via `adbkit::sdk`, else env/defaults.
    ///
    /// Returns an owned `PathBuf` so no lock is held across an `.await`.
    pub fn adb_path(&self) -> Result<PathBuf, AppError> {
        let (cached, custom) = {
            let inner = self.lock()?;
            (inner.adb.clone(), inner.config.sdk.custom_root.clone())
        };

        if let Some(path) = cached {
            return Ok(path);
        }

        let found = adbkit::sdk::locate(custom.as_deref().map(Path::new))
            .ok_or_else(AppError::sdk_missing)?;

        let mut inner = self.lock()?;
        inner.adb = Some(found.adb.clone());
        Ok(found.adb)
    }

    /// Persist a new SDK choice and invalidate the cache. `None` clears the choice.
    pub fn set_custom_root(&self, root: Option<PathBuf>) -> Result<(), AppError> {
        let mut inner = self.lock()?;
        inner.config.sdk.custom_root = root.map(|p| p.display().to_string());
        inner.adb = None;
        inner
            .config
            .save(&self.config_dir)
            .map_err(|e| AppError::config_write(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(name: &str) -> (AppState, PathBuf) {
        let dir = std::env::temp_dir().join(format!("oma-state-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        (AppState::new(dir.clone(), Config::default(), None), dir)
    }

    #[test]
    fn a_saved_custom_root_is_persisted_and_cached_cleared() {
        let (state, dir) = state("persist");
        let fake = dir.join("Sdk");
        std::fs::create_dir_all(fake.join("platform-tools")).unwrap();
        std::fs::write(fake.join("platform-tools").join("adb.exe"), b"stub").unwrap();

        state.set_custom_root(Some(fake.clone())).unwrap();
        assert_eq!(state.custom_root(), Some(fake.clone()));

        let (reloaded, warning) = Config::load(&dir);
        assert!(warning.is_none());
        assert_eq!(
            reloaded.sdk.custom_root.as_deref(),
            Some(fake.display().to_string().as_str())
        );

        // The saved root must win over the real machine's SDK during resolution.
        let resolved = state.adb_path().unwrap();
        assert!(resolved.starts_with(&fake), "got {}", resolved.display());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn warning_is_kept_for_the_setup_report() {
        let dir = std::env::temp_dir().join(format!("oma-state-warn-{}", std::process::id()));
        let state = AppState::new(dir, Config::default(), Some("resetted".into()));
        assert_eq!(state.warning().as_deref(), Some("resetted"));
    }
}
