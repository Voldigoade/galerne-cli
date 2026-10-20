//! Atomic file write operations and binary integrity header parsing.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

pub const MAGIC_BYTES: &[u8; 4] = b"GLRN";
pub const HEADER_SIZE: usize = 16;

/// Encodes payload bytes with magic bytes, schema version, CRC32 checksum, and length prefix.
pub fn encode_with_header(version: u32, payload: &[u8]) -> Vec<u8> {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(payload);
    let checksum = hasher.finalize();

    let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
    out.extend_from_slice(MAGIC_BYTES);
    out.extend_from_slice(&version.to_le_bytes());
    out.extend_from_slice(&checksum.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Decodes and validates header from raw bytes.
/// Returns `(version, payload_bytes)` or an `io::Error` on mismatch.
pub fn decode_and_verify(bytes: &[u8]) -> io::Result<(u32, &[u8])> {
    if bytes.len() < HEADER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Save file truncated or smaller than header",
        ));
    }

    if &bytes[0..4] != MAGIC_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid save file magic bytes",
        ));
    }

    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let expected_crc = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let payload_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;

    let payload = &bytes[HEADER_SIZE..];
    if payload.len() != payload_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Payload length mismatch: expected {}, got {}",
                payload_len,
                payload.len()
            ),
        ));
    }

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(payload);
    let actual_crc = hasher.finalize();

    if actual_crc != expected_crc {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Save corruption: checksum mismatch (expected {:#x}, got {:#x})",
                expected_crc, actual_crc
            ),
        ));
    }

    Ok((version, payload))
}

/// Atomically writes data to `path` using a sibling temporary file and `sync_all()`.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let pid = process::id();
    let temp_name = format!(
        "{}.tmp.{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        pid
    );
    let temp_path = path.with_file_name(temp_name);

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        file.write_all(data)?;
        file.sync_all()?;
    }

    // Replace target path atomically
    if let Err(e) = fs::rename(&temp_path, path) {
        // Clean up temp file on failure
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
}

/// Reads full bytes from a file.
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}
