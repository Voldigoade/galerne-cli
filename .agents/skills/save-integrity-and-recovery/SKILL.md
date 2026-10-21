---
name: save-integrity-and-recovery
description: Use when changing Galerne's save format, migrations, backups, corruption recovery, or local persistence.
---

# Save Integrity and Recovery

Save handling protects player progress across crashes and upgrades. Preserve the existing checksummed format and atomic-write design unless a measured defect requires a change.

## Compatibility and failure handling

- Reject a save from a newer schema without changing its bytes. Preserve that error through every CLI, dashboard, and game caller; never turn it into a default save that may later be written over the original.
- Distinguish an unsupported schema from verified corruption, I/O failure, and migration failure. Recovery actions should follow the actual error, not a string match.
- Validate a backup before restoring or rotating it. Surface backup and migration-backup failures when continuing would risk the only recoverable copy.
- Apply old-schema migrations deterministically and keep a recoverable copy before replacing the source save.

## Durable writes and tests

Write to a temporary sibling on the same filesystem, sync file contents, and use replacement behavior supported by the target OS. Keep temporary and backup handling safe if the process stops between steps.

Use filesystem-backed tests with real bytes. Cover round trips, checksum damage, missing or damaged backups, write failures, migration, and newer-schema rejection through user-facing callers. Assert that protected input bytes remain unchanged after an error.

Checksums detect accidental corruption; they do not prevent deliberate edits. Describe the save format and guarantees at the level actually implemented and tested.
