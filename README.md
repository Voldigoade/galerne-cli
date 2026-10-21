# Galerne

An ultra-lightweight, persistent native Rust terminal game designed explicitly around developer wait times.

```text
╭─────────────────────────────────────────────────────────────────────────────╮
│                               GALERNE v0.1.0                                │
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

All metrics are measured empirically from release binaries on standard developer hardware (Linux x86_64 and Windows x86_64):

| Metric | Measured Value | Constraint Budget |
| :--- | :--- | :--- |
| **Stripped Binary Size** | **798 KiB** | < 5.0 MiB |
| **Startup Latency** | **< 2.0 ms** | < 15.0 ms |
| **Resident Memory (RSS)** | **3.06 MiB** | < 20.0 MiB |
| **Idle CPU Utilization** | **0.0%** (no daemon) | 0.0% |
| **Typical Save Size** | **434 bytes** | < 5.0 KiB |
| **Diff Renderer Latency** | **4.40 µs / frame** (227k fps throughput) | Real-time |
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
    ├─► Press Ctrl+C (First interrupt)
    │     ├── Game view closes instantly
    │     ├── Progression is persisted
    │     ├── Terminal is cleanly restored
    │     └── cargo build CONTINUES RUNNING in log view
    │
    ▼
[ Task Log View ]
    │
    └─► Press Ctrl+C (Second interrupt)
          └── Cancels the cargo build natively (SIGINT)
```

---

## Installation

### Linux & macOS (One-Line Terminal Install)

```bash
curl -sSL https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.ps1 | iex
```

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
galerne play               # Direct manual game deployment
galerne stats              # Display career metrics and wait time saved
galerne doctor             # Run read-only installation health diagnostics
galerne saves              # Inspect save files and archived snapshots
galerne reset              # Archive current progress and initialize clean state
galerne setup              # Configure shell hooks (Bash, Zsh, PowerShell)
galerne enable             # Enable automatic wait-time activation
galerne disable            # Disable automatic activation (preserves saves)
galerne uninstall          # Reversibly remove shell hooks and files
galerne wrap <command...>  # Explicitly supervise a long-running build command
```

---

## Supported Task Detection

Galerne automatically detects and intercepts eligible developer build operations:

- **Rust:** `cargo build`, `cargo test`, `cargo clippy`, `cargo check`, `cargo bench`
- **JavaScript / Node:** `npm install`, `npm run build`, `npm test`, `pnpm install`, `yarn install`
- **Containers:** `docker build`, `docker compose build`
- **Native:** `make`, `ninja`, `cmake --build`
- **Go & Python:** `go build`, `go test`, `pytest`

### Anti-Flicker & Duration Prediction

- Commands historically taking longer than 8 seconds enter gameplay almost immediately (< 150 ms).
- Fast or unknown commands observe a 2.5-second anti-flicker delay. If your build finishes within 2.5 seconds, Galerne never opens, producing **zero visual flicker**.
- Interactive commands (`cargo run`, REPLs, `git`, `vim`) and watch modes (`--watch`) are **strictly excluded**.

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
Galerne v0.1.0 System Diagnostics

[✓] Executable:        /usr/local/bin/galerne (v0.1.0)
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
