## Android Panel — Windows installer

A Windows-only desktop panel for `adb`. Lists connected devices and controls the local adb server.

### Install

1. Download `*-setup.exe` below.
2. Verify it against `SHA256SUMS.txt` (optional but recommended):

   ```powershell
   Get-FileHash .\*_x64-setup.exe -Algorithm SHA256
   ```

3. Run it. The installer is **per-user**: no admin prompt, no elevation. It installs to
   `%LOCALAPPDATA%\Android Panel (working title)\`.
4. Silent install for scripts: `& ".\*_x64-setup.exe" /S`

### Before you start

- **Windows 10/11 x64** with the WebView2 runtime (preinstalled on Windows 11; the installer fetches it
  silently on Windows 10, which needs internet **once**).
- **Bring your own Android SDK.** The app ships no `adb`. Install Android Studio + SDK, or SDK
  Platform-Tools alone, then either set `ANDROID_HOME` or point the app at the folder containing
  `platform-tools\` from its setup screen.

### What this build does

- Detects Android Studio, the SDK, `adb`, the emulator and AVDs, and guides you when something is missing.
- Remembers your SDK folder (`%APPDATA%\dev.bramburn.androidpanel\config.json`).
- Lists devices from `adb devices -l` (refresh-only — no hotplug yet) and starts/stops the adb server.
  `Stop` disconnects every adb client on the machine, so it asks for confirmation.

Everything else in the upstream macOS product — appearance, accessibility, layout inspector, app/data/
simulate panels, MCP server — is **not implemented**. See the parity plan in
[`docs/windows-port/04-feature-parity-plan.md`](https://github.com/bramburn/oh-my-android/blob/main/docs/windows-port/04-feature-parity-plan.md).

### Notes

- The installer is **unsigned**: SmartScreen will warn about an unknown publisher. That is expected until
  a code-signing certificate is configured.
- Uninstalling leaves your settings behind. Delete `%APPDATA%\dev.bramburn.androidpanel` for a clean slate.