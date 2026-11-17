# PRD-to-v0.0.1 implementation matrix

| PRD capability | v0.0.1 status | Ownership / next requirement |
|---|---|---|
| Precision Touchpad Raw Input | Implemented source; hardware unverified | `gestouch-agent/input.rs`; G0 test required |
| 2–5 finger swipe/tap | Pure Rust recognizer; synthetic tests source-written | `gestouch-core/gesture.rs`; calibration needed |
| Multi-report contact frame | Basic buffering; not device-qualified | `gestouch-agent/input.rs`; add scan-time/contact-drop tests |
| Hotkey and media actions | Native `SendInput`; not hardware-qualified | `gestouch-agent/actions.rs` |
| Gesture CRUD and undo | Add/delete only; no undo/edit/clone | Native Settings + config |
| Custom gesture recorder | Reference design only | PRD UI-05 |
| Conflicting Windows gestures | Warning only; no automatic detection or suppression | PRD section 12 |
| Per-app profiles | Not implemented | PRD UI-08 |
| Edge/TipTap/hold | Not implemented | Later milestone |
| Brightness | Not implemented | API backend per-device and capability detection |
| 17-screen UI | 5 basic native pages + 17-screen HTML reference | UI screens UI-00..UI-16 |
| Windows dark theme / DPI | OS-default colors; no custom theme | UI design tokens / tests |
| Login autostart | Task Scheduler code/scripts; unverified | `scripts/install.ps1` / Settings |
| Real Windows Service | SCM process plus scheduled-task trigger; experimental | Multi-session/security hardening pending |
| Resume recovery | Re-registration trigger; unverified | agent main |
| Health | Per-user snapshot file; no authenticated IPC | alpha only |
| Desktop diagnostics | Status counts, no raw monitor/recorder | Future support bundle |
| Plugin architecture | Not implemented, intentionally no untrusted DLL loading | Future security review |
| Config import/export | TOML schema 1, backup on save; no UI import | Future restore flow |
| Release pipeline | Windows CI artifact only | GitHub Release deferred until owner confirms build |
| Performance | Goals only; `measure.ps1` can collect evidence | 30-minute idle and active capture needed |

**Scope principle:** v0.0.1 is an experimental implementation milestone, not the 1.0 full PRD realization. Source compilation has not yet been verified in this environment, which has no Rust/MSVC toolchain.
