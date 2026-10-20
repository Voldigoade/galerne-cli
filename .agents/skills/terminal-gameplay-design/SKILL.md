---
name: terminal-gameplay-design
description: Use when designing mechanics, game loops, session pacing, controls, visual feedback, or progression systems for a developer-oriented terminal game.
---

# Terminal Gameplay Design

Design guide for building genuinely enjoyable, high-respect terminal games tailored for developers. Developer games often run in micro-sessions (such as waiting for a compilation, test suite, or CI run) and demand zero friction.

## Core Design Principles

1. **Micro-Session Optimization (Seconds to Minutes)**:
   - A player may play for 5 seconds while a build completes, or for 5 minutes during a break.
   - The game must be instantly engaging within the first second of launching.
2. **Instant Interruption & Zero-Loss Resume**:
   - The player must be able to exit at any arbitrary millisecond (`Ctrl+C`, `q`, or terminal close).
   - The game must automatically persist an exact snapshot of state so the player loses zero progress upon interruption and resumes exactly where they stopped.
3. **Stand-Alone & Integrated Modes**:
   - The gameplay loop must be fully enjoyable when played intentionally as a standalone executable, not merely as a side-effect wrapper.
4. **Fun Core Loop First**:
   - Perfect the immediate tactile action-reaction loop before adding meta-progression, unlocks, or complexity. If the primary mechanic isn't compelling in isolation, meta systems will not save it.

## Cognitive Load & Controls

- **Low Cognitive Overhead**:
  - Keep rules clear and immediately discernible.
  - Avoid complex rulebooks, multi-paragraph tutorials, or obscure mechanics.
- **Immediate Feedback**:
  - Every input must result in instant visible or tactile consequences in the current frame.
  - Zero input lag; never block the rendering or event loop on I/O.
- **Frictionless Controls**:
  - Single-key actions: Arrow keys, WASD, Vim keys (`hjkl`), Space, Enter, `q`, `Esc`.
  - Avoid chord combinations (e.g. `Ctrl+Alt+Key`) or multi-key combos that conflict with terminal emulators or window managers.
- **Skip the Menu Maze**:
  - Launch directly into the action or present a clean, single-keypress start prompt. Avoid multiple sub-menus, confirmation dialogues, or configuration mazes.

## Visual Clarity & Terminal Aesthetics

- **Readable ASCII/Unicode Silhouettes**:
  - Prioritize visual contrast and readability over intricate ASCII art that turns into visual soup.
  - Key game entities, hazards, and resources must be instantly distinguishable at a glance.
  - Ensure legibility across both dark and light terminal color schemes.
- **Minimalist & Restrained Tone**:
  - Avoid meme overload, forced inside jokes, and patronizing developer tropes.
  - Keep UI copy clean, dry, and focused on the mechanics.

## Progression Without Dark Patterns

- **Meaningful Progression**:
  - Support cumulative progression (unlockable modifiers, persistent milestones, mastery tracking) that respects player time.
- **Anti-Patterns to Reject**:
  - **No Fake Depth**: Do not pad gameplay with arbitrary number grinding or superficial stat inflation.
  - **No Mobile-Game Dark Patterns**: Never introduce artificial wait timers, daily streak coercion, energy meters, fake currencies, or FOMO.
  - **No Irritating Interruptions**: Never interrupt active gameplay with unskippable dialogs, popups, or modal prompts.
