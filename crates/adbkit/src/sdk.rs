//! Locate the user's Android SDK — bring-your-own, nothing bundled (`docs/legal.md` §2 option (a)).
//!
//! Port of `Sources/Core/Android/AndroidSDK.swift:13-36`, adapted to Windows conventions. Upstream's
//! "saved choice first" ordering is now persisted (`sdk.customRoot`, override O3), and the function reports
//! **which rule matched** so the setup screen can say why it believes what it believes.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Directory that must contain `platform-tools\adb.exe`.
pub const ADB_RELATIVE: &str = r"platform-tools\adb.exe";

/// Why a location was chosen. Shown verbatim in the setup screen ("found via ANDROID_HOME").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SdkSource {
    /// The folder the user picked (persisted as `sdk.customRoot`).
    Custom,
    /// `ANDROID_HOME`
    AndroidHome,
    /// `ANDROID_SDK_ROOT`
    AndroidSdkRoot,
    /// `%LOCALAPPDATA%\Android\Sdk` — Android Studio's default on Windows.
    LocalAppDataDefault,
    /// `%ProgramFiles(x86)%\Android\android-sdk` — legacy standalone installer.
    ProgramFilesLegacy,
}

impl SdkSource {
    /// Stable, human-facing label. The UI never has to know the enum.
    pub fn label(self) -> &'static str {
        match self {
            SdkSource::Custom => "chosen folder",
            SdkSource::AndroidHome => "ANDROID_HOME",
            SdkSource::AndroidSdkRoot => "ANDROID_SDK_ROOT",
            SdkSource::LocalAppDataDefault => "Android Studio default folder",
            SdkSource::ProgramFilesLegacy => "legacy installer folder",
        }
    }
}

/// A validated SDK root: the directory *containing* `platform-tools\`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkLocation {
    /// SDK root, e.g. `C:\Users\me\AppData\Local\Android\Sdk`.
    #[serde(serialize_with = "crate::serde_path::serialize")]
    pub root: PathBuf,
    /// Full path to `adb.exe`.
    #[serde(serialize_with = "crate::serde_path::serialize")]
    pub adb: PathBuf,
    /// Which rule matched.
    pub source: SdkSource,
}

impl SdkLocation {
    /// Human-readable source, for the setup screen.
    pub fn source_label(&self) -> &'static str {
        self.source.label()
    }
}

/// Validate a directory as an SDK root. `None` when `platform-tools\adb.exe` is not there.
///
/// This is the **only** definition of "valid SDK" in the app: the folder picker and the locator both use it,
/// so a folder that the picker accepts can never fail discovery later.
pub fn validate_root(root: &Path, source: SdkSource) -> Option<SdkLocation> {
    let adb = root.join(ADB_RELATIVE);
    adb.is_file().then(|| SdkLocation {
        root: root.to_path_buf(),
        adb,
        source,
    })
}

/// Find the SDK with the real process environment.
pub fn locate(custom_root: Option<&Path>) -> Option<SdkLocation> {
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
///
/// Order (`AndroidSDK.swift:13-36`):
/// 1. `custom_root` — the user's saved choice
/// 2. `ANDROID_HOME`
/// 3. `ANDROID_SDK_ROOT`
/// 4. `%LOCALAPPDATA%\Android\Sdk`
/// 5. `%ProgramFiles(x86)%\Android\android-sdk`
pub fn locate_with(
    custom_root: Option<&Path>,
    env: &dyn Fn(&str) -> Option<String>,
    local_appdata: Option<&Path>,
    program_files_x86: Option<&Path>,
) -> Option<SdkLocation> {
    let mut candidates: Vec<(PathBuf, SdkSource)> = Vec::with_capacity(5);
    if let Some(root) = custom_root {
        candidates.push((root.to_path_buf(), SdkSource::Custom));
    }
    for (key, source) in [
        ("ANDROID_HOME", SdkSource::AndroidHome),
        ("ANDROID_SDK_ROOT", SdkSource::AndroidSdkRoot),
    ] {
        if let Some(value) = env(key).filter(|v| !v.trim().is_empty()) {
            candidates.push((PathBuf::from(value), source));
        }
    }
    if let Some(base) = local_appdata {
        candidates.push((
            base.join("Android").join("Sdk"),
            SdkSource::LocalAppDataDefault,
        ));
    }
    if let Some(base) = program_files_x86 {
        candidates.push((
            base.join("Android").join("android-sdk"),
            SdkSource::ProgramFilesLegacy,
        ));
    }

    candidates
        .into_iter()
        .find_map(|(root, source)| validate_root(&root, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Creates `<base>/<name>/platform-tools/adb.exe` and returns the SDK root.
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
        assert_eq!(found.source, SdkSource::Custom);
        assert!(found.adb.starts_with(&custom));
        assert!(found.adb.ends_with("adb.exe"));
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn falls_back_to_android_home_and_reports_the_source() {
        let base = scratch("home-only");
        let android_home = fake_sdk(&base, "home");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| android_home.display().to_string());

        let found = locate_with(None, &env, None, None).unwrap();
        assert_eq!(found.source, SdkSource::AndroidHome);
        assert!(found.adb.starts_with(&android_home));
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn android_sdk_root_is_third() {
        let base = scratch("sdk-root");
        let root = fake_sdk(&base, "sdkroot");
        let env = |k: &str| (k == "ANDROID_SDK_ROOT").then(|| root.display().to_string());

        let found = locate_with(None, &env, None, None).unwrap();
        assert_eq!(found.source, SdkSource::AndroidSdkRoot);
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn falls_back_to_local_appdata_default_layout() {
        let base = scratch("localappdata");
        let sdk = fake_sdk(&base, "Android/Sdk");

        let found = locate_with(None, &no_env, Some(&base), None).unwrap();
        assert_eq!(found.source, SdkSource::LocalAppDataDefault);
        assert!(found.adb.starts_with(&sdk), "got {}", found.adb.display());
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn empty_env_vars_are_ignored() {
        let base = scratch("empty-env");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| " ".to_string());
        assert!(locate_with(None, &env, None, None).is_none());
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn sdk_root_without_adb_is_rejected() {
        let base = scratch("no-adb");
        fs::create_dir_all(base.join("platform-tools")).unwrap();
        // platform-tools exists, adb.exe does not.
        assert!(validate_root(&base, SdkSource::Custom).is_none());
        assert!(locate_with(Some(&base), &no_env, None, None).is_none());
        fs::remove_dir_all(base).ok();
    }

    #[test]
    fn a_stale_custom_root_falls_through_to_the_next_rule() {
        let base = scratch("stale-custom");
        let home = fake_sdk(&base, "home");
        let env = |k: &str| (k == "ANDROID_HOME").then(|| home.display().to_string());
        let missing = base.join("unplugged-drive");

        let found = locate_with(Some(&missing), &env, None, None).unwrap();
        assert_eq!(found.source, SdkSource::AndroidHome);
        fs::remove_dir_all(base).ok();
    }
}
