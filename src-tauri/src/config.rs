//! The versioned, atomic config file — the only thing this app persists (override O3).
//!
//! Path: `%APPDATA%\dev.bramburn.androidpanel\config.json` (Tauri's `app_config_dir`).
//!
//! Rules, in order of importance:
//! 1. **Never crash on a bad file.** Corrupt or unreadable ⇒ copy it to `config.json.bak`, return defaults,
//!    and hand the caller a warning string to show in the UI.
//! 2. **Never silently discard the user's only setting.** The `.bak` is the escape hatch.
//! 3. **Atomic writes.** Temp file in the same directory, then `rename` (same volume ⇒ atomic on NTFS).
//! 4. **Versioned from day one.** A future schema bump must ship a numbered migration + tests; migrations
//!    are executed only in the main repo after merge-back, with human approval
//!    (`docs/windows-port/21-mvp-log.md` §4).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Current on-disk schema. Bump only with a migration arm in [`migrate`] and tests.
pub const SCHEMA_VERSION: u32 = 1;

/// File name inside the app config directory.
const FILE_NAME: &str = "config.json";

/// The whole persisted document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// On-disk schema version. Missing ⇒ 0, which [`migrate`] treats as unknown.
    pub schema_version: u32,
    /// SDK-related settings.
    #[serde(default)]
    pub sdk: SdkConfig,
}

/// SDK settings. `custom_root` is an absolute **SDK root** (contains `platform-tools\`), never `adb.exe`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkConfig {
    /// User-chosen SDK folder. `None` = fall through to env vars and default locations.
    #[serde(default)]
    pub custom_root: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            sdk: SdkConfig::default(),
        }
    }
}

impl Config {
    /// Path of the config file inside `dir`.
    pub fn path_in(dir: &Path) -> PathBuf {
        dir.join(FILE_NAME)
    }

    /// Read the config, or return defaults plus a human-readable warning.
    ///
    /// A missing file is **not** a warning: it is the first-run state.
    pub fn load(dir: &Path) -> (Self, Option<String>) {
        let path = Self::path_in(dir);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return (Self::default(), None)
            }
            Err(error) => {
                return (
                    Self::default(),
                    Some(format!(
                        "Settings could not be read ({}). Using defaults.",
                        error.kind()
                    )),
                )
            }
        };

        match serde_json::from_str::<Config>(&raw)
            .map_err(|e| e.to_string())
            .and_then(migrate)
        {
            Ok(config) => (config, None),
            Err(reason) => {
                // Keep the user's bytes, then start clean. `best_effort` because a failure to back up
                // must not stop the app from starting.
                let backup = dir.join(format!("{FILE_NAME}.bak"));
                let _ = std::fs::copy(&path, &backup);
                (
                    Self::default(),
                    Some(format!(
                        "Settings were reset (saved a backup as config.json.bak): {reason}"
                    )),
                )
            }
        }
    }

    /// Atomic save: temp file in the same directory, then rename over the target.
    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let target = Self::path_in(dir);
        let temp = dir.join(format!("{FILE_NAME}.tmp"));
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&temp, json)?;
        std::fs::rename(&temp, &target)
    }
}

/// Schema upgrade hook. **Schema 1 is current and has no migrations.**
///
/// Every future arm goes here, with tests, and is executed only after merge-back (see module docs).
fn migrate(config: Config) -> Result<Config, String> {
    match config.schema_version {
        SCHEMA_VERSION => Ok(config),
        0 => Err("config has no schemaVersion".to_string()),
        other => Err(format!("unsupported schemaVersion {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("oma-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_missing_file_is_the_first_run_state_not_a_warning() {
        let dir = scratch("missing");
        let (config, warning) = Config::load(&dir);
        assert_eq!(config, Config::default());
        assert!(warning.is_none());
        assert!(
            !Config::path_in(&dir).exists(),
            "load must not create the file"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn save_then_load_round_trips_the_custom_root() {
        let dir = scratch("roundtrip");
        let mut config = Config::default();
        config.sdk.custom_root = Some(r"D:\Android\Sdk".into());
        config.save(&dir).unwrap();

        let (loaded, warning) = Config::load(&dir);
        assert!(warning.is_none());
        assert_eq!(loaded.sdk.custom_root.as_deref(), Some(r"D:\Android\Sdk"));
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn the_file_on_disk_uses_the_documented_shape() {
        let dir = scratch("shape");
        Config::default().save(&dir).unwrap();
        let raw = fs::read_to_string(Config::path_in(&dir)).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["schemaVersion"], 1);
        assert!(value["sdk"].is_object());
        assert!(value["sdk"]["customRoot"].is_null());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_corrupt_file_is_backed_up_and_replaced_by_defaults() {
        let dir = scratch("corrupt");
        fs::write(Config::path_in(&dir), b"{ not json").unwrap();

        let (config, warning) = Config::load(&dir);
        assert_eq!(config, Config::default());
        assert!(warning.unwrap().contains("config.json.bak"));
        assert_eq!(
            fs::read_to_string(dir.join("config.json.bak")).unwrap(),
            "{ not json"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn an_unknown_schema_version_is_treated_as_corrupt_not_guessed() {
        let dir = scratch("future");
        fs::write(
            Config::path_in(&dir),
            br#"{"schemaVersion": 99, "sdk": {"customRoot": "C:\\x"}}"#,
        )
        .unwrap();

        let (config, warning) = Config::load(&dir);
        assert_eq!(config, Config::default(), "must not adopt a future schema");
        assert!(warning.unwrap().contains("schemaVersion 99"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_missing_schema_version_is_rejected() {
        let dir = scratch("no-version");
        fs::write(Config::path_in(&dir), br#"{"sdk": {"customRoot": null}}"#).unwrap();
        let (config, _) = Config::load(&dir);
        assert_eq!(config, Config::default());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn saving_leaves_no_temp_file_behind() {
        let dir = scratch("temp");
        Config::default().save(&dir).unwrap();
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|name| name.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "left behind {leftovers:?}");
        fs::remove_dir_all(dir).ok();
    }
}
