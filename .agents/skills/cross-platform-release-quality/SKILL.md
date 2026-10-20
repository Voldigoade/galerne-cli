---
name: cross-platform-release-quality
description: Use when configuring CI/CD pipelines, release packaging, cross-platform validation (Windows and Linux), linting, dependency audits, or binary size checks for Rust projects.
---

# Cross-Platform Release Quality

Standard operating procedures for shipping high-polish, reproducible, audited Rust applications targeting both Windows and Linux.

## Release Readiness Checklist

Before tagging any release or declaring milestone completion:

1. **Formatting & Style**:
   ```bash
   cargo fmt --all -- --check
   ```
2. **Strict Clippy Lints**:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Comprehensive Test Suite**:
   ```bash
   cargo test --all-targets --all-features
   cargo test --doc
   ```
4. **Documentation Check**:
   ```bash
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   ```
5. **Dependency & License Audit**:
   - Security vulnerabilities: `cargo audit`
   - License and dependency bans: `cargo deny check` (ensure no unexpected copyleft/GPL licenses if releasing under MIT/Apache-2.0, no duplicate crates)
6. **Binary Size & Stripping**:
   ```bash
   cargo build --release
   ls -lh target/release/<bin_name>
   ```
   Ensure symbols are stripped and binary size conforms to budget constraints.

## Cross-Platform Matrix (Linux & Windows)

Applications must be verified on both Linux and Windows environments:

- **Path Separators & Line Endings**:
  - Use `std::path::Path` and `std::path::PathBuf`. Never hardcode `/` or `\\`.
  - Handle both CRLF (`\r\n`) and LF (`\n`) in input parsing and file reading.
- **Terminal Capabilities**:
  - Test on Linux (VT100/xterm, UTF-8 locale, standard POSIX signals).
  - Test on Windows (Windows Terminal, standard console host `conhost.exe`, PowerShell, CMD).
- **Continuous Integration Matrix**:
  - Run CI on both `ubuntu-latest` and `windows-latest`.
  - Build and run the full test suite on both OS runners for all pull requests.

## Release Packaging & Artifact Hygiene

- **Distribution Archives**:
  - Linux: `.tar.gz` containing stripped binary, `README.md`, `LICENSE`.
  - Windows: `.zip` containing `.exe`, `README.md`, `LICENSE`.
- **Zero Generated Junk**:
  - Never include debug symbol files (`.pdb`), intermediate build objects, temporary caches, test artifacts, or local configuration files in release archives.
- **Reproducible Builds**:
  - Commit `Cargo.lock` for binary crates to ensure deterministic dependency resolution.

## Performance & Regression Benchmarks

- Run automated benchmarks (`cargo bench`) against baseline measurements before releasing major versions.
- Measure and record resident memory (RSS), peak memory, and binary size across releases.
- Flag any unexpected bloat (>10% size increase or memory regression) for mandatory investigation.
