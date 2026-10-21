use std::time::Instant;

use galerne::terminal::buffer::{Buffer, Color};

fn main() {
    println!("=== GALERNE MICROBENCHMARKS ===");

    // Benchmark 1: Buffer diff computation for 80x24 terminal (1,920 cells)
    let prev = Buffer::new(80, 24);
    let mut curr = Buffer::new(80, 24);

    // Mutate 20 random cells
    for i in 0..20 {
        curr.set_char(i * 3, 5, '@', Color::Cyan, Color::Reset);
    }

    let iterations = 100_000;
    let start = Instant::now();
    let mut diff_count = 0;

    for _ in 0..iterations {
        for _ in curr.diff(&prev) {
            diff_count += 1;
        }
    }

    let elapsed = start.elapsed();
    let ns_per_diff = elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "Double-buffer 80x24 diff: {:.2} µs/frame ({:.0} frames/sec throughput, total diffs: {})",
        ns_per_diff / 1000.0,
        1_000_000_000.0 / ns_per_diff,
        diff_count
    );

    // Benchmark 2: Save serialization + CRC32 header encoding
    let state = galerne::storage::schema::SaveState::default();
    let payload = serde_json::to_vec(&state).unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = galerne::storage::atomic::encode_with_header(1, &payload);
    }
    let elapsed = start.elapsed();
    let ns_per_save = elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "Save encode & CRC32:      {:.2} µs/op ({:.0} ops/sec)",
        ns_per_save / 1000.0,
        1_000_000_000.0 / ns_per_save
    );
}
