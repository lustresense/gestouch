# Release gates — Gestouch 0.0.1

Do **not** publish a GitHub Release or claim stability until the owner provides physical Windows build/testing feedback.

- [ ] `powershell -ExecutionPolicy Bypass -File .\development\build.ps1` succeeds on clean Windows 11 x64 with VS Build Tools and Windows SDK.
- [ ] Unit tests pass, `cargo fmt --check`, Cargo dependency audit/allowlisted licenses reviewed.
- [ ] Agent starts once, stays alive when UI closes, tray double click and pause work.
- [ ] `agent-status.txt` increments raw report counter when fingers touch physical PTP.
- [ ] HID logical coordinate range, contact count, tip state and release behavior confirmed for device.
- [ ] Swipe and tap trigger **once**, not duplicated or falsely triggered by one finger.
- [ ] Existing Windows 4-finger controls remain unchanged until deliberately configured; document conflicts.
- [ ] Media actions and keyboard combinations work in normal apps; elevated-app limits documented.
- [ ] Test idle 1h / 8h without stuck capture, resume from S3/Modern Standby, lock/unlock and device reconnect.
- [ ] Verify non-admin startup, optional SCM service start/stop and task re-launch, clean uninstall without orphan tasks.
- [ ] Use `development/scripts/measure.ps1`; report actual idle/active working set/private bytes, CPU and battery impact.
- [ ] Verify `dist` ZIP contains exact three binaries, `scripts`, README, licenses and documentation.
- [ ] Version is exactly `0.0.1`; names `gestouch` (repo) and `Gestouch` (brand). Never `GesTouch`.
- [ ] Review code signing, antivirus behavior, reproducibility and dependencies.
- [ ] After owner validates, decide whether to tag `v0.0.1` and create GitHub Release in a **separate** authorized task.

- [ ] Public-facing `README.md` links resolve after moving developer materials under `development/`.
- [ ] Root license is `GPL-2.0-only`, Cargo metadata agrees, bundled archives include `LICENSE`, and third-party license notices are reviewed.
