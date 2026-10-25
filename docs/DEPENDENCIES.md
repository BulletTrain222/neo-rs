# Dependency setup (implementation workspace)

Read AGENTS.md and docs/STATUS.md first. This is the canonical setup/download guide for the clean-room implementation workspace. Inspect installed tools before downloading replacements. Use official sources below; record actual installed versions and paths in private/. Do not obtain, copy, read, or configure a PS2 BIOS. Do not follow the upstream kit's emulator/BIOS/virtual-controller setup steps for this project.

## Build and run the rewrite on Windows

| Dependency | Official source | Installation and check |
| --- | --- | --- |
| Rust + Cargo, MSVC host | https://rustup.rs/ | Install the Windows x64 rustup toolchain. This workspace was verified with rustc 1.99.0. Check `rustc --version` and `cargo --version`. Keep Cargo.lock. |
| Visual Studio C++ Build Tools | https://visualstudio.microsoft.com/visual-cpp-build-tools/ | Install Desktop development with C++, including MSVC x64/x86 and a Windows SDK. The launcher uses vswhere to locate vcvars64.bat. |
| Bevy and Rust dependencies | Cargo.toml and Cargo.lock in this folder | Cargo downloads these automatically on the first online build. Bevy is pinned to 0.19.1. neopets-data uses the RustCrypto `sha2` 0.10 crate for disc fingerprints. Bevy's `bevy_audio` feature is enabled for sound output (it brings in `rodio` 0.22 and `cpal` from crates.io); the game's audio is decoded by this workspace's own code from the ISO and handed to it as PCM, so no audio format decoders are enabled. No separate Bevy editor/engine download is needed. Do not upgrade the engine merely to set up this project. |
| Git for Windows | https://git-scm.com/downloads/win | Needed for version control; not for reading the ISO. Check `git --version`. Do not publish automatically. |

Run from the workspace root:

```powershell
scripts/cargo.cmd fetch --locked
scripts/cargo.cmd test --locked
scripts/cargo.cmd build --locked -p neopets-app
```

These are online commands when caches are missing. `--offline` is only appropriate after dependencies have been cached. Rust/MSVC are system tools, not bundled binaries in this folder.

Use Set-ISO.cmd to select the user's existing USA 1.00 ISO. Do not download the game. Inspect-ISO.cmd validates it. The private ISO path from the original machine may be wrong after moving the folder.

## Already included as source

- third_party/ps2-float (0.3.0), MIT OR Apache-2.0. See docs/PROVENANCE.md.

Reverse-engineering tools (Python, Ghidra, Java, Emotion Engine extension, the oracle, the decomp kit) belong to the research workspace only and are documented in the research workspace. **Clean-room implementers do not install, open, or use them.**

## Explicitly not prerequisites

No PS2 BIOS, PCSX2, ViGEmBus, or virtual-controller driver is required for this workspace. No system-wide antivirus exclusions or global configuration changes are needed merely to set it up.
