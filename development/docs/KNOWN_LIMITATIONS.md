# Important 0.0.1 limitations / known risks

- **No verified Windows build:** The authoring environment did not have a Rust compiler, Windows SDK, Windows machine, or physical Precision Touchpad; all binaries require your Windows build.
- **No universal OS gesture override:** Windows 3/4-finger behavior may execute in parallel. Gestouch does not suppress existing system mappings.
- **PTP HID quirks:** Contact-count timing, tip flags, hybrid multi-report frames, contact release, logical coordinate caps, touchpad firmware, Modern Standby, Bluetooth and USB PTP variations need empirical tests.
- **Settings alpha UI:** five native pages implemented; full design spec is a **separate offline HTML wireframe** (17 screens) and not a shipped runtime GUI feature.
- **Service is experimental:** Running a true Windows SCM service may not restart an interactive scheduled task on every Windows policy configuration. The service has no HID access, runs at 60-second cadence and is optional. It assumes one consenting interactive user/task and does not securely supervise multiple user sessions.
- **No keyboard override of protected apps:** UIPI limits Win32 SendInput; no privilege escalation workaround.
- **No promise of 15/20 MiB RAM:** Measure private bytes and working set in a clean Windows environment; source size and Rust alone do not prove performance.
- **Config reload latency:** up to ~5 seconds; no reliable notification IPC.
- **No UI localization, per-app profiles, gesture editing/undo, brightness, TipTap, screen-edge, smart gestures or custom scripts yet.**
- **No code signing or anti-virus compatibility verification yet.** Defender false positives and code signing must be assessed before stable release.

Report bugs with device model, Windows build, physical touchpad capability info, steps, status snapshot and build output; do not include passwords or full raw input traces.
