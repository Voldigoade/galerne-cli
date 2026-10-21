> **Historical foundation plan:** This document describes an early runner prototype and is not an accepted final game design. The current branch continues toward a persistent adventure.

# Galerne Gameplay Reboot: Early Traversal Foundation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the rejected survivor-arena prototype with an early terminal-native traversal foundation featuring automatic forward motion, discrete action controls with input buffering and coyote time, authored multi-cell ASCII art, parallax scrolling, non-blocking pickups, and an embedded i18n foundation (EN, FR, ES, zh-CN). This plan records prototype work, not accepted final game design.

**Architecture:** A fixed-timestep simulation runner (~30 FPS) with decoupled ASCII diff-rendering. Chunks are generated deterministically ahead and pruned behind. Player actions (Jump, Slide, Dash) use state-machine animations with coyote time and input queuing. Textual UI is localized across 4 languages with auto-detection.

**Tech Stack:** Rust 2021, crossterm (events, alternate screen), serde, serde_json, crc32fast, ureq, sha2, getrandom. Zero new heavy dependencies.

---

## Global Constraints

- Native Rust only. No game engine dependencies, no audio dependencies in V1.
- Both Windows and Linux remain first-class targets with identical behavior.
- Stripped binary size budget < 5.0 MiB (current: ~2.6 MiB uncompressed, 878 KiB stripped).
- Active resident memory < 20.0 MiB, 0.0% idle CPU.
- Two-tier Ctrl+C invariant, smart activation, updater, and shell integration must remain 100% intact.
- No public release tag (v0.3.0) or GitHub release before explicit human playtest approval.

---

## Review Focus

1. **Terminal Input Repeat Immunity:** Terminal key repeats must never cause jerky movement; actions are discrete state-machine animations triggered on initial keydown.
2. **Readability Across Terminal Sizes:** Layout and parallax layers must adapt gracefully from minimum 80x24 up to 236x53 without empty void stretches.
3. **No Mid-Run Freezing:** Zero upgrade modals or text dialogues during active gameplay. Pickups apply instantly.
4. **Collision Fairness:** Multi-cell ASCII hitboxes must be slightly forgiving (inner bounding box) so near-misses feel thrilling rather than unfair.
5. **Safe Progression Migration:** v1 save files must migrate to v2 preserving all career time, shards, workshop upgrades, and settings.

---

## Task Decomposition

### Task 1: Lightweight Localization Infrastructure (`src/ui/i18n.rs`)
- [ ] Define `Locale` enum: `En`, `Fr`, `Es`, `ZhCn`.
- [ ] Implement locale detection from environment (`LANG`, `LC_ALL`, `LC_MESSAGES`, and Windows locale).
- [ ] Embed translation dictionaries for all UI text: dashboard, settings, help, workshop, stats, doctor, and onboarding prompts.
- [ ] Add unit tests verifying complete key coverage across all 4 locales with zero missing keys.

### Task 2: Runner Movement & Action State Machine (`src/game/runner/player.rs`, `src/game/runner/action.rs`)
- [ ] Define `PlayerAction`: `Running`, `Jumping(JumpState)`, `Sliding(SlideState)`, `Dashing(DashState)`.
- [ ] Implement discrete jump trajectory with launch, smooth arc, apex float, gravity fall, and landing frame.
- [ ] Implement coyote time (100ms grace window after leaving ground).
- [ ] Implement input buffering (queuing action 150ms before landing/recovery).
- [ ] Implement slide state (reduced vertical hitbox, duration, recovery).
- [ ] Implement dash state (speed burst, invulnerability, cooldown).
- [ ] Authored multi-cell ASCII character sprites (5x4 running cycle, jump frames, slide, dash).
- [ ] Add deterministic simulation unit tests for jump arcs, coyote time, and input buffering.

### Task 3: Parallax World & Deterministic Chunk Generator (`src/game/runner/world.rs`, `src/game/runner/parallax.rs`)
- [ ] Build 4-layer parallax engine:
  - Layer 1: Sky / Clouds / Wind streaks (0.1x speed)
  - Layer 2: Distant skyline / pylons / mountains (0.25x speed)
  - Layer 3: Midground gantries / powerlines / trees (0.5x speed)
  - Layer 4: Ground tracks / platforms (1.0x speed)
- [ ] Deterministic chunk-based generation with seed:
  - Chunk size ~50 columns.
  - Procedural obstacle placement with solvability guarantees.
  - Chunk recycling to prevent unbounded memory.
- [ ] Responsive width scaling (80 to 240 columns).
- [ ] Add unit tests for deterministic chunk generation and memory bound enforcement.

### Task 4: Obstacles, Non-Blocking Pickups & Collision (`src/game/runner/obstacles.rs`, `src/game/runner/items.rs`)
- [ ] Authored multi-cell ASCII obstacles:
  - Low barriers / debris (jump over)
  - Overhead pipes / steam ducts (slide under)
  - Broken tracks / gap hazards (jump across)
- [ ] Instant-apply pickups:
  - Gale Shards `[*]`
  - Speed Burst `[>>]`
  - Wind Shield `(O)`
  - Shard Magnet `[+]`
  - Score Multiplier `[x2]`
- [ ] Forgiving bounding-box collision detection.
- [ ] Floating particle feedback for collections.
- [ ] Add unit tests for collision detection and instant pickup effects.

### Task 5: Runner Game State, HUD & Session Pacing (`src/game/runner/state.rs`, `src/game/runner/mod.rs`)
- [ ] Integrate player, world, obstacles, and pickups into `RunnerState`.
- [ ] Pacing curve: gentle first 10s -> dynamic 10-30s -> intense 30-60s+.
- [ ] Minimal non-blocking HUD (Distance, Speed, Shards, Multiplier).
- [ ] Quick onboarding prompt overlay: `SPACE  JUMP` on first run, auto-clearing upon success.
- [ ] 200ms smooth build-completion deceleration transition.
- [ ] Swap `src/game/mod.rs` to expose the new runner as the primary game engine.
- [ ] Add unit tests for complete runner game loops.

### Task 6: Save Schema Migration v1 -> v2 (`src/storage/schema.rs`, `src/storage/migration.rs`)
- [ ] Update `CURRENT_SCHEMA_VERSION` to 2.
- [ ] Add `RunnerSnapshot` to `SaveState` for exact/deterministic runner resumption.
- [ ] Implement `migrate_save` for v1 -> v2: preserves career stats, shards, workshop upgrades, settings; archives legacy arena active-run.
- [ ] Update `workshop.rs` to map permanent upgrades to runner attributes (extra shield capacity, magnet radius, shard bonus, dash cooldown).
- [ ] Add unit tests for v1 -> v2 save migration and career data integrity.

### Task 7: UI & Dashboard Integration with Internationalization (`src/ui/dashboard.rs`)
- [ ] Update dashboard to use `t(key, locale)` for all screen text.
- [ ] Add `Settings -> Language` toggle (Auto, English, Français, Español, 简体中文).
- [ ] Connect `galerne play` and task supervisor to the new runner engine.
- [ ] Polish layout for high-column and standard terminals.

### Task 8: Visual QA & Visual Frame Snapshots (`tests/visual_qa_test.rs`)
- [ ] Build visual frame renderer capturing rendered ASCII frames.
- [ ] Inspect and verify frames:
  - Running animation cycle
  - Jump ascent, apex, landing
  - Slide under low overhead pipe
  - 80x24 standard terminal vs 200+ column wide terminal
- [ ] Confirm artwork readability, contrast, and aesthetic quality.

### Task 9: Full Quality Gates, Regression Testing & Playtest Build
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy --all-targets -- -D warnings`.
- [ ] Run full test suite (all new + existing infrastructure tests).
- [ ] Verify binary size (< 5.0 MiB) and execution metrics.
- [ ] Build optimized release binary on `gameplay-reboot` branch.
- [ ] Provide user with exact playtest instructions for human evaluation.
