//! Parsers for `adb` output. Pure functions only — no processes, no I/O, fully unit-testable.
//!
//! Port of `Sources/Core/Android/ADBClient.swift:98-104` (`device(fromListLine:)`) and
//! `Sources/Core/Android/Device.swift:4-33`.

use serde::Serialize;

/// One connected Android target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub serial: String,
    pub model: String,
    pub state: DeviceState,
}

/// adb connection state. Mirrors `Device.State` (`Sources/Core/Android/Device.swift:9-22`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceState {
    Device,
    Unauthorized,
    Offline,
    Other,
}

impl DeviceState {
    fn parse(raw: &str) -> Self {
        match raw {
            "device" => Self::Device,
            "unauthorized" => Self::Unauthorized,
            "offline" => Self::Offline,
            _ => Self::Other,
        }
    }
}

/// Parse the output of `adb devices -l`.
///
/// Input looks like:
/// ```text
/// List of devices attached
/// emulator-5554          device product:sdk_gphone64 model:sdk_gphone64_arm64 device:emu64a transport_id:1
/// R58M1234ABC            unauthorized usb:1-1 transport_id:2
/// * daemon started successfully
/// ```
///
/// Rules, all copied from upstream (`ADBClient.swift:98-104`):
/// - skip the `List of devices` header and adb's own `*` notes;
/// - a device line needs at least two whitespace-separated fields;
/// - `model:` wins, with `_` replaced by a space; otherwise the serial is the display name;
/// - unknown states map to [`DeviceState::Other`] rather than failing.
///
/// Windows extra rule (`AGENTS.md` §12): trailing `\r` is trimmed, because `adb` on Windows emits CRLF.
pub fn parse_devices(stdout: &str) -> Vec<Device> {
    stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r').trim();
            if line.is_empty() || line.starts_with("List of devices") || line.starts_with('*') {
                return None;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let model = parts
                .iter()
                .find_map(|part| part.strip_prefix("model:"))
                .map(|model| model.replace('_', " "))
                .unwrap_or_else(|| parts[0].to_string());
            Some(Device {
                serial: parts[0].to_string(),
                model,
                state: DeviceState::parse(parts[1]),
            })
        })
        .collect()
}

/// First line of `adb version`, trimmed — e.g. `Android Debug Bridge version 1.0.41`.
///
/// `adb version` is client-local: it prints the version and exits **without** starting the adb server, which
/// is why it is the only command the setup screen may run (ADR-0007 Amendment 2 §1).
pub fn parse_adb_version(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(|line| line.trim_end_matches('\r').trim())
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod version_tests {
    use super::parse_adb_version;

    #[test]
    fn takes_the_first_line_and_strips_crlf() {
        let raw = "Android Debug Bridge version 1.0.41\r\nVersion 37.0.1-15733141\r\n";
        assert_eq!(
            parse_adb_version(raw).as_deref(),
            Some("Android Debug Bridge version 1.0.41")
        );
    }

    #[test]
    fn skips_leading_blank_lines() {
        assert_eq!(
            parse_adb_version("\n\nAndroid Debug Bridge version 1.0.41\n").as_deref(),
            Some("Android Debug Bridge version 1.0.41")
        );
    }

    #[test]
    fn empty_output_is_none() {
        assert!(parse_adb_version(" \r\n\n").is_none());
    }
}
