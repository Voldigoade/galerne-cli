# Development & Verification Guide

This guide details the standard development workflow, quality gates, and verification commands required for this project.

## Build Commands

- **Debug Build**:
  ```bash
  cargo build
  ```
- **Optimized Release Build**:
  ```bash
  cargo build --release
  ```

## Verification & Quality Gates

Run these commands in order before submitting code or claiming completion of any task:

1. **Format Check**:
   ```bash
   cargo fmt --all -- --check
   ```
2. **Clippy Linter**:
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Automated Tests**:
   ```bash
   cargo test --all-targets --all-features
   cargo test --doc
   ```
4. **Documentation Check**:
   ```bash
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   ```

## Performance & Binary Inspection

- **Inspect Release Binary Size**:
  ```bash
  cargo build --release
  ls -lh target/release/<bin_name>
  size target/release/<bin_name>
  ```
- **Inspect Dependency Bloat**:
  ```bash
  cargo bloat --release --crates
  cargo bloat --release -n 20
  ```
- **Run Microbenchmarks**:
  ```bash
  cargo bench
  ```
- **Measure Peak Memory and Startup Time (Linux)**:
  ```bash
  /usr/bin/time -v target/release/<bin_name>
  ```

## Platform-Specific Verification

Both Windows and Linux are first-class targets.

### Linux
- Verify terminal restoration across Bash and Zsh.
- Test signal handling with `SIGINT` (Ctrl+C) and `SIGTERM`.
- Verify behavior when stdout is redirected to a pipe (`| cat` or `> /dev/null`).

### Windows
- Verify behavior in modern Windows Terminal and legacy Windows Console (`conhost.exe`).
- Test in PowerShell and CMD.
- Ensure terminal virtual terminal (VT) processing is cleanly enabled and restored.
- Verify path separator handling (forward slash vs backslash) and CRLF vs LF.

## Completion Checklist

Before declaring any implementation task complete:
- [ ] All code formatted (`cargo fmt --all -- --check`)
- [ ] Clippy clean with no warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] All unit, integration, and doc tests pass (`cargo test`)
- [ ] Terminal cleanup verified across normal, cancel (Ctrl+C), and error exits
- [ ] Binary size and memory footprint measured and compared against budgets
- [ ] Observable evidence attached to the task completion report
