# GestureSign V2 — Static Source Architecture & Risk Audit

**Audit date:** 2026-11-16 (Asia/Jakarta)  
**Upstream:** https://github.com/Tomclanc/GestureSignv2  
**Reviewed branch:** `main` at commit [`8e331791ca7c203a15f707871b803c2e94c63749`](https://github.com/Tomclanc/GestureSignv2/commit/8e331791ca7c203a15f707871b803c2e94c63749)  
**Current release verified via GitHub API:** `v18.4.6`, published 2026-11-16 06:13:23 UTC  
**Upstream license:** GNU GPL version 2 (`LICENSE`; verify file-by-file headers before reusing code)  
**Purpose:** design evidence for an *independently implemented* native Rust gesture utility, not an audit report claiming a certified security review.

## 0. Scope and confidence

The complete Git tree was enumerated through GitHub's recursive tree endpoint: **990 tree entries**, `truncated=false`. Inventoried major directories, project files, installer, tests, third-party/source sections, and inspected source code of core input, lifecycle, recovery, configuration, plugin, IPC, and action subsystems. The tree contains **351 `.cs` files, 50 `.csproj`, 187 `.json`, 93 `.xml`**, alongside other assets. Read and reviewed representative critical files within those modules. This is a **whole-tree inventory plus targeted static audit of critical paths**, **not** a line-by-line review of all 351 C# files, dependency vulnerability scan, compilation, fuzzing, live benchmarking, or Windows device testing. No security exploit is asserted without reproduction.

**Evidence vocabulary:** `Observed` = verified from a specific upstream source; `Risk` = a design/security/performance concern inferred from the code, not measured or proven exploitable; `Unknown` = needs testing or full review.

## 1. Inventory by subsystem and disposition

| Upstream part | Observed responsibility | Proposed native Rust disposition | Priority |
|---|---|---|---|
| `GestureSign.Daemon/` | WinForms-hosted per-user tray process, input window, hooks, gesture translation, triggers, overlays, Kando bridging | **Reimplement**, split `engine` / `input-win32` / `recovery` / `tray` | P0 |
| `GestureSign.Common/` | Application/action models, matching, config, gestures, IPC, plugin loader, logging | **Reimplement**, strongly typed data and versioned IPC | P0 |
| `GestureSign.PointPatterns/` | Pattern analysis/recognition math | **Independently implement** deterministic recognition and geometric normalization | P0 |
| `GestureSign.Foundation/` | Cross-cutting data models, tip-tap recognizer, capture lifecycle, intent models, localization | **Reimplement core only**, defer intent/UI localization | P0–P2 |
| `GestureSign.CorePlugins/` | Media, volume, brightness, hotkeys, script, window and cursor commands | **Native typed actions**, no DLL plugin loader in hot path | P0–P1 |
| `GestureSign.ExtraPlugins/` | Clipboard/text copy extras | Optional future adapters; don't port in MVP | P3 |
| `GestureSign.WinUI/` | WinUI 3 settings, action editors, Kando, localization, updates, device options | **Rust-native on-demand settings app**, no UI loaded in background process | P1 |
| `GestureSign.IntentLearning/` | Local intent model | Defer experimental ML; deterministic baseline first | P3 |
| `GestureSign.IntentDlc/` | Optional CPU/hardware inference (ONNX/Windows AI) | Optional future *separate process*, no default install | P3 |
| `ManagedWinapi/`, `WindowsInput/` | Managed Win32/hooks/input helpers | Replace with direct `windows-sys`/Win32 bindings | P0 |
| `Shared/`, `Defaults/` | UI helpers/default gesture data | Independently authored defaults; migration importer as later feature | P1–P2 |
| `installer/` | Bootstrapper, updater, uninstaller, packaging scripts | Rust-native installer/uninstaller or conventional MSI packaging; no runtime .NET | P1 |
| `.github/workflows/`, `tests/`, `tools/` | CI, regression and support diagnostics | Keep the *testing intent*; build new Rust tests and Windows runner coverage | P0 |
| `docs/` | Help, support, engineering docs | Author own docs and attribution, no GPL text/code copy | P1 |

### Upstream technology snapshot

- `global.json` targets .NET SDK `10.0.100` with feature roll-forward.
- `GestureSign.Daemon.csproj` targets `net10.0-windows10.0.19041.0`, `UseWindowsForms=true`, `UseSystemDrawing=true`, and x64/ARM64 configuration.
- `GestureSign.Common.csproj` uses WinForms **and WPF**, and Newtonsoft.Json 13.0.3.
- `GestureSign.WinUI.csproj` uses WinUI 3 and Windows App SDK packages, shared .NET/Windows App SDK runtime dependencies for portable/MSI distributions.
- `GestureSign.IntentDlc.csproj` uses conditional hardware libraries (`Microsoft.Windows.AI.MachineLearning`, ONNX Runtime Managed) only for its hardware backend.
- Upstream README describes Kando as optional and separately downloaded, not core engine.

## 2. Static technical findings

| ID | Finding and evidence | Severity to new design | Recommended requirement |
|---|---|---|---|
| A-01 | **Observed:** `Input/MessageWindow.cs` registers Digitizer usages with `RIDEV_INPUTSINK | RIDEV_DEVNOTIFY` on hidden native window; `WM_INPUT` drives capture. | P0 feasibility | Prove delivery in user background and compatibility with OEM PTP before UI work. |
| A-02 | **Observed:** `ProcessInputCommand` uses `Marshal.AllocHGlobal` and `FreeHGlobal` around *each* raw message; instantiates HID device parser and collections. **Risk:** allocation churn / CPU pressure in high-frequency input. | P1 performance | Reusable bounded buffers/arena, capped report sizes, zero-allocation hot-path target where feasible; profile first. |
| A-03 | **Observed:** `_sourceDevice`, `_requiringContactCount`, and `_outputTouchs` track input source and frames; zero-contact path may return without delivering frame when no outstanding contacts. **Risk:** malformed/orphan input can wedge ownership. Separate community test build explicitly addresses orphan zero-contact reports; do not assume release fixes every device. | P0 reliability | Explicit finite frame deadlines, report provenance, per-device state, trace-based regression fixtures, reset after verified stale state. |
| A-04 | **Observed:** `InputProvider.cs` subscribes to Windows `PowerModeChanged` and `SessionSwitch`; on resume/unlock reconnects after a 600 ms deferred request. | P0 reliability | Event-driven device re-enumeration, cancel in-progress gestures safely, idempotent recovery, bounded backoff. |
| A-05 | **Observed:** 30-second WinForms health timer logs snapshot, writes `Input-health.json` and repairs *confirmed registration mismatch*. `RawInputHealthPolicy` states idle silence is **not** repair evidence. **Risk:** periodic disk writes and checks could be overkill for an ultra-light agent. | P1 resource | In-memory counters; low-frequency checks; write diagnostics on transitions, errors or opt-in rather than every heartbeat. |
| A-06 | **Observed:** `PointCapture.RecoverInput` cancels current capture and calls re-registration/hook rebuild. | P0 reliability | Transactional recover; preserve config and enabled/paused state; prevent duplicate hooks or ghost presses. |
| A-07 | **Observed:** `Program.cs` calls `SetProcessInformation(ProcessPowerThrottling)` to opt the daemon out of execution-speed power throttling. **Risk:** possible battery impact; no benchmark attributed. | P1 power | Avoid blanket power-throttling opt-out; measure input latency vs battery/energy. |
| A-08 | **Observed:** `MainWindow.xaml.cs` registers ordinary startup with a `.lnk` in user's Startup folder, elevated startup with `schtasks.exe`/scheduled task `StartGestureSign`. Not a Windows SCM service. | P0 lifecycle | Explicit Light Mode auto-logon; optional true SCM supervisor + per-user interactive agent. |
| A-09 | **Observed:** `PluginManager.cs` finds extra `.dll` files and uses assembly loading/reflection. **Risk:** expanding trusted-code surface, with script/plugin action execution. | P0 security | No dynamic native DLL loading in MVP; typed built-in actions; explicit trust gate for scripts/extensions. |
| A-10 | **Observed:** `CustomNamedPipeServer.cs` creates a pipe ACL allowing Builtin Users read/write in a relevant path. **Risk:** command-bearing local IPC needs authorization review; **not** proof of an exploit. | P0 security | Per-user/session SID scoped pipe ACL, request validation, no SYSTEM command execution; adversarial IPC tests. |
| A-11 | **Observed:** `RunCommandPlugin.cs` can run PowerShell or CMD, including administrator mode on request. | P0 security | Scripts off by default; display exact commands; explicit user opt-in; never run commands through service privilege. |
| A-12 | **Observed:** local per-app matching, gesture exclusions, and Windows-preference controls exist in config/UI. | P0 compatibility | First-class collision policy: report conflict, let Windows win by default, never promise suppression unsupported by API. |
| A-13 | **Observed:** `GestureSign.WinUI` large code + 90 locale catalogs and Kando/AI management. | P2 footprint | Keep all settings/UI/resources out of agent process; lazy optional packages. |
| A-14 | **Observed:** `LegacyDataStore` reads/writes legacy JSON-based action and gesture data; OneDrive sync optional. | P2 migration | Versioned TOML/JSON schema, atomic save; import legacy data **without copying GPL code** if format is suitable; manual export-first fallback. |
| A-15 | **Observed:** core plugins include brightness and volume. Hardware capability and target display behavior vary. | P1 portability | Native feature probes; brightness fallback modes (internal WMI vs external DDC/CI, guarded optional), informative errors. |
| A-16 | **Observed:** source includes issue-specific Windows architecture and lifecycle tests. | P0 testing | Preserve these *test categories*, generate independent fixtures, require physical x64 and ARM64 gate for advertised support. |

### Input pipeline: key source paths inspected

1. [`GestureSign.Daemon/Input/MessageWindow.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/MessageWindow.cs): `WM_INPUT`, raw HID, capture source, packet assembly, allocations.
2. [`GestureSign.Daemon/Input/HidDevice.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/HidDevice.cs): HID contact parser.
3. [`GestureSign.Daemon/Input/TouchPadDevice.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/TouchPadDevice.cs): coordinate and contact handling.
4. [`GestureSign.Daemon/Input/InputProvider.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/InputProvider.cs): recovery on resume/unlock, heartbeat.
5. [`GestureSign.Daemon/Input/MessageWindow.Health.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/MessageWindow.Health.cs): raw input health snapshot.
6. [`GestureSign.Common/Input/RawInputHealthPolicy.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Common/Input/RawInputHealthPolicy.cs): do not recover on idle silence.
7. [`GestureSign.Daemon/Input/PointCapture.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/PointCapture.cs): mode and recovery.
8. [`GestureSign.Daemon/Input/PointEventTranslator.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Input/PointEventTranslator.cs): gesture source state and release handling.
9. [`GestureSign.Daemon/Program.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.Daemon/Program.cs): process lifecycle and power-throttling settings.
10. [`GestureSign.WinUI/MainWindow.xaml.cs`](https://github.com/Tomclanc/GestureSignv2/blob/8e331791ca7c203a15f707871b803c2e94c63749/GestureSign.WinUI/MainWindow.xaml.cs): startup implementation approximately lines 7338–7584.

### Other architectural paths inspected

- `GestureSign.Common/Applications/ApplicationManager.cs` — app matching, actions, config save debounce.
- `GestureSign.Common/Gestures/GestureManager.cs` — template-based recognition and saved gesture data.
- `GestureSign.PointPatterns/PointPatternAnalyzer.cs` — pattern analysis.
- `GestureSign.Foundation/TouchPadTipTapRecognizer.cs` — multitouch tap chord logic.
- `GestureSign.Daemon/Triggers/TouchPadEdgeTrigger.cs` — edge action resolution.
- `GestureSign.Common/Plugins/PluginManager.cs` — core and dynamic plugin loading.
- `GestureSign.Common/InterProcessCommunication/NamedPipe.cs` and `CustomNamedPipeServer.cs` — UI/daemon IPC.
- `GestureSign.WinUI/Services/DaemonBridgeService.cs` — UI to background bridge.
- `GestureSign.CorePlugins/RunCommand/RunCommandPlugin.cs` — script execution.
- `GestureSign.CorePlugins/ScreenBrightness/ScreenBrightnessPlugin.cs` — brightness actions.
- `GestureSign.CorePlugins/Volume/VolumePlugin.cs` and `MediaControl/MediaPlugins.cs` — volume/media.
- `GestureSign.Daemon/Input/TouchPadIntentBridge.cs` and `GestureSign.IntentDlc/GestureSign.IntentDlc.csproj` — optional inference integration.
- `GestureSign.WinUI/LegacyDataStore.cs` — configuration persistence and legacy actions.
- `GestureSign.Daemon/GestureSign.Daemon.csproj`, `GestureSign.WinUI/GestureSign.WinUI.csproj`, and `GestureSign.Common/GestureSign.Common.csproj` — runtime dependency graph.

## 3. Confirmed public upstream bug history

[Issue #6](https://github.com/Tomclanc/GestureSignv2/issues/6) is **titled "Feature Request: Legacy GestureSign .ges importer"**. Its *comment thread* also discusses intermittent input freeze, so do not mislabel the entire issue as only a freeze bug. [Community ARM64 test build](https://github.com/Tomclanc/GestureSignv2/releases/tag/v18.4.4-community-test1) explicitly mentions a fix attempt for orphan zero-contact HID source ownership and states physical Surface testing remained pending at that test release. The latest stable v18.4.6 addresses other gesture/volume issues; its release notes alone do not prove all intermittent input freezes fixed.

## 4. License assessment

**Observed:** upstream repo root `LICENSE` is GNU GPL version 2. The GPL's copyleft obligations must be evaluated if any original source is copied or a distributed derivative work is made. Merely implementing similar *functions, requirements, or ideas* independently does not automatically impose GPL on an independently written implementation.

**Recommendation:** independently implement the Rust code with original types, state machines, documentation, and tests; use public Microsoft API specifications and license-compatible examples. Choose `MIT OR Apache-2.0` if all future third-party code and contributions permit this. Include an acknowledgements page that credits upstream inspiration but does not copy implementation. Do **not** mechanically transpile GPL C# code and then relicense it permissively. If actual upstream source is reused or adapted, review specific licenses/rights and choose GPL-compatible distribution with proper notices and corresponding source.

**Needs further review before shipping:** each bundled asset, copied/default gesture dataset, external sample code, libraries/crates, icons, translation catalogs, installer helper, optional models and their individual licenses. An SPDX/NOTICE/SBOM check is a release gate.

## 5. Additional security audit work (not yet performed)

- Full call-graph and trust-boundary review for all named-pipe commands and message processors.
- Reproduce pipe access control against multiple local accounts and elevation levels.
- Confirm all plugin loading paths and updater verification / signature handling.
- Fuzz HID report descriptor and malformed report decoding; test overlarge lengths, zero-contact, out-of-order frames.
- Build and run x64/ARM64 binaries, unit tests, Windows hardware tests, static analysis (`cargo clippy`, `cargo audit`, MSVC checks where relevant), and threat modelling.
- Determine actual energy use, RAM private bytes/working set, CPU, and input event-to-action latency under physical devices.

## 6. Source and platform references

- Upstream: https://github.com/Tomclanc/GestureSignv2
- License: https://github.com/Tomclanc/GestureSignv2/blob/main/LICENSE
- Microsoft — Raw Input registration: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerrawinputdevices
- Microsoft — HID architecture and exclusive PTP usage: https://learn.microsoft.com/en-us/windows-hardware/drivers/hid/hid-architecture
- Microsoft — Session 0 isolation: https://learn.microsoft.com/en-us/windows/win32/services/service-changes-for-windows-vista
- Microsoft — Logon triggers: https://learn.microsoft.com/en-us/windows/win32/taskschd/logontrigger
- Microsoft — Service user accounts: https://learn.microsoft.com/en-us/windows/win32/services/service-user-accounts

**Final audit classification:** good reference for behavior and edge cases; **not** a low-risk base for an MIT-licensed mechanical Rust port. Independent, feature-gated Rust implementation recommended, with Phase 0 hardware feasibility as a mandatory stop/go milestone.
