---
name: terminal-runtime-engineering
description: Use when changing Galerne's terminal rendering, input loop, command supervision, process lifecycle, or interrupt behavior.
---

# Terminal Runtime Engineering

Galerne's terminal and process wrapper are user-trust infrastructure. Extend the existing Rust architecture and prove the specific failure paths affected by a change.

## Preserve these behaviors

- Non-eligible and interactive commands bypass supervision with inherited standard streams.
- Eligible commands keep running while the game is visible. Before activation, their output remains live; after game exit, task output continues and the child exit status is propagated.
- During supervised play, the first interrupt exits the game view while the task continues. A later interrupt cancels the child.
- Restore terminal mode, cursor, and screen on normal exits and handled errors. Use the existing guard and verify partial-entry and cleanup failure paths before changing it.
- Bound captured output and process any reader threads without making child completion depend on a descendant closing an inherited pipe.

Keep the diff renderer and event-driven loop lightweight. Avoid busy polling, unnecessary full-screen writes, shell-string execution, and new runtime dependencies without measured need.

Linux and Windows have different process and terminal APIs. Verify each platform claim using its CI or native evidence; a cross-platform compile is not a terminal behavior test. Unit tests of a signal state machine do not replace an end-to-end terminal test when terminal restoration changes.
