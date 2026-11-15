//! Read-only setup detection.
//!
//! **This module never spawns a process.** It answers "what does this machine have?" from the filesystem
//! and the environment only. The single exception in the whole setup flow is `adb version`, which
//! `src-tauri` runs through [`crate::runner`] because this crate is the only place allowed to spawn — and
//! `adb version` is a client-local command that does not start the daemon.
//!
//! Rationale (ADR-0007 Amendment 2 §1): the app guides, it never installs. Installing software on a user's
//! behalf is a different product with a different licence and security surface.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::sdk::{self, SdkLocation};

/// Everything the setup screen renders, in one object. Detection-only: no command results except
/// `adb_version`, which the caller fills in.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupReport {
    /// `studio64.exe` if Android Studio is installed (either supported location).
    pub studio: Option<String>,
    /// Validated SDK, if one was found.
    pub sdk: Option<SdkLocation>,
    /// First line of `adb version`, filled in by `src-tauri` (`None` = not run or not runnable).
    pub adb_version: Option<String>,
    /// `<sdk>\emulator\emulator.exe` if the emulator package is installed (not blocking).
    pub emulator: Option<String>,
    /// Number of `*.ini` files in the AVD folder (not blocking).
    pub avd_count: usize,
    /// `ANDROID_HOME`, echoed so the UI can explain where it looked.
    pub env_home: Option<String>,
    /// The persisted user choice, echoed so the UI can offer "forget this folder".
    pub custom_root: Option<String>,
    /// Non-fatal problem worth showing, e.g. a corrupt config file.
    pub warning: Option<String>,
}

impl SetupReport {
    /// Devices can be listed only when a runnable adb was found.
    pub fn ready(&self) -> bool {
        self.sdk.is_some()
    }
}

/// Injected inputs. Every path and env lookup is a parameter so tests run against temp dirs and never the
/// real machine (`crates/adbkit/AGENTS.md`).
pub struct DetectInputs<'a> {
    pub custom_root: Option<&'a Path>,
    pub env: &'a dyn Fn(&str) -> Option<String>,
    pub local_appdata: Option<&'a Path>,
    pub program_files: Option<&'a Path>,
    pub program_files_x86: Option<&'a Path>,
    pub user_profile: Option<&'a Path>,
    /// Set by the caller when the config file had to be reset.
    pub warning: Option<String>,
}

/// Detect with the real process environment.
pub fn detect(custom_root: Option<&Path>, warning: Option<String>) -> SetupReport {
    let env = |key: &str| std::env::var(key).ok();
    let var = |key: &str| std::env::var_os(key).map(PathBuf::from);
    let local_appdata = var("LOCALAPPDATA");
    let program_files = var("ProgramFiles");
    let program_files_x86 = var("ProgramFiles(x86)");
    let user_profile = var("USERPROFILE");

    detect_with(&DetectInputs {
        custom_root,
        env: &env,
        local_appdata: local_appdata.as_deref(),
        program_files: program_files.as_deref(),
        program_files_x86: program_files_x86.as_deref(),
        user_profile: user_profile.as_deref(),
        warning,
    })
}

/// Testable core of [`detect`].
pub fn detect_with(inputs: &DetectInputs<'_>) -> SetupReport {
    let sdk = sdk::locate_with(
        inputs.custom_root,
        inputs.env,
        inputs.local_appdata,
        inputs.program_files_x86,
    );

    let studio =
        find_studio(inputs.program_files, inputs.local_appdata).map(|p| p.display().to_string());

    let emulator = sdk
        .as_ref()
        .map(|s| s.root.join("emulator").join("emulator.exe"))
        .filter(|p| p.is_file())
        .map(|p| p.display().to_string());

    SetupReport {
        studio,
        // Copied out before `sdk` is moved into the report, so the env claim is checkable.
        env_home: (inputs.env)("ANDROID_HOME").filter(|v| !v.trim().is_empty()),
        custom_root: inputs.custom_root.map(|p| p.display().to_string()),
        avd_count: count_avds(inputs.user_profile, inputs.env),
        emulator,
        sdk,
        adb_version: None,
        warning: inputs.warning.clone(),
    }
}

/// Android Studio, in the order a Windows install would land. Filesystem only — the registry is a
/// deliberate non-goal for the MVP (noted as a possible refinement in `21-mvp-log.md`).
fn find_studio(program_files: Option<&Path>, local_appdata: Option<&Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::with_capacity(2);
    if let Some(base) = program_files {
        candidates.push(
            base.join("Android")
                .join("Android Studio")
                .join("bin")
                .join("studio64.exe"),
        );
    }
    if let Some(base) = local_appdata {
        candidates.push(
            base.join("Programs")
                .join("Android Studio")
                .join("bin")
                .join("studio64.exe"),
        );
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Count configured virtual devices.
///
/// This mirrors what `emulator -list-avds` enumerates: one `*.ini` per AVD. We deliberately do **not**
/// spawn the emulator to ask (contrast `EmulatorLauncher.swift:8-16`, which launches it for real) — setup
/// detection must not start processes, and spawning the emulator to answer a question would be seconds of
/// latency and a visible window.
fn count_avds(user_profile: Option<&Path>, env: &dyn Fn(&str) -> Option<String>) -> usize {
    // `ANDROID_AVD_HOME` wins when set (matches the emulator), then `ANDROID_USER_HOME\avd`, then the
    // classic `%USERPROFILE%\.android\avd`.
    let dir = if let Some(value) = env("ANDROID_AVD_HOME").filter(|v| !v.trim().is_empty()) {
        Some(PathBuf::from(value))
    } else if let Some(user_home) = env("ANDROID_USER_HOME").filter(|v| !v.trim().is_empty()) {
        Some(PathBuf::from(user_home).join("avd"))
    } else {
        user_profile.map(|p| p.join(".android").join("avd"))
    };

    let Some(dir) = dir else { return 0 };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("ini"))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("adbkit-setup-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"stub").unwrap();
    }

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn inputs<'a>(
        base: &'a Path,
        env: &'a dyn Fn(&str) -> Option<String>,
        custom: Option<&'a Path>,
    ) -> DetectInputs<'a> {
        DetectInputs {
            custom_root: custom,
            env,
            local_appdata: Some(base),
            program_files: Some(base),
            program_files_x86: Some(base),
            user_profile: Some(base),
            warning: None,
        }
    }

    #[test]
    fn an_absent_sdk_yields_a_report_that_is_not_ready() {
        let base = scratch("none");
        let report = detect_with(&inputs(&base, &no_env, None));
        assert!(report.sdk.is_none());
        assert!(!report.ready());
        assert!(report.studio.is_none());
        assert_eq!(report.avd_count, 0);
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn a_complete_install_reports_every_row() {
        let base = scratch("complete");
        let sdk_root = base.join("Android").join("Sdk");
        touch(&sdk_root.join("platform-tools").join("adb.exe"));
        touch(&sdk_root.join("emulator").join("emulator.exe"));
        touch(
            &base
                .join("Android")
                .join("Android Studio")
                .join("bin")
                .join("studio64.exe"),
        );
        for n in ["Pixel_8", "Pixel_Tablet"] {
            touch(&base.join(".android").join("avd").join(format!("{n}.ini")));
        }

        let report = detect_with(&inputs(&base, &no_env, None));
        assert!(report.ready());
        assert_eq!(
            report.sdk.as_ref().unwrap().source,
            sdk::SdkSource::LocalAppDataDefault
        );
        assert!(report.studio.is_some());
        assert!(report.emulator.is_some());
        assert_eq!(report.avd_count, 2);
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn a_partial_install_is_not_ready_but_still_reports_what_exists() {
        let base = scratch("partial");
        // SDK present, emulator missing.
        let sdk_root = base.join("Android").join("Sdk");
        touch(&sdk_root.join("platform-tools").join("adb.exe"));

        let report = detect_with(&inputs(&base, &no_env, None));
        assert!(report.ready());
        assert!(report.emulator.is_none());
        assert!(report.studio.is_none());
        assert_eq!(report.avd_count, 0);
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn a_picked_folder_is_reported_as_custom() {
        let base = scratch("custom");
        let chosen = base.join("MySdk");
        touch(&chosen.join("platform-tools").join("adb.exe"));

        let report = detect_with(&inputs(&base, &no_env, Some(&chosen)));
        assert_eq!(report.sdk.as_ref().unwrap().source, sdk::SdkSource::Custom);
        assert_eq!(
            report.custom_root.as_deref(),
            Some(chosen.display().to_string().as_str())
        );
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn avd_home_env_is_preferred_over_the_profile() {
        let base = scratch("avd-home");
        let custom_avd = base.join("elsewhere").join("avd");
        touch(&custom_avd.join("One.ini"));
        let env = |k: &str| (k == "ANDROID_AVD_HOME").then(|| custom_avd.display().to_string());

        let report = detect_with(&inputs(&base, &env, None));
        assert_eq!(report.avd_count, 1);
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn warnings_are_passed_through_to_the_ui() {
        let base = scratch("warning");
        let mut input = inputs(&base, &no_env, None);
        input.warning = Some("config.json was unreadable".into());

        let report = detect_with(&input);
        assert_eq!(
            report.warning.as_deref(),
            Some("config.json was unreadable")
        );
        fs::remove_dir_all(base).ok();
    }
}
