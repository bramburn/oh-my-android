//! Golden-file tests for `adb devices -l`.
//!
//! Fixtures are captured verbatim and stored with their original line endings
//! (`.gitattributes` marks them `-text` so CRLF survives checkout).
//! Rule (`docs/windows-port/17-testing-strategy.md` §3): never hand-write a fixture from memory.

use adbkit::{parse_devices, Device, DeviceState};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn device(serial: &str, model: &str, state: DeviceState) -> Device {
    Device {
        serial: serial.to_string(),
        model: model.to_string(),
        state,
    }
}

#[test]
fn basic_emulator_and_usb_device() {
    let got = parse_devices(&fixture("devices_l_basic.txt"));
    assert_eq!(
        got,
        vec![
            device("emulator-5554", "sdk gphone64 arm64", DeviceState::Device),
            device("R58M1234ABC", "SM G973F", DeviceState::Device),
        ]
    );
}

#[test]
fn mixed_states_notes_blank_lines_and_crlf() {
    let raw = fixture("devices_l_mixed.txt");
    assert!(
        raw.contains('\r'),
        "fixture must keep CRLF to test the Windows rule"
    );

    let got = parse_devices(&raw);
    assert_eq!(
        got,
        vec![
            device("R58M1234ABC", "R58M1234ABC", DeviceState::Unauthorized),
            device("emulator-5554", "emulator-5554", DeviceState::Offline),
            device("192.168.1.42:5555", "Pixel 5", DeviceState::Device),
            device("R58M9999XYZ", "R58M9999XYZ", DeviceState::Other),
        ]
    );
}

#[test]
fn empty_list_yields_no_devices() {
    assert!(parse_devices(&fixture("devices_l_empty.txt")).is_empty());
}

#[test]
fn model_underscores_become_spaces() {
    let got = parse_devices("abc device model:sdk_gphone64_arm64 transport_id:1\n");
    assert_eq!(got[0].model, "sdk gphone64 arm64");
}

#[test]
fn missing_model_falls_back_to_serial() {
    let got = parse_devices("abc device transport_id:1\n");
    assert_eq!(got[0].model, "abc");
}

#[test]
fn unknown_state_is_other() {
    let got = parse_devices("abc bootloader transport_id:1\n");
    assert_eq!(got[0].state, DeviceState::Other);
}

#[test]
fn single_field_line_is_ignored() {
    assert!(parse_devices("abc\n").is_empty());
}

#[test]
fn serializes_to_camel_case_json_for_the_ui() {
    let json = serde_json::to_string(&parse_devices("abc device model:Pixel_5\n")[0]).unwrap();
    assert_eq!(
        json,
        r#"{"serial":"abc","model":"Pixel 5","state":"device"}"#
    );
}
