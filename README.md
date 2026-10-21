# Galerne

An ultra-lightweight, persistent native Rust terminal game designed explicitly around developer wait times.

```text
╭─────────────────────────────────────────────────────────────────────────────╮
│                               GALERNE v0.2.1                                │
├──────────────────────┬──────────────────────────────────────────────────────┤
│ ▶ PLAY               │  DEPLOYMENT CONTROLS                                 │
│   WORKSHOP           │                                                      │
│   STATS              │  Active Operation in Progress:                       │
│   SAVES              │  Wave 3  |  Level 4  |  HP 75/120                    │
│   DOCTOR             │                                                      │
│   SETTINGS           │  Press Enter to Resume Operation.                    │
│   HELP               │                                                      │
│   EXIT               │                                                      │
├──────────────────────┴──────────────────────────────────────────────────────┤
│ ↑/↓: navigate menu   ←/→: workshop items   Enter: select   Esc/q: exit      │
╰─────────────────────────────────────────────────────────────────────────────╯
```

When you trigger long development operations (`cargo build`, `cargo test`, `npm install`, `docker build`), Galerne activates seamlessly in your terminal. You get an immediate, satisfying top-down survivor combat break. When your compilation completes, Galerne saves your progression, shows a brief transition, and restores your terminal output with zero broken state.

---

## Measured Performance & Budgets

All metrics are measured empirically from release binaries on standard Linux x86_64 developer hardware (Windows x86_64 CI validations run on each commit; native Windows benchmarks will be documented separately upon empirical host profiling):

| Metric | Measured Value (Linux) | Constraint Budget |
| :--- | :--- | :--- |
| **Stripped Binary Size** | **878 KiB** | < 5.0 MiB |
| **Startup Latency** | **< 2.0 ms** | < 15.0 ms |
| **Resident Memory (RSS)** | **3.10 MiB** | < 20.0 MiB |
| **Idle CPU Utilization** | **0.0%** (no daemon) | 0.0% |
| **Typical Save Size** | **434 bytes** | < 5.0 KiB |
| **Diff Renderer Latency** | **4.46 µs / frame** (224k fps throughput) | Real-time |
| **In-Memory Save Serialization** | **0.04 µs / op** (25.3M ops/sec) | Microbenchmark |
| **Full Atomic Disk Save I/O** | **29.11 µs / save** (34.3k saves/sec) | < 5.0 ms |
| **Wrapping Latency Overhead** | **10.59 ms** | < 25.0 ms |

- **Zero Resident Daemon**: Galerne runs only when you play or execute tasks. No background processes or battery drain.
- **Fully Offline**: Zero telemetry, zero analytics, zero network requests during normal use.

---

## Two-Tier Interrupt UX (Critical Invariant)

When playing during a compilation task:

```text
cargo build
    │
    ▼
[ Galerne Game View ]
    │
    ├─► Press Ctrl+C or Esc / q (First interrupt)
    │     ├── Game view closes instantly
    │     ├── Exact run state and progression are persisted
    │     ├── Terminal cooked mode is cleanly restored
    │     └── cargo build CONTINUES RUNNING in live log view
    │
    ▼
[ Task Log View ]
    │
    ├─► Arriving output streams live to stdout
    │
    └─► Press Ctrl+C (Second interrupt)
          └── Cancels the supervised build command
```

Platform interrupt handling:
- **Linux / Unix:** Interrupts gracefully via native `SIGINT` forwarded directly to the child process group (`kill(-pid, SIGINT)`).
- **Windows:** Cancels via process-tree termination (`taskkill /PID <pid> /T /F`), ensuring the entire spawned compiler and linker process tree (`cargo`, `rustc`, `link.exe`, etc.) is cleanly terminated without orphan tasks.

---

## Installation

### Windows (x86_64 Recommended Native Installer)

Download and run **`Galerne-Setup-x86_64.exe`** from the [GitHub Releases](https://github.com/Voldigoade/galerne/releases/latest) page. The installer automatically configures your user PATH and sets up PowerShell integration (supporting both Windows PowerShell 5.1 and modern PowerShell 7+).

Alternatively, install via PowerShell:

```powershell
irm https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.ps1 | iex
```

### Linux (x86_64 One-Line Terminal Install)

```bash
curl -sSL https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.sh | sh
```

*(Note: Prebuilt release artifacts are provided for Linux x86_64 and Windows x86_64. For macOS, aarch64, or custom environments, see [Building From Source](#building-from-source).)*

### Building From Source

Prerequisites: Rust 1.80+ and Cargo.

```bash
git clone https://github.com/Voldigoade/galerne.git
cd galerne
cargo build --release
cp target/release/galerne ~/.local/bin/  # Or add target/release to PATH
galerne setup
```

---

## Usage

### Interactive Control Center

Running `galerne` with no arguments opens the interactive dashboard:

```bash
galerne
```

### Direct CLI Commands

```bash
galerne play               # Direct manual game deployment (bypasses dashboard)
galerne stats              # Display career metrics and wait time saved
galerne doctor             # Run live read-only installation health diagnostics
galerne saves              # Inspect save files and archived snapshots
galerne reset [-y|--yes]   # Archive current progress and initialize clean state
galerne restore <archive>  # Restore a previous archived save snapshot
galerne setup              # Configure shell hooks (Bash, Zsh, PowerShell)
galerne enable             # Enable automatic wait-time activation
galerne disable            # Disable automatic activation (preserves saves)
galerne update             # Safely install latest release with SHA-256 verification
galerne update --check     # Check for newer releases without modifying system
galerne uninstall          # Reversibly remove shell hooks and files
galerne wrap <command...>  # Explicitly supervise a long-running build command
```

---

## Smart Activation & Wait-Time Prediction

Galerne automatically detects and intercepts eligible developer build operations:

- **Rust:** `cargo build`, `cargo test`, `cargo clippy`, `cargo check`, `cargo bench`
- **JavaScript / Node:** `npm install`, `npm run build`, `npm test`, `pnpm install`, `yarn install`
- **Containers:** `docker build`, `docker compose build`
- **Native:** `make`, `ninja`, `cmake --build`
- **Go & Python:** `go build`, `go test`, `pytest`

### Project-Aware Intelligence
- **Project Isolation:** Predictions are strictly scoped to `(Project + Command)`. A 45-second build in a large repository never causes a 2-second build in a smaller project to launch Galerne.
- **Live Output Streaming:** Normal task output remains visible live during pre-game observation; commands such as `cargo build` never appear frozen or delayed.
- **Pre-Game Interruption:** Hitting Ctrl+C before gameplay appears cancels the child task immediately without swallowing the signal or opening the game.
- **5-Second Wait Protection:** Unknown or new commands use a conservative fallback threshold of 8.0 seconds. Any command completing within ~5 seconds completes normally with **zero game appearance or terminal flicker**.
- **Known-Long Operations:** Operations historically taking ≥ 8.0 seconds enter gameplay almost immediately (< 150 ms anti-flicker guard).
- **Graceful Extension:** Fast operations that unexpectedly take long (e.g., clean rebuilds) will smoothly activate once the 8.0s threshold is crossed.
- **True Duration History:** History records actual child termination time even if the user leaves game view early to inspect logs in Task View.
- **Cancellation Immunity:** Tasks interrupted by the user (Ctrl+C on any path, SIGINT, or exit code 130) are excluded from duration history, preventing aborted tasks from corrupting future predictions.
- **Privacy Protection:** Project identities are derived from local persistent 32-byte OS random salts and SHA-256 hashes. Raw absolute file paths and arbitrary command-line arguments are **never stored**.
- **Strictly Bounded Storage:** Prediction history on disk is strictly enforced to $< 10\text{ KiB}$ through LRU entry pruning.
- **Interactive Exclusion:** Interactive commands (`cargo run`, REPLs, `git`, `vim`) and watch modes (`--watch`) **strictly bypass** Galerne with inherited stdio.

---

## Safe Self-Update (`galerne update`)

Galerne features an explicit, offline-by-default self-updater:
- **No Background Daemons:** Zero background update checking, zero polling, and zero network calls during gameplay or normal commands.
- **SHA-256 Integrity Verification:** Every downloaded payload is validated against release checksums before modification.
- **Linux:** Performs atomic binary replacement (`temp sibling + rename`) with automated rollback if the replacement fails smoke testing.
- **Windows:** Safely hands off to `Galerne-Setup-x86_64.exe --upgrade --wait-pid <PID>`, waiting for the active process to exit before replacing the locked binary.
- **Data Preservation:** Save progression, workshop upgrades, and user settings are 100% preserved across upgrades.

*(Note: Users on v0.1.1 must install v0.2.0 once manually; all future updates can then be performed seamlessly via `galerne update`.)*

---

## Controls

- **Movement:** Arrow keys or `WASD`
- **Combat:** Gale Blaster automatically fires at the nearest adversary in range.
- **Upgrades:** On level up, select from 3 distinct in-run upgrades (`↑`/`↓` and `Enter`).
- **Exit / Suspend:** `Ctrl+C` or `Esc` or `q` (persists run progress and restores terminal).

---

## Save Integrity & Recovery

Local state is stored in:
- **Linux:** `~/.local/share/galerne/save.dat`
- **Windows:** `%LOCALAPPDATA%\galerne\save.dat`

### Reliability Guarantees
- **Atomic Writes:** Every write targets a temporary sibling file, flushes with `sync_all()`, and atomically renames over the target path.
- **CRC32 Checksums:** Save files contain a 16-byte binary header (`b"GLRN"`, version, CRC32, length). Torn writes or bit corruption are detected prior to deserialization.
- **Automatic Quarantine & Recovery:** If primary save fails checksum validation, it is quarantined to `save.dat.corrupt.<timestamp>` and restored from `save.dat.bak`.
- **Safe Reset:** `galerne reset` archives your previous career to `archive_<timestamp>.dat` before initializing a fresh state, allowing full recovery anytime via `galerne saves`.

---

## Diagnostic Verification (`galerne doctor`)

Run read-only diagnostics at any time to verify system health:

```bash
$ galerne doctor
Galerne v0.2.1 System Diagnostics

[✓] Executable:        /usr/local/bin/galerne (v0.2.1)
[✓] Target Platform:   linux x86_64
[✓] Standard Output:   Interactive TTY
[✓] Terminal Size:     80x24 (Meets 60x20 minimum)
[✓] Color Depth:       ANSI 16 / 256 Color Supported
[✓] Data Storage:      ~/.local/share/galerne (Writable)
[✓] Save Integrity:    CRC32 Verified
[✓] Shell Hook:        Active in Bash (~/.bashrc)
[✓] Resident Daemons:  None (Zero resident memory / 0.0% idle CPU)
[✓] Network Policy:    Fully Offline (Zero telemetry, analytics, or ads)

Diagnostic verification complete. All essential invariants verified.
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.
