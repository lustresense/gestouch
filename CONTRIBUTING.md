# Contributing

Gestouch is an independently implemented project. Contributions must be compatible with **MIT OR Apache-2.0**, with no pasted/translated code from copyleft projects unless relicensing rights are explicitly established.

Before opening a pull request, read `docs/ARCHITECTURE.md`, `docs/IMPLEMENTATION_MATRIX.md` and `docs/KNOWN_LIMITATIONS.md`. Run `cargo fmt --check` and `cargo test --workspace --target x86_64-pc-windows-msvc` on Windows. Hardware-dependent input changes should include synthetic tests, hardware model, firmware/Windows build, evidence of no duplicate firing, sleep/resume behavior and performance impact. Avoid invasive hooks, unsafe pointer assumptions and privilege escalation.

Open an issue before adding external plugin loaders, kernel drivers, telemetry or resident GUI runtimes.
