# 06 — ADB integration

The single most important document. Every adb/scrcpy/`app_process`/5037/`localabstract` hit from the
requirement grep is accounted for here.

```sh
rg -n "adb|scrcpy|app_process|5037|localabstract" Sources
```

Result: ~90 hits. `scrcpy|app_process|localabstract` → **0 matches in code**. Mirroring does not exist
(see `07-mirroring-and-control.md`).

## 0. How adb is invoked, in one paragraph

All device work goes through `ADBClient` (`Sources/Core/Android/ADBClient.swift:4-15`). Its single
implementation `AndroidDebugBridge` builds argv and hands it to `ProcessShellRunner`
(`Sources/Core/Shell/ShellRunner.swift:32-113`), which spawns `<sdk>/platform-tools/adb` with
`stdin = /dev/null`, drains stdout/stderr, enforces a 60 s timeout and terminates on cancellation.
`adb start-server` is the one command run **detached with no pipes**, because a piped client that
implicitly starts the server would let the daemon inherit our pipe and hang us
(`Sources/Core/Android/ADBClient.swift:43-47`, `Sources/Core/Shell/ShellRunner.swift:58-67`, `:123-131`).
The only non-process adb access is a raw TCP socket to the adb server for `host:track-devices`
(`Sources/Core/Android/DeviceTracker.swift:96-118`).

## 1. Binary discovery and version

| Aspect | Behaviour | Evidence |
|---|---|---|
| Candidate roots, in order | `sdk.customRoot` (UserDefaults) → `ANDROID_HOME` → `ANDROID_SDK_ROOT` → `~/Library/Android/sdk` → `/opt/homebrew/share/android-commandlinetools` → `/usr/local/share/android-commandlinetools` → `/opt/homebrew/share/android-sdk` → `/usr/local/share/android-sdk` | `Sources/Core/Android/AndroidSDK.swift:13-32` |
| Validity test | `<root>/platform-tools/adb` must be **executable**; no `adb version` probe | `Sources/Core/Android/AndroidSDK.swift:9`, `:34-36` |
| No SDK | Panel shows "Android SDK Not Found" + a Settings link; MCP tools return a message | `Sources/State/AppModel.swift:47-70`, `Sources/MCP/Tools/ToolEnvironment.swift:20-26` |
| Version pinning | none; any adb that answers is used | — |
| Server port | `ANDROID_ADB_SERVER_PORT` respected for the tracking socket only; CLI calls inherit adb's own default | `Sources/Core/Android/DeviceTracker.swift:29-31` |

`adb version` itself is never run. **No version check exists** — an ancient `adb` fails at command level
with a `ShellError` toast.

## 2. Server lifecycle

| Action | Command | Trigger | Evidence |
|---|---|---|---|
| Ensure running (app) | `adb start-server`, detached, before the tracker connects | app launch | `Sources/State/AppModel.swift:44`, `Sources/State/DeviceStore.swift:29-39` |
| Ensure running (MCP) | same, exactly once per process | first tool call | `Sources/MCP/Tools/ToolEnvironment.swift:64-72` |
| Ensure running (tracker) | same, before each reconnect attempt | socket EOF / connect failure | `Sources/Core/Android/DeviceTracker.swift:79-94` |
| Restart | `adb kill-server`, sleep 500 ms, `adb start-server` | "Restart adb" in panel/header/menu, or the `adbFailed` state | `Sources/Core/Android/ADBClient.swift:82-86`, `Sources/UI/HeaderView.swift:49-50`, `Sources/UI/PanelRootView.swift:50-53`, `Sources/App/OhMyAndroidApp.swift:37` |
| Kill on quit | **never** — deliberate, the server is shared with Studio | — | `Sources/App/AppDelegate.swift:39-42` |
| Host protocol | only `host:track-devices`; **no** `host:version`, `host:devices-l`, `host:tport`, `host-serial:` | — | `Sources/Core/Android/DeviceTracker.swift:109-113` |

`host:track-devices` framing: `String(format: "%04x", service.utf8.count) + service` written to a
`connect()`ed `AF_INET` socket; then the server pushes a device-list block on every change. The app
**ignores the payload** and only uses the read as a "something changed" signal, then re-runs
`adb devices -l` itself (`:58-77`, `:33-43`).

## 3. Invocation table

`one-shot` = process spawn. `argv` = passed as `Process.arguments` (no shell). `shell` = one
`adb shell <string>` handed to the device's `sh`. Frequency abbreviations: L=launch, H=hover/stale
refresh, A=user action, O=once per session.

### 3.1 Transport / device enumeration

| Command / args | Call site | Inputs & quoting | Mode | Parsing | Exit / errors | Timeout | Freq | Failure modes |
|---|---|---|---|---|---|---|---|---|
| `devices -l` | `ADBClient.swift:50` | none | one-shot | line split; skip `List of devices` and `*`; `serial state model:…` (`:98-104`) | non-zero → `ShellError`; empty → "no device" | 60 s | every tracker event, every 4 s while nothing ready | stale `offline` entries; `unauthorized` |
| `-s <serial> shell getprop sys.boot_completed` | `ADBClient.swift:106-114` | serial from `devices -l` | one-shot | trimmed == `"1"` | swallowed (`try?`) | 60 s | per device, per refresh | device drops mid-read → device kept as not-booted |
| `-s <serial> emu avd name` | `ADBClient.swift:116-121` | emulator serial only | one-shot | first line | swallowed; `KO:` check (`:68-72`) | 60 s | per emulator, per refresh | console auth failure → nil AVD name |
| `start-server` | `ADBClient.swift:45-47` | none | detached, no pipes | — | never surfaced | none | L / reconnect | silent; tracker retries |
| `kill-server` | `ADBClient.swift:82-86` | none | one-shot | — | `try?` ignored | 60 s | A (manual) | if the user has devices mid-work they disappear |
| raw `host:track-devices` | `DeviceTracker.swift:96-118` | fixed service string | streaming socket | payload ignored | EOF → backoff + restart | — | O | firewall/AV blocking localhost; port conflict if `ANDROID_ADB_SERVER_PORT` differs from what adb uses |

### 3.2 Shell / exec-out primitives (used by features)

| Command | Call site | Notes |
|---|---|---|
| `-s <serial> shell <cmd>` | `ADBClient.swift:64-66` | returns stdout only; stderr discarded on success |
| `-s <serial> shell "<c1>; echo @@OHMYANDROID@@; <c2>; …"` | `ADBClient.swift:31-36` | batched read; returns nil if section count ≠ command count |
| `-s <serial> exec-out <cmd>` | `ADBClient.swift:74-76` | binary-safe (screenshots, XML, DB bytes) |
| `-s <serial> emu <args…>` | `ADBClient.swift:68-72` | anything containing `KO:` becomes an `AppError` |
| `-s <serial> <args…>` | `ADBClient.swift:78-80` | raw subcommand (used by `install`) |

### 3.3 Device facts

| Command (as one batch) | Call site | Parsing |
|---|---|---|
| `getprop ro.build.version.release` | `DeviceInfo.swift:41-51` | direct |
| `getprop ro.build.version.sdk` | same | direct |
| `pm list packages com.google.android.gms` | same | `contains("com.google.android.gms")` |
| `dumpsys package com.google.android.gms 2>/dev/null \| grep -m1 versionName` | same | regex `versionName=([\d.]+)` |
| `getprop ro.product.cpu.abi` | same | direct |
| `wm size` | same | `Override size: WxH` else `Physical size: WxH` (`Support.swift:29-34`) |
| `wm density` | same | `Override density` else `Physical density` (`:24-27`) |
| `grep -E 'MemTotal\|MemAvailable' /proc/meminfo` | same | regex, formatted in GB |
| `df -k /data \| tail -1` | same | columns 2 & 4 |

### 3.4 Appearance / display

| Feature | Read | Write | Evidence |
|---|---|---|---|
| Dark mode | `cmd uimode night` | `cmd uimode night yes\|no` | `DisplayFeatures.swift:9-13` |
| Font size | `settings get system font_scale` | `settings put system font_scale <0.85…2.00, %.2f>` | `DisplayFeatures.swift:23-27` |
| Display size | `wm density` (ratio vs physical) | `wm density reset` at 1.0, else `wm density <physical×k>` | `DisplayFeatures.swift:40-58` |
| Language/locale | `L=$(cmd locale get-device-locale 2>/dev/null); case "$L" in ''\|*rror*) getprop persist.sys.locale;; *) echo "$L";; esac` | `cmd locale set-device-locale '<bcp47>'` with fallback `am update-config --locale '<bcp47>'` | `DisplayFeatures.swift:85-99` |
| Rotate | — | `adb emu …` (console, via `EmulatorFeatures.rotate`) | `FeatureCatalog.swift:11`, `EmulatorFeatures.swift` (not read in full) |
| Bounds overlay | `getprop debug.layout` | `setprop debug.layout true\|false && service call activity 1599295570 >/dev/null` | `GenericFeatures.swift:22-36`, `Feature.swift:49-53` |
| Overdraw | `getprop debug.hwui.overdraw` | same, value `show` | `DebugFeatures.swift:36` |
| GPU bars | `getprop debug.hwui.profile` | same, value `visual_bars` | `DebugFeatures.swift:37` |
| Show taps | `settings get system show_touches` | `settings put system show_touches 1\|0` | `DebugFeatures.swift:41`, `GenericFeatures.swift:12-18` |
| Pointer location | `settings get system pointer_location` | same pattern | `DebugFeatures.swift:42` |
| Animations off | `settings get global animator_duration_scale` | three `settings put global {window,transition,animator}_*_scale 0\|1` (sequential, 3 process spawns) | `DebugFeatures.swift:11-17` |
| Dev options screen | — | `am start -a android.settings.APPLICATION_DEVELOPMENT_SETTINGS` | `DebugFeatures.swift:53` |

### 3.5 Accessibility

| Feature | Read | Write | Evidence |
|---|---|---|---|
| TalkBack | `settings get secure enabled_accessibility_services` | on: `settings put secure enabled_accessibility_services 'com.google…TalkBackService'` + `settings put secure accessibility_enabled 1`; off: `settings delete secure enabled_accessibility_services` + `accessibility_enabled 0` | `AccessibilityFeatures.swift:12-23` |
| Bold text | `settings get secure font_weight_adjustment` | `settings put secure font_weight_adjustment 300\|0` | `:33-37` |
| Invert colors | `settings get secure accessibility_display_inversion_enabled` | `settings put … 1\|0` | `:46-50` |
| Soft keyboard | `settings get secure show_ime_with_hard_keyboard` | `settings put … 1\|0` | `InputFeatures.swift:45-49` |

### 3.6 Navigation, input, app actions

| Command | Call site | Quoting |
|---|---|---|
| `input keyevent <code>` (3=home, 4=back, 111=escape, 187=recents, 66=enter) | `InputFeatures.swift:12`, `InputTools.swift:74` | none needed (int) |
| `input text '<ascii with %s for spaces>'` | `InputFeatures.swift:33-42` | ASCII-only guard (`:40`), spaces → `%s`, then `shellQuoted` |
| `input tap <x> <y>` / `input swipe x y x y 1000` | `InputTools.swift:24` | ints from dp, validated |
| `input swipe <x1> <y1> <x2> <y2> <ms>` | `InputTools.swift:57` | ints |
| `input keyevent <code>; input keyevent <code>; …` (repeat) | `InputTools.swift:113` | `Array(repeating:).joined("; ")` |
| `am start -W -a android.intent.action.VIEW -d '<url>'` | `AppFeatures.swift:14`, `AppTools.swift:41` | `shellQuoted` |
| `am force-stop '<pkg>'` | `AppFeatures.swift:80`, `AppTools.swift:47` | `shellQuoted` |
| `am force-stop '<pkg>' && monkey -p '<pkg>' -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1` | `AppFeatures.swift:84` | `shellQuoted` |
| `am start -a android.settings.APPLICATION_DETAILS_SETTINGS -d package:'<pkg>'` | `AppFeatures.swift:88` | `shellQuoted` |
| `pm clear '<pkg>'` | `AppFeatures.swift:76` | destructive (`isDestructive`, `:75`) |
| `pm reset-permissions -p '<pkg>'` | `AppFeatures.swift:92` | destructive |
| `pm uninstall '<pkg>'` | `AppFeatures.swift:96` | destructive |
| `cmd package resolve-activity --brief -c android.intent.category.LAUNCHER <pkg> \| tail -n 1` | `AppTools.swift:48` | package validated by regex (`ToolEnvironment.swift:101-108`) |
| `am start -W -n '<activity>'` | `AppTools.swift:50` | `shellQuoted` |
| `dumpsys activity activities \| grep -m1 -E 'Resumed:\|ResumedActivity:'` | `ForegroundAppReader.swift:11` | parse `u\d+ ([\w.]+)/` |
| `cmd connectivity airplane-mode enable\|disable` | `NetworkFeatures.swift:50` | fixed |
| `svc wifi enable\|disable` | `NetworkFeatures.swift:63` | fixed |
| `svc data enable\|disable` | `NetworkFeatures.swift:76` | fixed |
| `pm list packages -3` + `run-as $p true` sweep (one script) | `AppData.swift:37-44` | package names from the device, used unquoted inside a `for` loop |

### 3.7 Install

| Command | Call site | Notes |
|---|---|---|
| `-s <serial> install -r -d -g <absolute local path>` | `AppFeatures.swift:49` | argv, **not** a device shell → spaces are safe; success detected by `output.contains("Success")`, otherwise `AppError(output)` (`:50`) |

No `install-multiple`, no `push`/`pull`/`sync`, no `install-create` streamed session, no progress
reporting. A large APK blocks the feature's busy spinner for the whole transfer (bounded by 60 s).

### 3.8 Screenshots, recording, UI trees, app data

| Command | Mode | Notes |
|---|---|---|
| `exec-out screencap -p` | one-shot, binary | PNG signature scan drops a foldable "multiple displays" warning prefix (`ADBClient.swift:21-27`) |
| `exec-out uiautomator dump /data/local/tmp/ohmyandroid_ui.xml >/dev/null 2>&1 && cat /data/local/tmp/ohmyandroid_ui.xml; rm -f /data/local/tmp/ohmyandroid_ui.xml` | one-shot, binary | retried once at 600 ms if `< 100` bytes (`LayoutSnapshot.swift:98-109`) |
| `shell wm density` (+ `screencap` in parallel) | one-shot ×2 concurrent | `LayoutSnapshot.swift:83-95` |
| `adb emu screenrecord start --fps <n> --bit-rate <n> --time-limit 180 [--size WxH] <temp .webm>` | one-shot console | **console splits args on spaces** — the reason for a temp path and a later move (`CaptureFeatures.swift:52-58`, `:70-86`, `:126-138`) |
| `adb emu screenrecord stop` | one-shot console | file settled by polling size ≤40 × 250 ms (`:88-99`) |
| `shell run-as '<pkg>' ls -1 <dir> 2>/dev/null` | one-shot | `try?` → empty list on denial (`AppData.swift:87-90`) |
| `exec-out run-as '<pkg>' cat shared_prefs/<file>` | one-shot binary | parsed by `PreferencesXMLParser` (`:50-53`, `:94-141`) |
| `exec-out run-as '<pkg>' cat databases/<file>` (+ `-wal`, `-shm`) | 3 × one-shot binary | side files accepted only if WAL magic matches (`:61-85`) |
| `shell run-as '<pkg>' true && echo ok` | one-shot | debuggable probe (`DataTools.swift:106`) |

### 3.9 logcat (MCP only)

```
P=$(pidof -s <pkg>); [ -z "$P" ] && echo '@@NOT_RUNNING' && exit 0; \
logcat -d -v tag [-b crash] [-T "$(( $(date +%s) - <s> )).000"] [--pid=$P] '*:<level>' | grep -iE -- '<pattern>' | tail -n <n>
```
`Sources/MCP/Tools/AppTools.swift:116-151`. One-shot `-d` only; `grep` pattern is `shellQuoted`
(`:141`); lines truncated to 500 chars; empty ↔ "No log lines match". **No streaming, no `-f`, no
pause/clear.**

### 3.10 Emulator console (`adb emu`)

| Domain | Commands | Evidence |
|---|---|---|
| AVD name / snapshots | `emu avd name`; `emu avd snapshot save\|load <name>` | `ADBClient.swift:116-121`, `EmulatorTools.swift:31-32` |
| Recording | `emu screenrecord start…/stop` | `CaptureFeatures.swift:73-76` |
| Network | `emu network status`, `emu network speed <id>`, `emu network delay <n>` | `NetworkFeatures.swift:28-36` |
| Power/battery | `emu power display`, `emu power capacity <n>`, `emu power ac on\|off`, `emu power status charging\|discharging` | `HardwareFeatures.swift:12-34` |
| Telephony | `emu sms send <phone> <text>` | `HardwareFeatures.swift:47` |
| Location | `emu geo fix <lon> <lat>` | `LocationFeatures.swift:32`, `:48` |
| Rotate / fingerprint / call | via `ConsoleActionFeature` (`adb emu <fixed args>`) | `GenericFeatures.swift:40-53`, `FeatureCatalog.swift:11`, `:62-64` |
| Emulator launch (not adb) | `<sdk>/emulator/emulator -list-avds`, `-avd <name>` (detached) | `EmulatorLauncher.swift:8-27` |

Emulator console authentication (`~/.emulator_console_auth_token`) is handled inside `adb` itself —
`UNVERIFIED` (not exercised; no emulator on this machine).

## 4. Injection and quoting hazards (complete list)

| Hazard | Current handling | Risk |
|---|---|---|
| Free text → shell | `String.shellQuoted` single-quote + `'\''` escape (`Support.swift:14`) | correct for POSIX sh; **not** applied everywhere it should be — see below |
| Package name → shell | `shellQuoted` in feature commands; regex-validated in MCP (`ToolEnvironment.swift:101-108`) | low |
| URL → shell | `shellQuoted` (`AppFeatures.swift:14`, `AppTools.swift:41`) | low; but `am start` also interprets `--es` style args inside the URL string |
| logcat `grep` pattern | `shellQuoted` (`AppTools.swift:141`) | low |
| `input text` | ASCII + single-line guard, spaces → `%s`, then quoted (`InputFeatures.swift:39-42`) | **semantic limit**: non-ASCII cannot be typed, documented in UI text |
| `settings put` values | interpolated unquoted (`Feature.swift:46`), but every caller passes a literal or a number | low, fragile |
| `setprop` value | unquoted, literal (`Feature.swift:51`) | low |
| APK path | argv, not shell (`AppFeatures.swift:49`) | safe |
| Device output → next command | only logcat `pidof` and `resolve-activity` → `am start -n` (quoted) | medium: `resolve-activity` output is device-controlled but quoted |
| `run-as` file names | `shellQuoted` (`AppData.swift:51`, `:65`) | low |
| Console arg splitting | emulator console splits on spaces; app avoids user text in console args except SMS body (`HardwareFeatures.swift:47`) | **medium**: an SMS body with spaces will break/extend the console command — `UNVERIFIED` whether the console joins the rest |
| Exit codes | only `0` is success; `ShellError` shows stderr-or-stdout (`ShellRunner.swift:13-22`) | adb returns 0 for many semantic failures → features add their own text checks (`"Success"`, `"Error"`, `"KO:"`) |
| Encoding | stdout decoded as UTF-8 always (`ShellRunner.swift:36-43`); device output is assumed UTF-8 | medium on OEM shells that emit invalid UTF-8 for filenames |

## 5. What is NOT implemented (relevant to the port backlog)

- `push`/`pull`/`sync`, file browser, drag-out of device files.
- `forward`/`reverse`, `tcpip`, `pair`, `connect`, `mdns` (wireless debugging).
- `root`, `uninstall`-system, `remount`, `reboot`/`reboot bootloader`, `wait-for-device`.
- `logcat` streaming/clear/pause.
- `screenrecord` on real devices (only the emulator console path exists).
- `dumpsys` beyond the two greps above; `getprop` read-only.
- `host:version`, `host:devices-l`, `host:tport` — no device-metadata shortcut, hence one `adb` call per
  device for boot state.
- Per-device parallelism limits.
- Any handling of `adb`'s exit code 1 vs 2 distinction, or of `adb` server version mismatch messages.

## Evidence

`rg -n "adb|ADB|5037|emu|shell\(|execOut\(" Sources` plus full reads of: `ADBClient.swift`,
`ShellRunner.swift`, `AndroidSDK.swift`, `DeviceTracker.swift`, `DeviceInfo.swift`, `LayoutSnapshot.swift`,
`AppData.swift`, `EmulatorLauncher.swift`, `ForegroundAppReader.swift`, all `Sources/Features/*.swift`,
`Sources/MCP/Tools/AppTools.swift:110-151`, `Sources/MCP/Tools/{Input,Screen,Data,Emulator}Tools.swift`
(call lines), `Sources/MCP/Tools/ToolEnvironment.swift`.

## Open questions

#4 (full per-tool argument contracts), #3 (platform-tools redistribution).