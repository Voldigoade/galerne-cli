---
name: rust-native-performance
description: Use when designing, building, or optimizing native Rust applications where startup latency, binary size, resident memory, CPU usage, allocation behavior, or dependency weight are primary engineering constraints.
---

# Rust Native Performance Engineering

Engineering guide for small, fast native Rust applications with strict resource budgets. Always measure before optimizing and substantiate all performance claims with observable evidence.

## Core Principles

1. **Measure Before Optimizing**: Never apply optimizations based on guesswork or intuition. Establish a baseline benchmark or measurement first.
2. **Never Claim "Zero Overhead" Without Measurement**: Quantify actual CPU cycles, memory bytes, and startup milliseconds.
3. **Distinguish Memory Metrics**:
   - **Binary Size**: Stored executable size on disk (`ls -lh`, `size`).
   - **Virtual Memory (VIRT)**: Total address space mapped by OS.
   - **Resident Memory (RSS)**: Physical RAM currently mapped to process (`ps -o rss`, `/proc/[pid]/smaps`).
   - **Private / Working Set**: Memory unique to the process and active pages (crucial on Windows via `GetProcessMemoryInfo`).
   - **Peak Memory**: High-water mark of memory usage during execution (`/usr/bin/time -v` max resident set size).
4. **Avoid Heavyweight Runtimes**: Do not include Tokio, async-std, or complex thread pools unless asynchronous concurrency or network multiplexing genuinely requires it. Synchronous, event-driven loops or lightweight standard threads (`std::thread`) are preferred for native terminal apps.
5. **Evaluate Release Profiles Experimentally**: Test compiler flags with empirical measurements rather than blindly applying presets.
6. **No UPX or Binary Packers**: Never use UPX. Binary packers cause higher startup latency (decompression cost), prevent page sharing in OS cache, increase runtime private memory, and trigger antivirus heuristics.
7. **Cross-Platform Verification**: Measure on both Linux and Windows. Memory allocators and system calls differ fundamentally.

## Dependency Management & Footprint

- **Audit Dependency Weight**: Run `cargo tree -d` to detect duplicate crates and `cargo tree --depth 1` to audit top-level dependencies.
- **Minimize Cargo Features**: Disable default features where possible (`default-features = false, features = [...]`).
- **Inspect Binary Bloat**:
  ```bash
  cargo bloat --release --crates
  cargo bloat --release -n 20
  ```

## Allocation & Data Structure Hygiene

- **Avoid Heap Allocation in Hot Loops**:
  - Reuse allocated buffers (`Vec::clear()` retains capacity; pass `&mut Vec` or `&mut String`).
  - Prefer stack-allocated or small-buffer types (`smallvec`, `tinyvec`, or fixed-size arrays) for small collections with predictable upper bounds.
  - Borrow slices (`&str`, `&[u8]`) instead of cloning owned strings/vectors.
  - Avoid unnecessary pointer-chasing and pointer wrappers (`Box`, `Rc`, `Arc`, `RefCell`) when value types or borrowed references suffice.
- **Cache-Friendly Layouts**: Prefer contiguous flat storage (flat `Vec`, struct of arrays) over node-based trees or linked structures.

## Benchmarking & Profiling Workflow

1. **Micro-Benchmarking**:
   - Always run microbenchmarks on release builds: `cargo bench`.
   - Ensure benchmark targets isolate the hot path and warm up appropriately.
2. **CPU & Allocation Profiling**:
   - Linux CPU: `perf record -F 99 -g -- cargo run --release` and `perf report`, or `cargo flamegraph --bin <bin_name>`.
   - Allocation Profiling: Use DHAT (via `valgrind --tool=dhat` or the `dhat` crate) to track heap blocks and total allocations.
   - Windows: Use Windows Performance Recorder / Windows Performance Analyzer or Tracy.
3. **Startup Latency & Execution Footprint**:
   - Linux: `/usr/bin/time -v target/release/<bin>`
   - Windows: Measure elapsed process time via high-resolution monotonic clocks (`Instant::now()`).

## Experimental Release Profile Configuration

Tune `Cargo.toml` profiles methodically, measuring binary size and execution speed after each change:

```toml
[profile.release]
opt-level = 3        # Test "z" or "s" if binary size outweighs raw CPU performance
lto = "thin"         # Test "fat" for cross-crate optimization if compile time allows
codegen-units = 1    # Enables better optimization and size reduction across units
panic = "abort"      # Eliminates unwinding landing pads and runtime overhead
strip = true         # Strips symbols and debug info from release binary
```

Validate actual binary size changes after every adjustment:
```bash
cargo build --release
ls -lh target/release/<bin_name>
```
