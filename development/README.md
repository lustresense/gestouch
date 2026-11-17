# Gestouch Development Hub

Everything a maintainer needs to **build, test, install, measure, debug, and release** Gestouch lives here. Public-facing product information is in the root [README](../README.md).

> v0.0.1 is experimental. No real Windows binary or touchpad hardware has been verified by the original authoring environment. CI is a first build gate, not proof of device compatibility.

## One command: build from the repository root

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\development\build.ps1
```

Requirements: Windows 11 x64, PowerShell 5.1+, Visual Studio Build Tools (MSVC C++ workload) + Windows SDK, internet for the initial Rust Cargo dependency download. If cargo is not installed, the script attempts `winget` Rustup installation. Open a fresh terminal if PATH changes aren't available.

This script formats the workspace, executes Rust tests, compiles the release targets and packages a ZIP at `dist/Gestouch-0.0.1-win-x64-portable.zip`. It does **not** produce or publish a GitHub Release.

**Install** at current-user startup:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\development\build.ps1 -Install
```

**Experimental SCM supervisor** (run elevated PowerShell; additional overhead):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\development\build.ps1 -Install -Service
```

If an existing build is already packaged, use `development/scripts/install.ps1 -SourceDirectory <absolute-built-folder>` and `development/scripts/uninstall.ps1`. Uninstall preserves config unless `-RemoveConfig` is specified.

## Repository map

| Location | Purpose |
| --- | --- |
| `../crates/` | Rust engine, Windows FFI, agent, settings, optional service |
| `build.ps1` | **Single build entrypoint**; performs tests/build/package and optional installation |
| `scripts/install.ps1` | Install interactive agent, optionally experimental service |
| `scripts/uninstall.ps1` | Remove installed agent/task/service |
| `scripts/measure.ps1` | Measure Windows RAM/CPU usage (not a precomputed result) |
| `scripts/publish.ps1` | Maintainer-only git/gh push helper; **does not publish a release** |
| `docs/ARCHITECTURE.md` | Process and HID pipeline design |
| `docs/IMPLEMENTATION_MATRIX.md` | What v0.0.1 actually implements |
| `docs/KNOWN_LIMITATIONS.md` | Device and feature risks |
| `docs/PRD_Gestouch_v1.0.md` | Target product specification (future scope included) |
| `docs/UPSTREAM_AUDIT.md`, `docs/PROVENANCE.md` | Upstream references and source lineage |
| `docs/RELEASE_CHECKLIST.md` | Required release gates |
| `design/` | HTML wireframes and visual concept previews; not shipping UI |
| `CONTRIBUTING.md` | Contribution and license policy |

## Maintainer verification order

1. Run a fresh Windows build. Capture compiler/test exit codes and `dist/` artifact path.
2. On a physical Precision Touchpad, verify Raw Input contact identification, multi-contact frames, and gesture recognition with Windows 3/4-finger shortcuts both enabled and adjusted.
3. Confirm pause/resume, lock/unlock, sleep/wake, user logoff/login, and task startup on AC/battery.
4. Install and uninstall Light Mode; then independently test experimental Service Mode with admin rights.
5. Measure CPU, working set/private bytes, and battery impact with `scripts/measure.ps1`; compare with actual evidence, not target numbers.
6. Review security, third-party licenses, PRD implementation matrix, then follow `docs/RELEASE_CHECKLIST.md` before publishing any tagged release.

## License

Gestouch itself uses [GPL-2.0-only](../LICENSE). Dependency license information and source provenance are tracked separately; see `docs/THIRD_PARTY_NOTICES.md` and `docs/PROVENANCE.md`. It is an independent Rust implementation inspired by GestureSign, not a repository derived by copying the C# source.
