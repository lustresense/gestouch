# Contributing to Gestouch

Thanks for helping build a better open-source Windows touchpad experience. **Gestouch v0.0.1 is an experimental Rust implementation**, so hardware testing and reliability fixes matter as much as new features.

## Before opening a PR

1. Open an issue for large changes and refer to the [PRD](docs/PRD_Gestouch_v1.0.md) and [implementation matrix](docs/IMPLEMENTATION_MATRIX.md).
2. Follow [developer setup](README.md). On Windows, run `powershell -ExecutionPolicy Bypass -File .\development\build.ps1` from the repo root.
3. Include synthetic tests for gesture logic and, for input changes, device model, Windows build, HID-related behavior, power/resume results, and an honest statement of what was not tested.
4. Avoid adding invasive hooks, elevation, arbitrary plugin loading, telemetry, external services, or a permanent heavyweight UI process without security/performance discussion.
5. Keep README claims grounded in a tested implementation. Design mockups are not product screenshots.

## License and provenance

Gestouch source is offered under **GNU GPL v2.0 only (`GPL-2.0-only`)**. By contributing, you agree to license your contributions under that project's license. You retain copyright to your contributions. Do not paste third-party code or artwork without proof of permission, attribution, and compatible licensing. The codebase currently contains an independent implementation inspired by [GestureSign V2](https://github.com/Tomclanc/GestureSignv2); do not misrepresent it as an upstream source fork. See [provenance](docs/PROVENANCE.md).

## Pull request checklist

- [ ] Matches repository style and license policy.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo test --workspace --target x86_64-pc-windows-msvc` passes (or clearly states why not).
- [ ] Build runs with `development/build.ps1` on Windows (or marks a specific blocker).
- [ ] No new dependency without license/security review.
- [ ] No misleading RAM, CPU, battery, or compatibility claims.
- [ ] Relevant docs and tests updated.

Please report security issues privately rather than posting exploit details in public issues; see the root [SECURITY.md](../SECURITY.md).
