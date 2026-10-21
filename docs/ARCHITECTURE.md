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
  The fixed header carries the schema version, and the serialized payload includes the matching top-level `version: u32`.
- **Forward & Backward Compatibility**:
  - **Upgrades**: Upgrades are automatic and deterministic via sequential migration functions (`v1_to_v2`, `v2_to_v3`).
  - **Newer Saves**: If a save file indicates a higher version than the current binary supports (`version > CURRENT_SCHEMA_VERSION`), loading returns an explicit unsupported-schema error. Save, reset, and restore operations must preserve that file unchanged.
- **Integrity Verification**:
  Saves use CRC32 over the serialized payload. Deserialization must only run after verifying the magic bytes and checksum.
- **Atomic Persistence**:
  Always write saves to a sibling temporary file in the same filesystem directory, flush and sync, then atomically rename over the target save path.
- **Safe Recovery**:
  The active save is `save.dat`, with a previous-save backup at `save.dat.bak`. A corrupt primary is moved to `save.dat.corrupt.<timestamp>` before recovery from the backup. Missing, corrupt, I/O-failed, and unsupported-schema saves remain distinct outcomes; only a missing save may initialize defaults.
