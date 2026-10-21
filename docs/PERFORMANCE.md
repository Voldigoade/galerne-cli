# Performance Budgets & Measurement Protocol

Resource consumption is a primary product feature. All performance assertions must be backed by reproducible empirical measurements.

The budgets below are targets, not current measurements. The figures in `docs/measurements/2026-10-21-baseline.md` describe an earlier build and do not establish performance for the current branch.

## Performance Budgets

Target constraints for release builds on standard developer machines (Linux x86_64 and Windows x86_64):

- **Startup Latency**: < 15 ms from invocation to initial frame rendering.
- **Binary Size (Stripped Release)**: Target < 5 MB (ideally < 2 MB, subject to audited crate dependencies).
- **Resident Memory (RSS)**: < 20 MB during active gameplay.
- **Peak Memory**: < 30 MB high-water mark.
- **CPU Utilization (Idle / Waiting for Input)**: Avoid sustained CPU use while waiting for input; measure on the current binary and terminal before making claims.
- **Rendering Bandwidth**: Use double-buffered diff updates and avoid full-screen redraws to keep terminal output responsive over remote shells and low-bandwidth connections.

## Measurement Records

Record baseline performance measurements in `docs/measurements/`:
- Document date, OS kernel, CPU model, compiler version, and exact measurement command.
- Keep baseline runs committed so regressions can be detected objectively across git history.

## Measurement Commands

### Startup & Memory Profiling (Linux)
```bash
/usr/bin/time -v target/release/<bin_name>
```
Look for:
- `Elapsed (wall clock) time`
- `Maximum resident set size (kbytes)`
- `Minor (reclaiming a frame) page faults`

### CPU Profiling with Flamegraph
```bash
cargo flamegraph --bin <bin_name>
```

### Heap Allocation Tracking with DHAT
Run under Valgrind DHAT to audit total allocated bytes and block count:
```bash
valgrind --tool=dhat target/release/<bin_name>
```
Review:
- `At t-gmax, bytes in use`
- `Total allocations` (aim to minimize hot-loop allocations to 0)

### Inspecting Release Binary Size
```bash
cargo bloat --release --crates
size target/release/<bin_name>
```
