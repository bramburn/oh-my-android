# Legal

No legal advice. Facts and obligations only, each with a source. **Nothing here relicense anything.**

## 1. The repo licence

| Fact | Value | Evidence |
|---|---|---|
| Licence | MIT | `LICENSE:1` |
| Copyright line | `Copyright (c) 2026 Royan AB` | `LICENSE:3` |
| Obligations | include the copyright notice **and** the permission notice in all copies or substantial portions of the Software | `LICENSE:4-20` |
| Warranty | none; "AS IS" | `LICENSE:15-19` |
| Copyleft | none | — |
| File-level notices | none (no per-file headers) | `rg -n "SPDX|Copyright" Sources` → 0 hits |

### What this fork must therefore do

1. Keep `LICENSE` **byte-for-byte identical** to upstream. Done; verified by `git diff upstream-baseline-9f37391 -- LICENSE` → empty.
2. State that it is a modified fork, with the upstream URL and fork commit. Done in `NOTICE.md`.
3. Not imply endorsement by the upstream author. The fork README names the upstream project and links it.

### What this fork may do

- Re-implement the behaviour in another language and ship it under MIT (with the upstream notice kept).
- Rewrite or drop any code. MIT grants "use, copy, modify, merge, publish, distribute, sublicense, and/or
  sell copies" (`LICENSE:4-9`). **A clean-room rewrite is not legally required.**

### What this fork must not do without the author's permission

- Reuse the names/marks that identify the upstream product to end users:
  "Oh My Android", the app icon (`Sources/App/Assets.xcassets/AppIcon.appiconset/*`),
  the bundle id `se.royan.ohmyandroid` (`project.yml:74`),
  the Homebrew cask/tap identity (`Scripts/release.sh:88-111`),
  the GitHub release feed `ateymoori/oh-my-android` and the pinned Sparkle EdDSA key
  (`project.yml:57-58`).
  Reason: a fork publishing under these would send users updates signed by the upstream key — or
  silently break updates — and would misattribute the work. Rename and re-key before any public release.
- State or imply the upstream author endorses this fork.

## 2. Bundled / distributed binaries

| Binary | Bundled today? | Rights / obligations | Evidence |
|---|---|---|---|
| Sparkle 2.10.0 (`Sparkle.framework`) | **Yes**, inside the app | MIT. Its own licence and copyright must travel with the framework (it does, as an embedded resource of the framework). | `project.yml:26-31` |
| `adb`, `emulator` (Android SDK platform-tools / emulator) | **No.** The app locates the user's SDK and refuses to run without it. Upstream comment: "Nothing is bundled; Google's license forbids redistribution." | Google Android SDK terms govern `platform-tools`. Redistribution by a third party is **not** allowed under the SDK terms without a separate agreement. | `Sources/Core/Android/AndroidSDK.swift:3-6`, `Sources/Core/Android/EmulatorLauncher.swift:3-6` |
| `scrcpy` | Not used, not bundled | n/a (Apache-2.0 if ever bundled) | `docs/architecture/07-mirroring-and-control.md` |
| `ffmpeg` | Not used, not bundled | n/a | `rg ffmpeg` → 0 hits |
| Device-side binaries (`uiautomator`, `settings`, `cmd`, `pm`, `am`, `input`, `run-as`, `logcat`) | Not distributed by us; they live on the device image | AOSP, Apache-2.0 | invoked via `adb shell` (`docs/architecture/06-adb-integration.md`) |
| Apple frameworks | Linked, not redistributed | Apple SDK terms | `project.yml` |

**Consequence for a Windows port:** shipping `platform-tools` inside a Windows installer is a *licensing
decision*, not an engineering one. Three lawful options: (a) never bundle — reuse upstream's "bring your
own SDK" model; (b) download platform-tools from Google on first run, under Google's terms, with the user
accepting them; (c) obtain permission. Option (a) is the only one this fork can take without external
input. `UNVERIFIED`: the current wording of Google's terms — resolve via `platform-tools/NOTICE.txt`
from an SDK install (`docs/open-questions.md` #3).

## 3. What a Windows re-implementation must attribute

| Item | Obligation |
|---|---|
| Upstream MIT notice | reproduce in the Windows app's About/licences screen and in the repository |
| Sparkle | only if Sparkle-equivalent code is used — it will not be (Windows needs a different updater) |
| Upstream prompts / tool descriptions copied verbatim | they are MIT-covered text; attribution in the About screen is sufficient and recommended |
| Android Open Source Project strings/commands | no attribution needed (command names, not code) |

## 4. Privacy / data protection

No telemetry, no PII collection, no server. The only network call is the update-check GET
(`SECURITY.md:9-16`, `docs/architecture/09-cross-cutting.md`). A Windows port that adds crash reporting
would change the privacy posture stated in `SECURITY.md` and `README.md` ("no telemetry"), so it must be
opt-in and documented — otherwise the fork is not a faithful port.

## 5. Evidence

`LICENSE`, `NOTICE.md`, `project.yml:26-31`, `:57-58`, `:69-74`, `Sources/Core/Android/AndroidSDK.swift:3-6`,
`Scripts/release.sh:88-111`, `SECURITY.md`, `docs/upstream/README.md`.