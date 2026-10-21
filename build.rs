use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("embedded_galerne.bin");

    if let Ok(embed_src) = env::var("EMBED_GALERNE_BIN") {
        let src = PathBuf::from(embed_src);
        if src.exists() {
            fs::copy(src, &dest_path).expect("Failed to copy EMBED_GALERNE_BIN to OUT_DIR");
            return;
        }
    }

    // Default to empty byte array if no embedded binary specified
    if !dest_path.exists() {
        fs::write(&dest_path, b"").expect("Failed to create fallback embedded_galerne.bin");
    }
}
