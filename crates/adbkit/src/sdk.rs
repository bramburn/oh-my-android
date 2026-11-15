//! Locate the user's Android SDK — bring-your-own, nothing bundled (`docs/legal.md` §2 option (a)).
//!
//! Port of `Sources/Core/Android/AndroidSDK.swift:13-36`, adapted to Windows conventions. Upstream's
//! "saved choice first" ordering idea is kept as `custom_root`; persistence is deliberately absent in the
//! MVP, so callers pass the saved choice (if any) each time.

use std::path::{Path, PathBuf};

/// Directory that must contain `platform-tools\adb.exe`.
const ADB_RELATIVE: &str = r"platform-tools\adb.exe";

/// Find `adb.exe`, in the order a Windows user would expect.
///
/// 1. `custom_root` (a saved choice — MVP: always `None`)
/// 2. `ANDROID_HOME`
/// 3. `ANDROID_SDK_ROOT`
/// 4. `%LOCALAPPDATA%\Android\Sdk` (Android Studio's default)
/// 5. `%ProgramFiles(x86)%\Android\android-sdk` (legacy installer default)
pub fn locate(custom_root: Option<&Path>) -> Option<PathBuf> {
    let env = |key: &str| std::env::var(key).ok();
    let local_appdata = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files_x86 = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    locate_with(
        custom_root,
        &env,
        local_appdata.as_deref(),
        program_files_x86.as_deref(),
    )
}

/// Testable core of [`locate`]: environment access is injected so no test touches the real machine.
pub fn locate_with(
    custom_root: Option<&Path>,
    env: &dyn Fn(&str) -> Option<String>,
    local_appdata: Option<&Path>,
    program_files_x86: Option<&Path>,
) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::with_capacity(5);
    if let Some(root) = custom_root {
        candidates.push(root.to_path_buf());
    }
    for key in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Some(value) = env(key).filter(|v| !v.trim().is_empty()) {
            candidates.push(PathBuf::from(value));
        }
    }
    if let Some(base) = local_appdata {
        candidates.push(base.join("Android").join("Sdk"));
    }
    if let Some(base) = program_files_x86 {
        candidates.push(base.join("Android").join("android-sdk"));
    }

    candidates
        .into_iter()
        .map(|root| root.join(ADB_RELATIVE))
        .find(|adb| adb.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Creates `<base>/<n>/platform-tools/adb.exe` and returns the SDK root.
    fn fake_sdk(base: &Path, name: &str) -> PathBuf {
        let root = base.join(name);
        fs::create_dir_all(root.join("platform-tools")).unwrap();
        fs::write(root.join("platform-tools").join("adb.exe"), b"stub").unwrap();
        root
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("adbkit-sdk-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn finds_adb_in_a_custom_root_first() {
        let base = scratch("custom");
        let custom = fake_sdk(&base, "custom");
        let android_home = fake_sdk(&base, "home");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| android_home.display().to_string());

        let found = locate_with(Some(&custom), &env, None, None).unwrap();
        assert!(found.starts_with(&custom));
        assert!(found.ends_with("adb.exe"));
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn falls_back_to_android_home() {
        let base = scratch("home-only");
        let android_home = fake_sdk(&base, "home");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| android_home.display().to_string());

        assert!(locate_with(None, &env, None, None).is_some_and(|p| p.starts_with(&android_home)));
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn falls_back_to_local_appdata_default_layout() {
        let base = scratch("localappdata");
        let sdk = fake_sdk(&base, "Android/Sdk");
        let env = no_env;

        let found = locate_with(None, &env, Some(&base), None).unwrap();
        assert!(found.starts_with(&sdk), "got {}", found.display());
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn empty_env_vars_are_ignored() {
        let base = scratch("empty-env");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| "   ".to_string());
        assert!(locate_with(None, &env, None, None).is_none());
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn sdk_root_without_adb_is_rejected() {
        let base = scratch("no-adb");
        fs::create_dir_all(base.join("platform-tools")).unwrap();
        // platform-tools exists, adb.exe does not.
        assert!(locate_with(Some(&base), &no_env, None, None).is_none());
        fs::remove_dir_all(base).ok();
    }
}
