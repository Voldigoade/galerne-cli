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

    // Benchmark 2: In-memory Save serialization + CRC32 header encoding
    let state = galerne::storage::schema::SaveState::default();
    let payload = serde_json::to_vec(&state).unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = galerne::storage::atomic::encode_with_header(1, &payload);
    }
    let elapsed = start.elapsed();
    let ns_per_save = elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "In-memory Save encode & CRC32:  {:.2} µs/op ({:.0} ops/sec)",
        ns_per_save / 1000.0,
        1_000_000_000.0 / ns_per_save
    );

    // Benchmark 3: Full atomic disk save (temp file write + sync_all + rename)
    let temp_dir = std::env::temp_dir().join(format!("galerne_bench_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let manager = galerne::storage::manager::SaveManager::with_dir(&temp_dir);

    let disk_iterations = 500;
    let start_disk = Instant::now();
    for _ in 0..disk_iterations {
        manager.save(&state).expect("disk save");
    }
    let elapsed_disk = start_disk.elapsed();
    let us_per_disk_save = (elapsed_disk.as_micros() as f64) / (disk_iterations as f64);
    println!(
        "Full atomic disk save (I/O):    {:.2} µs/save ({:.0} saves/sec, with sync_all + rename)",
        us_per_disk_save,
        1_000_000.0 / us_per_disk_save
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
