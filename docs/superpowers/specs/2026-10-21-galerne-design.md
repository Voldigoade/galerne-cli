# Galerne Design Specification

**Date:** 2026-10-21  
**Product Name:** Galerne  
**Binary / Command:** `galerne`  
**Targets:** Linux x86_64, Windows x86_64  

---

## 1. Intent & Purpose

Developers spend hours each week waiting for compilations (`cargo build`, `cargo test`), package installations (`npm install`), container builds (`docker build`), and test suites. Traditional attempts to ease this waiting either:
1. Steal excessive resources (bloated Electron apps, background daemons, heavy Python/Node runtimes), actively slowing down the compilation they are waiting on.
2. Force the developer to context-switch away from their active terminal window, losing track of when the build finishes.
3. Fail to restore terminal state upon exit or cancellation (`Ctrl+C`), leaving shells corrupted.

**Galerne** is an ultra-lightweight, persistent native Rust terminal game designed explicitly around developer wait times. It activates seamlessly during eligible development tasks, offers instant, satisfying top-down survivor gameplay for micro-sessions (5–30 seconds) or extended manual play, saves progression automatically, and restores the terminal instantly when the underlying task finishes.

---

## 2. Goals, Non-Goals, and Anti-Goals

### Goals
- **Instant Micro-Session Play**: Engaging within < 1 second. Immediate movement and automatic combat.
- **Ultra-Light Resource Footprint**:
  - Memory: < 15 MiB resident set size (RSS).
  - Startup latency: < 15 ms.
  - Zero resident daemon; zero background polling when inactive.
  - Minimal release binary size (< 5 MiB stripped, targeting ~2 MiB).
- **Exact Persistence & Session Continuity**: Interrupted runs can be resumed; meta-currency (Gale Shards) unlocks workshop enhancements.
- **Flawless Terminal Discipline**: Guaranteed terminal restoration (raw mode, cursor visibility, alternate screen) under all normal, signal, panic, and cancellation exit paths.
- **Two-Tier Interrupt Semantics**:
  - `Ctrl+C` while in game view leaves the game view and returns to the running task log without canceling the task.
  - `Ctrl+C` in the task log view interrupts the child task natively.
- **Smart Task Detection**: Historical duration estimation predicts task length so long tasks enter gameplay immediately, while fast tasks complete without visual flicker.
- **Robust Save Integrity**: Schema-versioned atomic writes (`tmp` + `sync` + `rename`), CRC32 validation, automatic quarantine of corrupt files, rotating known-good backups, recoverable reset archives.

### Non-Goals (V1)
- No audio engine or sound system.
- No network requirements, telemetry, analytics, or cloud saves.
- No multiplayer, matchmaking, or online leaderboards.
- No heavy frameworks (no Tokio runtime, no Electron, no WebViews).
- No speculative RPG systems (crafting grids, 50-item inventories).

### Anti-Goals
- **Never slow down the user's build**.
- **Never leave the terminal corrupted** (`stty sane` should never be needed).
- **Never swallow build failure exit codes**.
- **Never overwrite a known-good save file with corrupt data**.
- **No mobile dark patterns**: no timers, daily streaks, or fake grind.

---

## 3. Architecture & Subsystems

```
┌────────────────────────────────────────────────────────┐
│                        CLI / Main                      │
│   (Arg parsing, bare interactive menu vs subcommands)   │
└───────────┬────────────────────────────────┬───────────┘
            │                                │
            ▼                                ▼
┌───────────────────────┐        ┌───────────────────────┐
│     Direct Modes      │        │    Task Interceptor   │
│ (play, stats, doctor, │        │ (cargo/npm/docker wrapper,│
│  saves, setup, etc.)  │        │  duration history,    │
└───────────┬───────────┘        │  process supervision) │
            │                    └───────────┬───────────┘
            ▼                                │
┌────────────────────────────────────────────┼───────────┐
│                                            ▼           │
│                 Terminal Engine (Crossterm)             │
│   - RAII Guard & Panic Hook                            │
│   - Double-Buffer Diff Renderer                        │
│   - Input Polling & 2-Tier Ctrl+C Router               │
└─────────────────────┬──────────────────────────────────┘
                      │
                      ▼
┌────────────────────────────────────────────────────────┐
│                   Game Simulation                      │
│   - Arena & Coordinates                                │
│   - Player, Weapons, Projectiles                       │
│   - Enemy Archetypes (Drifter, Swarmer, Charger, Boss) │
│   - XP, Levels, In-run Upgrades                        │
└─────────────────────┬──────────────────────────────────┘
                      │
                      ▼
┌────────────────────────────────────────────────────────┐
│              Persistence & Save Integrity              │
│   - Atomic Writes (tmp -> sync -> rename)              │
│   - CRC32 Checksums & Magic Header                     │
│   - Quarantine & Backup Recovery                       │
│   - Workshop Meta-Progression & Archives               │
└────────────────────────────────────────────────────────┘
```

### 3.1 Terminal Engine & Double-Buffer Diff Renderer
- Uses minimal `crossterm` primitives (without heavy TUI dependencies).
- Frame buffer: 2D grid of `Cell { ch: char, fg: Color, bg: Color, bold: bool }`.
- Comparison: Compare current frame vs previous frame. Only write escape codes and glyphs for mutated cells.
- Buffered stdout with `std::io::BufWriter` flushed once per frame.
- Frame rate: ~25-30 FPS when active, 0% CPU when paused or waiting for key events in menus.
- Safe min-dimensions check (minimum 60x20): renders clean "Terminal too small" notice instead of panicking.
- `TerminalGuard` RAII struct disables raw mode, shows cursor, and exits alternate screen on drop. Global panic hook guarantees cleanup.

### 3.2 Gameplay Simulation (Survivor Action Core)
- **Arena**: Clamped 2D arena adapting to terminal bounds with ASCII border walls.
- **Player**: Coordinates `(f32, f32)`, smooth velocity vector, health points, XP meter, Gale Shards currency.
- **Weapon System**: Gale Blaster — automatically targets nearest enemy within range; fires projectiles at fixed cooldown. Upgradable fire rate, projectile speed, multishot, and damage.
- **Enemy Archetypes**:
  1. *Drifter* (`*`): Low health, steady floating movement toward player.
  2. *Swarmer* (`x`): Fast, low health, moves in quick erratic zig-zags.
  3. *Charger* (`#`): Telegraphed wind-up then straight-line high-speed rush.
  4. *Spitter* (`%`): Keeps distance, fires low-velocity projectiles.
  5. *Boss: The Tempest* (`Ω`): High health, cyclical attack phases (bullet ring, charge, minion summon).
- **In-Run Upgrades**: Upon leveling up, prompt 3 concise choices (e.g., +Damage, +Fire Rate, +Move Speed, +Multishot, +Pickup Radius).
- **Exact Run Resume**: Active run state (player pos/hp, wave timer, active enemies, seed) is stored on interruption and cleanly resumed.

### 3.3 Persistence & Save Integrity
- File location:
  - Linux: `$XDG_DATA_HOME/galerne/save.dat` (default `~/.local/share/galerne/save.dat`)
  - Windows: `%LOCALAPPDATA%\galerne\save.dat`
- Header:
  `[Magic b"GLRN" (4B)] [Version u32 (4B)] [Checksum u32 (4B)] [Payload Length u32 (4B)] [Payload bytes]`
- Serializer: Compact binary encoding (or lean JSON payload verified by CRC32).
- Atomic write: Write to `save.dat.tmp.<pid>`, `sync_all()`, atomic rename to `save.dat`.
- Backups: Keep `save.dat.bak`. If primary is invalid/corrupt, move primary to `save.dat.corrupt.<timestamp>`, restore `.bak`.
- Reset archive: `galerne reset` moves current save to `archive_<timestamp>.dat`, allowing full later restoration via `galerne saves`.

### 3.4 Smart Task Interceptor & Supervision
- Matches commands:
  - Rust: `cargo build`, `cargo test`, `cargo clippy`, `cargo check`
  - JavaScript: `npm install`, `pnpm install`, `yarn install`, `npm run build`
  - Containers: `docker build`, `docker compose build`
  - Native: `make`, `ninja`, `cmake --build`
- Excludes: REPLs, watch modes (`--watch`, `cargo watch`), interactive commands (`cargo run`).
- Duration Predictor:
  - Tracks rolling execution times per normalized task key (e.g. `cargo:build`, `npm:install`).
  - Tasks historically > 8 seconds activate gameplay immediately (< 200 ms).
  - Unknown tasks observe a 2.5-second anti-flicker delay before launching the game view.
- Log capture: Bounded ring buffer of child stdout/stderr (up to 2,000 lines / 512 KiB).
- Ctrl+C handling:
  - In game view: Pressing `Ctrl+C` suspends the game, saves state, restores terminal to normal buffer, and streams the running child process stdout/stderr. Child is NOT killed.
  - In task view: Pressing `Ctrl+C` sends SIGINT / Ctrl+C signal to child process normally.
- Task completion transition: Short 250ms ASCII collapse/dissolve effect followed by brief "+XP / +Shards" summary, then prints the child command's final output and exits with the child's exact exit code.

### 3.5 Interactive Control Center & CLI
- Bare invocation `galerne`:
  Opens ASCII dashboard with options:
  - Play (enter or resume game)
  - Progression (Workshop upgrades)
  - Stats (total wait time saved, runs played, enemies defeated)
  - Saves (backup & restore management)
  - Settings (color, animation speed, anti-flicker delay)
  - Doctor (diagnostic checks)
  - Exit
- CLI Subcommands:
  - `galerne play`: Direct launch into game.
  - `galerne stats`: Output summary metrics.
  - `galerne doctor`: Read-only health check.
  - `galerne saves`: List archives and backups.
  - `galerne reset`: Confirmed reset with mandatory archiving.
  - `galerne setup`: Inject shell hook into bash, zsh, or powershell.
  - `galerne enable` / `galerne disable`: Toggle automatic wrapping.
  - `galerne uninstall`: Reversibly remove shell integration and optionally purge data.
  - `galerne wrap <command...>`: Explicit wrapper command for tasks.

---

## 4. Verification & Testing Matrix
1. Unit tests for arena bounds, collision, projectile physics, and leveling.
2. Save system tests: round-trip, atomic writes, bit-flip corruption recovery, migration test, archive restore.
3. Process interceptor tests: exit code preservation, stdout/stderr capture, timeout, Ctrl+C event routing.
4. Terminal engine tests: diff algorithm correctness, buffer resizing, zero-allocation cell updates.
