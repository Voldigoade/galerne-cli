# Smart Activation & Safe Self-Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement project-aware smart wait-time prediction (no gameplay for ~5s tasks, near-immediate for historically long tasks) and an explicit, safe, verified `galerne update` system for Linux x86_64 and Windows x86_64.

**Architecture:**
- **Smart Activation:** Detect project root (Git repository root or working directory), hash with an installation-local salt into an opaque project ID without storing raw paths or raw commands. Maintain a bounded per-project duration history using robust medians with minimum evidence requirements. Short or unknown operations observe an 8.0s conservative threshold (so ~5s tasks never show Galerne), while known-long operations activate after a 150ms anti-flicker guard. Cancelled tasks never poison history.
- **Safe Self-Update:** Add `galerne update` and `galerne update --check`. Use lightweight HTTPS client (`minreq` with `https-rustls` and `sha2`) to query the official GitHub repository, verify SHA-256 against release metadata, atomically replace on Linux with smoke-test rollback, and delegate on Windows to `Galerne-Setup-x86_64.exe --upgrade --wait-pid <pid>` to safely replace the locked executable. Save data and settings are strictly preserved.
- **Save Migrations:** Establish an explicit migration framework with automatic pre-migration backups, schema version checks, and corrupt quarantine.

**Tech Stack:** Rust 2021, `minreq` (synchronous HTTPS with `https-rustls`), `sha2` (pure Rust SHA-256), `crossterm`, `serde`/`serde_json`, `crc32fast`.

**Spec:** User master mission prompt for Galerne V0.2.

## Global Constraints
- Native Rust only; no Electron, webviews, Node, Python, or Java.
- Linux x86_64 and Windows x86_64 are first-class targets.
- Stripped binary size < 5.0 MiB (current ~859 KiB).
- Zero resident daemons, zero background polling, zero network checks on normal startup or during gameplay.
- Explicit `galerne update` only; completely offline normal operation.
- Non-eligible commands (`cargo run`, `npm run dev`, `docker ps`, `go run`, `--watch`) must continue direct bypass.
- Two-tier Ctrl+C invariant strictly preserved.
- Format with `rustfmt`, clippy `-D warnings` clean.

## Review Focus
1. **~5-second build flicker:** An eligible build completing in 4.5 seconds on an unknown or short project must never display the alternate screen or flicker the terminal.
2. **Project isolation:** A 40-second build in Repository A must never cause a 2-second build in Repository B to immediately open Galerne.
3. **Cancelled task poisoning:** Pressing Ctrl+C 1.5 seconds into a 60-second build must not record 1.5 seconds into history and corrupt the median.
4. **Corrupted or mismatched update download:** A failed download or bad checksum must abort immediately, leaving the working executable 100% intact with zero partial files.
5. **Windows file-locking during update:** Windows updater must wait for the running `galerne.exe` PID to exit before writing the replacement binary.

---

## Tasks

### Task 1: Project Identity & Privacy-Safe Key Derivation
**Files:**
- Create: `src/interceptor/project.rs`
- Modify: `src/interceptor/mod.rs`
- Test: `tests/project_identity_test.rs`

- [ ] **Step 1:** Write failing tests in `tests/project_identity_test.rs` asserting:
  - Git repository root detection when inside a repository subdirectory.
  - Fallback to current working directory when not in a git repository.
  - Project ID is opaque hex and does NOT contain raw directory strings.
  - Salt is persistent and created under data dir.
  - Two distinct directories produce two distinct project IDs.
- [ ] **Step 2:** Implement `src/interceptor/project.rs` with `find_project_root` and `derive_project_id(root, salt)`.
- [ ] **Step 3:** Run tests and verify passing.
- [ ] **Step 4:** Commit: `git commit -m "Implement privacy-safe project identity derivation"`.

### Task 2: Project-Aware Bounded History & Activation Policy
**Files:**
- Modify: `src/interceptor/history.rs`
- Modify: `src/interceptor/supervisor.rs`
- Test: `tests/smart_activation_test.rs`

- [ ] **Step 1:** Write failing tests in `tests/smart_activation_test.rs` covering:
  - Unknown project + command: fallback threshold = 8.0s (a 5s run does not trigger).
  - Known-long project + command (median >= 8.0s with >= 2 samples): anti-flicker = 150ms.
  - Known-short project + command (median < 8.0s): stays hidden during normal run, but triggers if run exceeds 8.0s.
  - Two different projects with same `CargoBuild` maintain independent medians.
  - Cancelled tasks are NOT recorded.
  - History is bounded (max samples per key capped at 10; max tracked projects capped).
  - Backward compatibility: gracefully handles v0.1.1 `history.json` without crashing.
- [ ] **Step 2:** Refactor `src/interceptor/history.rs` to store `HashMap<String, ProjectTimingRecord>` keyed by `format!("{}:{}", project_id, class.label())`. Add `ActivationPolicy` state model (`KnownLong`, `KnownShort`, `Unknown`).
- [ ] **Step 3:** Update `src/interceptor/supervisor.rs` to resolve project ID, query activation policy, use appropriate anti-flicker guard (150ms vs 8000ms), and record durations ONLY on normal child completion (not on Ctrl+C cancellation).
- [ ] **Step 4:** Run all tests and verify passing.
- [ ] **Step 5:** Commit: `git commit -m "Implement project-aware smart activation and bounded history"`.

### Task 3: Save Schema Migration Framework
**Files:**
- Create: `src/storage/migration.rs`
- Modify: `src/storage/manager.rs`
- Modify: `src/storage/mod.rs`
- Test: `tests/save_migration_test.rs`

- [ ] **Step 1:** Write failing tests in `tests/save_migration_test.rs` validating:
  - Loading current v1 save works seamlessly.
  - Simulated schema migration from v1 to v2 creates a timestamped pre-migration backup before migration.
  - Migration preserves all career stats, workshop upgrades, active run, and settings.
  - Corrupted data is quarantined, and future schemas (> supported) return clean errors without overwriting.
- [ ] **Step 2:** Implement `src/storage/migration.rs` with `migrate_save_if_needed`.
- [ ] **Step 3:** Integrate migration hook into `SaveManager::load_file`.
- [ ] **Step 4:** Run all persistence tests and verify passing.
- [ ] **Step 5:** Commit: `git commit -m "Implement save schema migration framework with pre-migration backups"`.

### Task 4: Lightweight Network Client & Checksum Verification
**Files:**
- Modify: `Cargo.toml`
- Create: `src/updater/checksum.rs`
- Create: `src/updater/version.rs`
- Create: `src/updater/release.rs`
- Create: `src/updater/mod.rs`
- Test: `tests/updater_version_test.rs`
- Test: `tests/updater_checksum_test.rs`

- [ ] **Step 1:** Add `minreq` (with `https-rustls`) and `sha2` to `Cargo.toml`. Run `cargo check` and measure binary size.
- [ ] **Step 2:** Write tests in `tests/updater_version_test.rs` validating semantic version comparison:
  - `v0.10.0 > v0.9.0`
  - `v0.2.0 > v0.1.1`
  - Prereleases (`-rc1`, `-beta`) are flagged as prerelease.
  - Same version is detected as up to date.
- [ ] **Step 3:** Write tests in `tests/updater_checksum_test.rs` validating SHA-256 calculation and verification against `.sha256` files.
- [ ] **Step 4:** Implement `src/updater/version.rs`, `src/updater/checksum.rs`, `src/updater/release.rs`.
- [ ] **Step 5:** Run tests and verify passing.
- [ ] **Step 6:** Commit: `git commit -m "Add updater semantic versioning, release parsing, and SHA-256 verification"`.

### Task 5: Linux Atomic Update & Rollback Engine
**Files:**
- Create: `src/updater/linux.rs`
- Modify: `src/updater/mod.rs`
- Test: `tests/updater_linux_test.rs`

- [ ] **Step 1:** Write unit/integration tests in `tests/updater_linux_test.rs` validating:
  - Atomic replacement of binary file.
  - Rollback if new binary fails smoke test (`--version`).
  - Safe error reporting when target path is not writable.
  - Clean removal of temporary update artifacts.
- [ ] **Step 2:** Implement `src/updater/linux.rs`.
- [ ] **Step 3:** Run tests and verify passing.
- [ ] **Step 4:** Commit: `git commit -m "Implement Linux atomic binary update with automated rollback"`.

### Task 6: Windows Upgrade Engine in Native Setup
**Files:**
- Modify: `src/bin/setup.rs`
- Create: `src/updater/windows.rs`
- Modify: `src/updater/mod.rs`
- Test: `tests/updater_windows_test.rs`

- [ ] **Step 1:** Update `src/bin/setup.rs` to support `--upgrade --wait-pid <PID> --target <PATH>`. When invoked, waits for `<PID>` to terminate, replaces the target executable, verifies replacement, and cleans up.
- [ ] **Step 2:** Implement `src/updater/windows.rs` which downloads `Galerne-Setup-x86_64.exe`, verifies checksum, and launches it with `--upgrade --wait-pid <PID> --silent`.
- [ ] **Step 3:** Write tests in `tests/updater_windows_test.rs` simulating upgrade mode and pid waiting.
- [ ] **Step 4:** Run tests and verify passing.
- [ ] **Step 5:** Commit: `git commit -m "Implement Windows updater via setup --upgrade with PID handoff"`.

### Task 7: CLI Integration & Update Command (`galerne update` / `galerne update --check`)
**Files:**
- Modify: `src/ui/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Test: `tests/cli_test.rs`

- [ ] **Step 1:** Add `CliAction::Update { check_only: bool }` to `src/ui/cli.rs`.
- [ ] **Step 2:** Dispatch in `src/main.rs` to `galerne::updater::run_update(check_only)`.
- [ ] **Step 3:** Update `print_help()` to document `galerne update` and `galerne update --check`.
- [ ] **Step 4:** Test `galerne update --check` with mock/live sources.
- [ ] **Step 5:** Commit: `git commit -m "Integrate galerne update and galerne update --check into CLI"`.

### Task 8: Release Workflow & Distribution Assets Alignment
**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `Cargo.toml`
- Modify: `README.md`

- [ ] **Step 1:** Update `.github/workflows/release.yml` to publish standalone `galerne-x86_64-unknown-linux-gnu` alongside the tarball, with SHA-256 checksums for both Linux and Windows setup/binaries.
- [ ] **Step 2:** Update documentation in `README.md` detailing smart activation, 5s threshold, and `galerne update`.
- [ ] **Step 3:** Commit: `git commit -m "Align release workflow assets and documentation for v0.2.0"`.

### Task 9: Full Quality Gates, CI Verification, and Release v0.2.0
- [ ] **Step 1:** Run `cargo fmt --all -- --check`.
- [ ] **Step 2:** Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] **Step 3:** Run `cargo test --all-targets --all-features`.
- [ ] **Step 4:** Push to `main` and verify green GitHub CI on `ubuntu-latest` and `windows-latest`.
- [ ] **Step 5:** Tag `v0.2.0`, push tag, and verify GitHub Release workflow builds and uploads all assets.
- [ ] **Step 6:** Run live end-to-end update test verifying `galerne update --check` against published release.
