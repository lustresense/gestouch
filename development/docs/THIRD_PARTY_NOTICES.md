# Third-party dependencies — review before release

The Rust source currently declares crates such as `serde` and `toml` in `crates/gestouch-core/Cargo.toml`. They are distributed by their own authors under their respective licenses; Gestouch's **GPL-2.0-only** choice does **not** change the license of any dependency.

The source ZIP does not vendor third-party dependency source. Cargo resolves/downloads dependencies on first build. **Before publishing binaries:** generate and commit `Cargo.lock`, inspect the resolved dependency graph and exact crate licenses with a reputable license scanner (`cargo-deny`/`cargo-license` or equivalent), check SPDX compatibility including transitive dependencies, generate an SBOM and preserve all required notices. No automatic blanket compatibility claim is made here.

Research references to GestureSign and Microsoft API documentation are recorded in [PROVENANCE.md](PROVENANCE.md) and [UPSTREAM_AUDIT.md](UPSTREAM_AUDIT.md); they are not bundled third-party libraries.
