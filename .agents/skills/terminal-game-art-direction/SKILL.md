---
name: terminal-game-art-direction
description: Use when creating or reviewing Galerne's terminal sprites, scenery, animation, parallax, or visual composition.
---

# Terminal Game Art Direction

ASCII and ANSI are the drawing medium, not a reason to use primitive or generic art. Compose authored scenes that make a terminal frame feel deliberate, deep, and unexpectedly large.

## Art and composition

- Give the traveler a silhouette, movement, and recurring equipment that remain recognizable at a glance. Make scarf, pack, and traversal gear read as one character rather than a collection of glyphs.
- Design obstacles and landmarks with distinct scale and shape. Use foreground, midground, and distant structure to show height, depth, weather, and inhabited space.
- Build regions from authored compositions and landmarks. Short strings repeated by modulo scrolling or random symbols used as scenery read as wallpaper.
- Keep the world damaged but lived in: immense old infrastructure, people, vegetation, mist, wind, and restrained light. Avoid generic cyberpunk, fantasy, grimdark, and post-apocalypse shorthand.
- Preserve legibility at 80×24 and use larger terminals to add scenic scale and detail rather than stretch a small layout. Respect character-cell width and provide safe glyph choices for Windows terminals.

## Visual review

Render representative frames from the actual game at small and large terminal sizes. Inspect the frame itself for silhouette, hierarchy, repetition, clipping, color contrast, and motion continuity. Repeat for the relevant action states and localized UI.

Buffer snapshots and tests are useful for bounds, clipping, and regression checks. Character-presence assertions do not establish visual quality. Do not claim art acceptance until a person has inspected the rendered frames.
