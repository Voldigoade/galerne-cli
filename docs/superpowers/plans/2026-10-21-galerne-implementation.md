> **Historical, obsolete plan:** This implementation plan describes the rejected survivor-arena game. It is retained for history and is not an implementation guide for current work.

# Galerne Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an ultra-lightweight, persistent native Rust terminal game designed around developer wait times, featuring top-down survivor gameplay, exact session continuation, bulletproof terminal restoration, two-tier Ctrl+C handling, and measured single-digit MiB footprint.

**Architecture:** Monolithic native binary with modular domain components: a low-overhead double-buffer diff terminal renderer, a deterministic gameplay loop and physics engine, an atomic CRC32-validated local state persistence layer, a process supervisor with duration prediction and signal routing, and an interactive ASCII control center.

**Tech Stack:** Native Rust, `crossterm` (minimal features for raw terminal & events), `serde` + `serde_json` (state serialization), `crc32fast` (save integrity checksums). Zero Tokio, zero Electron, zero heavy runtimes.

**Spec:** `docs/superpowers/specs/2026-10-21-galerne-design.md`

## Global Constraints

- **Language:** Native Rust, formatted with `rustfmt`, passing `clippy -- -D warnings`.
- **Platforms:** First-class support for Linux x86_64 and Windows x86_64.
- **Dependencies:** Strictly lean; no runtime daemons, no network/cloud requirements, no Tokio, no audio system.
- **Resource Discipline:** Target < 15 MiB resident memory (RSS), < 15 ms startup latency, binary size < 5 MiB stripped.
- **Terminal Restoration:** Guaranteed restoration under all exit paths (normal, signal, panic, resize).
- **Two-Tier Interrupt:** `Ctrl+C` in game view exits game view only without terminating child task; `Ctrl+C` in log view interrupts child.

## Review Focus

1. **Terminal State Restoration on Panic / Force Kill:** Must cleanly leave alternate screen and disable raw mode.
2. **Child Process Failure Propagation:** Exit status of child command must be preserved identically (e.g. exit code 101 on rustc compile error).
3. **Save File Torn-Write Protection:** Sudden power loss or SIGKILL must never corrupt previous save; atomic write + backup quarantine.
4. **Sub-Terminal Boundary Checking:** Resizing terminal down to small sizes (< 60x20) must never crash or panic.
5. **Short-Command Anti-Flicker:** Rapid commands (< 1s) must not cause terminal flashing or flickering.

---

### Task 1: Project Scaffolding & Cargo Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `LICENSE` (MIT)

**Steps:**
- [ ] Configure `Cargo.toml` with lean dependencies (`crossterm` with minimal default features disabled, `serde`, `serde_json`, `crc32fast`).
- [ ] Configure release profile (`opt-level = 3`, `lto = "thin"`, `codegen-units = 1`, `panic = "abort"`, `strip = true`).
- [ ] Implement skeleton `src/lib.rs` and `src/main.rs` with version output.
- [ ] Verify `cargo check` and `cargo test`.
- [ ] Commit with message: `Initialize Cargo project with audited lean dependencies`.

---

### Task 2: Terminal Engine & Double-Buffer Diff Renderer

**Files:**
- Create: `src/terminal/buffer.rs`
- Create: `src/terminal/engine.rs`
- Create: `src/terminal/guard.rs`
- Create: `src/terminal/mod.rs`
- Test: `tests/terminal_buffer_test.rs`

**Steps:**
- [ ] Write failing test for `Buffer` 2D grid allocation, cell mutation, and diff generation.
- [ ] Implement `Cell` (`char`, `Color`, `Modifier`) and double-buffered `Buffer`.
- [ ] Implement `TerminalGuard` with RAII drop cleanup and panic hook installation.
- [ ] Implement `Engine` that computes frame diffs and flushes only mutated cells to `BufWriter<Stdout>`.
- [ ] Run tests and verify zero-allocation updates on identical frames.
- [ ] Commit with message: `Add double-buffer diff terminal renderer and RAII guard`.

---

### Task 3: Core Game Simulation & Entities

**Files:**
- Create: `src/game/arena.rs`
- Create: `src/game/player.rs`
- Create: `src/game/weapons.rs`
- Create: `src/game/enemies.rs`
- Create: `src/game/state.rs`
- Create: `src/game/mod.rs`
- Test: `tests/gameplay_simulation_test.rs`

**Steps:**
- [ ] Write failing tests for player movement, bounds clamping, auto-firing targeting, and enemy movement towards player.
- [ ] Implement `Arena` with dynamic terminal sizing and wall collision bounds.
- [ ] Implement `Player` entity with position `(f32, f32)`, velocity, health, and stats.
- [ ] Implement `Weapon` (Gale Blaster) with projectile physics and nearest-enemy targeting.
- [ ] Implement 4 distinct enemy archetypes: Drifter (slow floater), Swarmer (erratic rusher), Charger (telegraphed charge), Spitter (projectile firer).
- [ ] Implement Boss encounter: The Tempest with multi-phase attacks.
- [ ] Run simulation tests and verify deterministic step execution.
- [ ] Commit with message: `Implement core survivor gameplay simulation and enemy archetypes`.

---

### Task 4: In-Run Leveling, Upgrades & Combat Feedback

**Files:**
- Create: `src/game/upgrades.rs`
- Create: `src/game/particles.rs`
- Modify: `src/game/state.rs`
- Test: `tests/upgrades_test.rs`

**Steps:**
- [ ] Write failing tests for XP drop collection, level thresholds, and upgrade selection modifiers.
- [ ] Implement XP gem spawning upon enemy death and vacuum magnet collection.
- [ ] Implement upgrade draft system offering 3 randomized, distinct in-run upgrades (+Damage, +Speed, +Multishot, +Piercing, +Cooldown).
- [ ] Implement visual feedback: floating combat damage numbers and hit spark particles.
- [ ] Run tests and verify level curve and stat calculations.
- [ ] Commit with message: `Add XP leveling, in-run upgrade drafting, and combat particles`.

---

### Task 5: Save State, Integrity Header, CRC32 & Corruption Recovery

**Files:**
- Create: `src/storage/schema.rs`
- Create: `src/storage/atomic.rs`
- Create: `src/storage/manager.rs`
- Create: `src/storage/mod.rs`
- Test: `tests/save_integrity_test.rs`

**Steps:**
- [ ] Write failing tests for save serialization, CRC32 header verification, atomic write replacement, and corrupt file quarantine.
- [ ] Implement `SaveState` data structure (schema version, player progression, workshop unlocks, active run state, stats).
- [ ] Implement atomic writer: serialize -> append magic `b"GLRN"` + `version: u32` + `crc32: u32` -> write to `save.dat.tmp.<pid>` -> `sync_all()` -> `fs::rename`.
- [ ] Implement recovery engine: if primary save has checksum mismatch or corrupt header, rotate corrupt file to `save.dat.corrupt.<timestamp>` and recover from `.save.bak`.
- [ ] Run tests verifying that bit-flipped files never overwrite backups and trigger clean recovery.
- [ ] Commit with message: `Implement atomic save persistence with CRC32 integrity and recovery`.

---

### Task 6: Workshop Meta-Progression & Statistics

**Files:**
- Create: `src/storage/workshop.rs`
- Create: `src/storage/stats.rs`
- Modify: `src/storage/manager.rs`
- Test: `tests/workshop_test.rs`

**Steps:**
- [ ] Write failing tests for persistent currency (Gale Shards), workshop purchase costs, stat bonuses, and reset archiving.
- [ ] Implement `Workshop` upgrade nodes (Base Health, Starting Damage, Movement Speed, Magnet Radius, Shard Multiplier).
- [ ] Implement `StatsTracker` (cumulative play time, wait time saved, runs started, bosses defeated).
- [ ] Implement `archive_and_reset()` to safely back up current progress before fresh initialization.
- [ ] Run tests verifying progression math and archive integrity.
- [ ] Commit with message: `Implement workshop meta-progression and safe reset archives`.

---

### Task 7: Smart Command Detection, Matcher & Duration Prediction

**Files:**
- Create: `src/interceptor/matcher.rs`
- Create: `src/interceptor/history.rs`
- Create: `src/interceptor/mod.rs`
- Test: `tests/command_matcher_test.rs`

**Steps:**
- [ ] Write failing tests for command pattern matching (cargo, npm, docker, make) and exclusion rules (watch modes, interactive commands).
- [ ] Implement command parser and normalizer (e.g. `["cargo", "build", "--release"]` -> `cargo:build`).
- [ ] Implement `DurationHistory` with exponential moving average / rolling median per command class.
- [ ] Implement prediction logic: tasks historically > 8s enter gameplay immediately (< 200 ms); unknown tasks wait 2.5s anti-flicker delay.
- [ ] Run tests verifying command matching and duration estimation.
- [ ] Commit with message: `Implement smart task matching and duration prediction`.

---

### Task 8: Process Supervision, Two-Tier Ctrl+C Handler & Dissolve Transition

**Files:**
- Create: `src/interceptor/process.rs`
- Create: `src/interceptor/signals.rs`
- Create: `src/interceptor/transition.rs`
- Modify: `src/interceptor/mod.rs`
- Test: `tests/process_supervision_test.rs`

**Steps:**
- [ ] Write failing tests for process spawning, output ring buffer capture, and exit code propagation.
- [ ] Implement asynchronous/threaded process monitor reading stdout and stderr into bounded ring buffer (2,000 lines max).
- [ ] Implement two-tier signal router:
  - In GAME view: `Ctrl+C` suspends game view, saves state, drops alternate screen, switches to live task output; child continues running.
  - In TASK view: `Ctrl+C` sends standard interrupt to child.
- [ ] Implement terminal completion transition: 250ms matrix/dissolve animation, "+XP / +Shards" banner, followed by exact exit code propagation.
- [ ] Run integration tests verifying child success and failure handling.
- [ ] Commit with message: `Implement child process supervision with two-tier Ctrl+C handling`.

---

### Task 9: Interactive Control Center TUI & Direct CLI Subcommands

**Files:**
- Create: `src/ui/menu.rs`
- Create: `src/ui/dashboard.rs`
- Create: `src/ui/mod.rs`
- Modify: `src/main.rs`
- Test: `tests/cli_test.rs`

**Steps:**
- [ ] Write tests for CLI argument parsing (`play`, `stats`, `doctor`, `saves`, `reset`, `wrap`).
- [ ] Implement interactive home control center for bare invocation (`galerne`):
  - ASCII banner and status dashboard.
  - Arrow key navigation between Play, Workshop, Stats, Saves, Settings, Doctor, Exit.
- [ ] Implement direct CLI subcommand handlers with clean text output when called directly.
- [ ] Run CLI tests verifying argument dispatch.
- [ ] Commit with message: `Implement interactive control center and CLI subcommands`.

---

### Task 10: Shell Integration, Doctor & Safe Lifecycle

**Files:**
- Create: `src/shell/installer.rs`
- Create: `src/shell/doctor.rs`
- Create: `src/shell/mod.rs`
- Test: `tests/shell_integration_test.rs`

**Steps:**
- [ ] Write failing tests for shell wrapper script generation and idempotency.
- [ ] Implement shell hooks for Bash (`~/.bashrc`), Zsh (`~/.zshrc`), and PowerShell (`$PROFILE`) using reversible `# BEGIN GALERNE` marker blocks.
- [ ] Implement `galerne doctor` diagnostic checks: terminal TTY, color support, save dir permissions, PATH resolution, shell hook status.
- [ ] Implement `galerne enable`, `disable`, and `uninstall` commands with clean reversible removal.
- [ ] Run integration tests verifying non-destructive config file edits.
- [ ] Commit with message: `Add shell integration, doctor diagnostics, and lifecycle management`.

---

### Task 11: Release Profiles, Performance Benchmarking & CI/CD

**Files:**
- Create: `benches/engine_bench.rs`
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`
- Create: `scripts/install.sh`
- Create: `scripts/install.ps1`
- Create: `README.md`

**Steps:**
- [ ] Run release build, check binary size and strip symbols (`size target/release/galerne`).
- [ ] Run microbenchmarks on frame diff calculation and save serialization.
- [ ] Measure startup time and memory footprint (RSS) with `/usr/bin/time -v`.
- [ ] Configure GitHub Actions CI workflow for Linux and Windows (`ubuntu-latest`, `windows-latest`).
- [ ] Write transparent installation scripts for Linux (`curl | sh`) and Windows (`irm | iex`).
- [ ] Complete comprehensive README with empirical performance measurements, controls, and architecture documentation.
- [ ] Commit with message: `Configure release workflows, benchmarks, and installation scripts`.
