# Definitive Gameplay & World Vertical Slice Plan

## 1. Vision & Architecture
Transform Galerne from a generic runner prototype into a terminal-native side-scrolling atmospheric expedition:
- **Region:** *The Cinder Line Viaduct* — a monumental ancient transit spine spanning a windy rift, overgrown with hardy flora and retrofitted with suspended human dwellings.
- **Player:** The *Traveler* — authored multi-cell silhouette with wind cowl, flowing scarf, traversal pack with aerodynamic wind-fins.
- **Objective:** Traverse three distinct sectors, navigate route forks (Lower Pylons, Interior Machinery Tunnel, High Sky Truss), meet companion **Kael**, and restore **Relay Station 04 (The Gale Node)**.
- **Refuge:** A persistent base camp inside an abandoned railway gantry that visibly transforms and powers up once the Line is restored.
- **Technical Model:** 
  - Fixed-grid coordinate system with diff-buffer terminal rendering.
  - Layered depth engine scaling responsively up to 236x53.
  - Discrete action state machine + Tailwind build duration tiers + Flow rhythm.
  - Zero heap allocation per frame, bounded memory, full offline privacy.

---

## 2. Milestone Breakdown

### Phase A: Architecture & Schema
- Define World State & Save Schema v3:
  - `RegionProgress`: checkpoints reached, Line restored flag, route unlocked.
  - `Inventory`: compact array of real items (Relics, Components, Shards).
  - `CompanionState`: discovered, dialogue seen, present at refuge.
  - `RefugeState`: power level, facilities unlocked.
- Automatic migration from v2 to v3 with backup.

### Phase B: Art Engine & Mascot Character
- Rich ASCII sprite sheet for the Traveler:
  - Idle, Run (4 frames), Jump Ascent/Apex/Descent, Slide, Dash, Wind Glide, Activate, Hit, Respawn.
- Multi-layer visual horizon engine:
  - Sky/wind currents, distant megastructure silhouettes, midground viaduct arches/cables/shanties, ground track with sleepers, foreground silhouette accents.
  - Dynamic scaling from 80x24 to 236x53.

### Phase C: The Cinder Line Viaduct (Authored Region Traversal)
- Authored sector sequence:
  - Sector 1: *Lower Viaduct & Shanty Cables* (Checkpoints 1 & 2, low barriers, wind updrafts).
  - Sector 2: *Interior Gantry & Ventilation Shaft* (Quiet interior traversal, ceiling hazards, discovery cache).
  - Sector 3: *The High Truss & Observation Roost* (Branching path, companion encounter with Kael, wind bridge).
  - Sector 4: *The Gale Node (Relay Station 04)* (Final approach, traversal puzzle, activation mechanism).
- Checkpoints & safe respawn with zero inventory loss.

### Phase D: Wind Traversal & Flow
- Wind currents: updrafts and horizontal gales that lift and propel the Traveler.
- Tailwind Build Bonus: tiers I through IV based on smart task duration prediction, neutral on manual play.
- Flow multiplier system: reward clean vaults and glides with subtle visual audio-visual resonance.

### Phase E: Inventory, Discoveries & Companion Kael
- Item catalog: *Resonance Core*, *Induction Filament*, *Ancient Compass*, *Gale Shards*.
- Companion Kael: authored ASCII character, observation shelter, localized dialogue, moves to refuge upon Line restoration.
- Discovery: hidden maintenance chamber with ancient relic.

### Phase F: The Persistent Refuge
- Base camp view accessible via dashboard.
- Dual state: Dormant (cold, dark, abandoned) vs Restored (powered Line rails, glowing coils, Kael present).
- Workbench functionality for inspecting relics and refining traversal equipment.

### Phase G: Relay Activation & World Consequence
- Dramatic 1.5s ASCII transformation sequence when activating Relay Station 04:
  - Line power surge, beacon light ignites, world backdrop updates to show restored energy conduits.
  - Triggers permanent progression in the Refuge.

### Phase H: Code Cleanup, Comments & Hygiene
- Thorough audit of comments across all modified files: remove restatement comments, keep reasoning and invariant documentation.
- Remove obsolete runner remnants.

### Phase I: Quality Gates & Visual QA
- Capture deterministic ANSI/text frames for 80x24, 120x35, and 236x53.
- Run complete test suite, formatting, and clippy with `-D warnings`.
- Build release binary and measure size (< 3.5 MiB), RSS (< 20 MiB), and CPU.
- Prepare playtest build.
