# Architecture & Design Decisions

This document outlines the architectural conventions, decision record processes, and local state management guidelines for the project.

## Architecture Decision Records (ADRs)

When evaluating significant technical choices (such as terminal abstractions, serialization formats, or event loops), record the decision in `docs/decisions/`.

Keep ADRs concise and structured:
1. **Context**: What problem are we solving? What are the constraints?
2. **Options Considered**: Short list of evaluated alternatives with pros/cons.
3. **Decision**: Chosen approach and rationale.
4. **Consequences**: Trade-offs accepted and invariants introduced.

ADR files live in `docs/decisions/` named sequentially: `0001-<short-title>.md`.

## Save File Compatibility & State Evolution

Local state persistence must adhere to strict stability and recovery standards:

- **Schema Versioning**:
  Every save payload includes a top-level `version: u32` in its fixed header.
- **Forward & Backward Compatibility**:
  - **Upgrades**: Upgrades are automatic and deterministic via sequential migration functions (`v1_to_v2`, `v2_to_v3`).
  - **Downgrades / Newer Saves**: If a save file indicates a higher version than the current binary supports (`version > CURRENT_VERSION`), the game must gracefully refuse to load it and notify the user. Under no circumstances should an unsupported newer save be overwritten or corrupted.
- **Integrity Verification**:
  Saves must include a checksum (CRC32, xxHash, or BLAKE3) over the serialized payload. Deserialization must only run after verifying magic bytes and matching the checksum.
- **Atomic Persistence**:
  Always write saves to a sibling temporary file in the same filesystem directory, flush and sync, then atomically rename over the target save path.
- **Safe Recovery**:
  Maintain a previous known-good backup (`.save.bak`). If the primary save is unreadable or fails checksum validation, quarantine the corrupt file (`.save.corrupt.<timestamp>`), restore from `.save.bak`, and notify the player without panicking.
