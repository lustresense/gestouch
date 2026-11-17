# Gestouch

**Native Windows Precision Touchpad gesture experiments — Rust · v0.0.1**

> **Developer preview, not a verified production release.** This is an independent Rust implementation, not a translation/fork of GestureSign V2. Real Precision Touchpad compatibility, service recovery, memory usage, and gesture conflicts have **not yet been verified on physical Windows hardware**. See [Known limitations](docs/KNOWN_LIMITATIONS.md).

Gestouch is built around a small, event-driven **per-user input agent**, a separate **native Win32 Settings app**, and an **optional true Windows Service** acting only as a supervisor. The Windows service never reads touchpad input, injects keystrokes, or runs scripts in Session 0.

## Get started (Windows 11 x64)

**Prerequisites:** Windows 11 x64, Windows Precision Touchpad, PowerShell 5.1+, Visual Studio 2022 **Build Tools + MSVC C++ toolchain + Windows SDK**, and internet access on first Cargo build. The single build script can install Rustup through `winget` if Rust is missing. It cannot silently substitute for the MSVC/Windows SDK requirement.

From an extracted source folder in PowerShell:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\build.ps1
```

That one command formats source, runs unit tests, builds all three Windows binaries in release mode, and packages `dist/Gestouch-0.0.1-win-x64-portable.zip`. **Nothing is auto-installed or elevated by the build alone.**

To build **and install** for the signed-in user:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\build.ps1 -Install
```

For **optional experimental real Windows Service Mode**, open elevated PowerShell (Run as Administrator):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\build.ps1 -Install -Service
```

An independent install of already-built binaries is possible using `scripts/install.ps1` after building, and `scripts/uninstall.ps1` removes the scheduled task and service. User configurations are preserved unless `-RemoveConfig` is passed to uninstall.

### Running without installation

After building, launch `dist/Gestouch-0.0.1-win-x64/gestouch-agent.exe`, then `gestouch-settings.exe`. The agent stays alive if Settings closes. Double-click the agent system tray icon to reopen Settings; **right-click toggles pause/resume** (alpha UI; richer context menu is planned).

### First gesture

1. Launch Settings → **Gestures**.
2. Select finger count, direction and a media/volume/keyboard action.
3. Click **Add rule**. The agent rereads `config.toml` within approximately five seconds.
4. Test on a physical Precision Touchpad; check **Diagnostics** for raw report and recognized-gesture counts.

**No custom gestures are enabled by default.** Gestouch will not overwrite your existing 4-finger Windows volume/music configuration. However, creating a gesture whose trigger Windows already owns may still cause double actions; the alpha does **not** suppress Windows-native gestures or automatically resolve their conflicts.

## v0.0.1 feature scope

| Component | In 0.0.1 | Verification |
|---|---|---|
| Rust gesture recognizer | Swipe/tap state machine; multi-finger coherence; debounce; tests | Source-written, **not executed in this environment** |
| PTP input | Message-only Win32 receiver; Raw Input registration; HID contact ID / X / Y; coordinate normalization; basic hybrid-report assembly | **Requires real Windows build + device** |
| Actions | Volume/mute, media next/previous/play/pause, allowlisted keyboard shortcuts via `SendInput` | Windows hardware testing pending |
| Native UI | Dashboard, gesture rules, diagnostics, startup, about | Windows UI testing pending |
| Recovery | Resume, device changes, registration mismatch; no reset solely from silence | Windows sleep/resume testing pending |
| Startup | Current-user scheduled task | Installation test pending |
| True SCM service | Optional supervisor runs the interactive user's registered scheduled task | Experimental; security/Session 0 tests pending |
| UI design reference | 17-screen HTML wireframe + full PRD | Design reference, **not production UI** |
| Installer | PowerShell install/uninstall; portable ZIP | Windows testing pending |

**Not implemented in 0.0.1:** full PRD's 17 native production screens, per-app gesture profiles, arbitrary gesture drawing, dynamic threshold tuning UI, gesture conflict suppression, TipTap, edge gestures, custom plugin modules, brightness, screen-recorder, multi-user service supervision, Microsoft Store packaging, ARM64 binaries and automatic updater. These are tracked as later milestones; see the [implementation matrix](docs/IMPLEMENTATION_MATRIX.md). Never advertise an unimplemented feature as shipping.

## Repository structure

```text
.
├── build.ps1                       # One-command Windows entry point
├── Cargo.toml                      # Rust workspace 0.0.1
├── crates/
│   ├── gestouch-core/             # Pure Rust config + recognizer + unit tests
│   ├── gestouch-win/              # Minimal native Win32 API declarations
│   ├── gestouch-agent/            # HID capture, dispatch, tray, recovery
│   ├── gestouch-settings/         # Native Win32 Settings, on demand
│   └── gestouch-service/          # Optional SCM-only supervisor
├── scripts/
│   ├── build.ps1                  # ONE entry point for build + ZIP
│   ├── install.ps1                # User / optional elevated Service Mode
│   ├── uninstall.ps1
│   ├── publish.ps1                # Optional GitHub CLI create/push (never release)
│   └── measure.ps1                # Runtime memory/CPU recording
├── docs/
│   ├── PRD_Gestouch_v1.0.md       # Original complete specification
│   ├── WIREFRAMES.html            # Interactive design, 17 screens
│   ├── UPSTREAM_AUDIT.md
│   ├── ARCHITECTURE.md
│   ├── IMPLEMENTATION_MATRIX.md
│   ├── KNOWN_LIMITATIONS.md
│   └── RELEASE_CHECKLIST.md
└── .github/workflows/windows.yml # Test + build + artifact; NO auto-release
```

## Build & runtime details

- **Single agent process** with Win32 message loop; no polling of finger position while idle. A periodic 30-second health check verifies raw input registration, while config is reread every 5 seconds. Neither requires preventing Windows sleep.
- **UI is not required for runtime.** Settings is a separate native Win32 binary, no WebView/Electron/.NET runtime.
- **Config:** `%LOCALAPPDATA%\Gestouch\config.toml` (schema version 1).
- **Health:** `%LOCALAPPDATA%\Gestouch\agent-status.txt`; informational only, not a reliable authenticated IPC channel.
- **Autostart:** Windows Task Scheduler (`GestouchAgent`), *not* a per-user privileged service.
- **Optional service:** `GestouchSupervisor`, true SCM process in Session 0. It asks Task Scheduler to run the existing interactive task, rather than launching as SYSTEM on the desktop. It does **not** preserve UI state across user-session changes.
- **No telemetry and no cloud service.**

## Tests and hardware evidence

Run `cargo test --workspace --target x86_64-pc-windows-msvc`. Hardware qualification requires dedicated testing of distinct laptop touchpads, Windows gestures, lock/unlock, Modern Standby, multi-report HID, user elevation, sleep/resume, service lifecycle and memory. Use `scripts/measure.ps1` to capture runtime sample CSV. **RAM <20 MiB is a goal, not an observed result.**

For issues, attach Windows build version, device name, Gestouch 0.0.1 binary hash, agent status, reproduction steps, and whether Windows system gestures are enabled. Do not post raw personal HID/keyboard traces.

## Source and licensing

Gestouch is independently authored under **MIT OR Apache-2.0**. The repository includes no copied GestureSign V2 source (which uses GPL-2.0); the upstream repo and Microsoft HID documentation were studied as behavioral references and are cited in `docs/UPSTREAM_AUDIT.md`. Redistribution of external libraries and any added assets requires separate review. Copyright is owned by Gestouch contributors.

See [Contributing](CONTRIBUTING.md), [Security](SECURITY.md) and [Release checklist](docs/RELEASE_CHECKLIST.md).
