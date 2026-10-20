---
name: save-integrity-and-recovery
description: Use when designing or implementing local state persistence, serialization, save migrations, corruption recovery, atomic file writes, and integrity verification.
---

# Save Integrity and Recovery

Engineering guide for bulletproof, compact, versioned local state persistence. Local state must survive sudden process termination, system crashes, power loss, and schema evolution.

## Threat Model & Honest Security

> [!IMPORTANT]
> **Threat Model Reality**: A fully offline, open-source application running on a machine controlled by the player cannot provide perfect anti-cheat or unhackable DRM. Never pretend otherwise.
>
> The design target is:
> 1. Complete protection against data corruption, torn writes, and crashes.
> 2. Clean, deterministic forward migrations.
> 3. Safe fallback and automatic recovery if files are malformed.
> 4. Reasonable tamper-evidence (e.g. checksums / light obfuscation) to prevent casual save corruption, without compromising performance, offline capability, or portability.

## Storage Architecture & Header Layout

Format save files with a fixed, explicit binary/structured header:

```text
+-----------------------+--------------------+-------------------------+--------------------+
| Magic Bytes (4 bytes) | Version (4 bytes)  | Checksum (4 or 8 bytes) | Serialized Payload |
| e.g. b"GMSV"          | u32 little-endian  | CRC32 or xxHash64       | Compact binary/JSON|
+-----------------------+--------------------+-------------------------+--------------------+
```

- **Magic Bytes**: Instantly detects wrong file formats or zeroed-out blocks.
- **Version Number**: `u32` representing the exact schema revision.
- **Checksum**: Computed over the serialized payload to detect torn writes, disk sector corruption, or malformed bytes before attempting deserialization.

## Atomic Write Strategy (Temporary File + Sync + Rename)

To prevent partially written or corrupt files during unexpected exit, power cuts, or SIGKILL:

1. Write new save state to a temporary file in the **same directory and filesystem** (e.g., `save.dat.tmp.12345`). Creating the temp file on the same mount ensures the final rename is atomic.
2. Flush user-space buffers and sync OS disk buffers (`std::fs::File::sync_all()` or `sync_data()`).
3. Atomically rename the temporary file over the primary save file (`std::fs::rename`). On POSIX systems, `rename` is atomic. On Windows, ensure rename semantics replace existing files safely.
4. Clean up any leftover temporary files during process startup.

## Backup Rotation & Corruption Recovery

Maintain a rolling backup strategy:
- `game.save`: Active save state.
- `game.save.bak`: Previous known-good save state.
- `game.save.corrupt.<timestamp>`: Quarantined bad save file.

### Loading & Recovery State Machine

1. Attempt to read and validate `game.save`:
   - Verify magic bytes and checksum.
   - If valid, deserialize and check version.
2. If `game.save` is corrupt or missing:
   - Check if `game.save.bak` exists and is valid.
   - If backup is valid, quarantine the bad `game.save` to `game.save.corrupt.<timestamp>` for post-mortem analysis.
   - Restore state from `game.save.bak` and notify user gracefully.
3. If both primary and backup are missing or corrupt:
   - Archive the unreadable files.
   - Initialize fresh default state without crashing.
   - Never panic or produce an unhandled error on malformed input.

## Schema Versioning & Forward Evolution

- **Deterministic Migrations**:
  - Store migrations as sequential transform functions: `v1_to_v2(data) -> data`, `v2_to_v3(data) -> data`.
  - When loading an older save, sequentially apply migrations up to the current version.
- **Future Version Detection**:
  - If `save_version > CURRENT_VERSION`, reject loading gracefully with a clear message (e.g., "Save created by a newer version of the game"). Never overwrite or truncate a newer save file.

## Serialization & Footprint

- Choose compact, fast serialization:
  - Binary formats (`postcard`, `bincode`, or packed structures) provide minimal footprint (< 10 KB), fast load times, and low parsing overhead.
  - If human readability or debug inspection is needed, use compact JSON or CBOR with optional zstd compression for larger snapshots.
- Bound allocation sizes: Never trust length prefixes in untrusted input without maximum allocation limits.
