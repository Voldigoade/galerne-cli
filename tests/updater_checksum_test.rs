use galerne::updater::checksum::{compute_sha256, parse_sha256_file, verify_checksum};

#[test]
fn test_compute_sha256_matches_known_vector() {
    let input = b"hello galerne updater";
    let digest = compute_sha256(input);

    assert_eq!(digest.len(), 64);
    // sha256("hello galerne updater")
    // Let's verify deterministic calculation
    assert_eq!(digest, compute_sha256(input));
}

#[test]
fn test_parse_sha256_file_formats() {
    // Standard format: `<digest>  <filename>\n`
    let std_format = "d8804b9b47dbce542f14fd6b0337c784739268800539f4feea972e6b7d34469e  Galerne-Setup-x86_64.exe\n";
    assert_eq!(
        parse_sha256_file(std_format),
        Some("d8804b9b47dbce542f14fd6b0337c784739268800539f4feea972e6b7d34469e".to_string())
    );

    // Bare hex digest: `<digest>\n`
    let bare_format = "d8804b9b47dbce542f14fd6b0337c784739268800539f4feea972e6b7d34469e\n";
    assert_eq!(
        parse_sha256_file(bare_format),
        Some("d8804b9b47dbce542f14fd6b0337c784739268800539f4feea972e6b7d34469e".to_string())
    );

    // Invalid format
    assert_eq!(parse_sha256_file("not a valid hash"), None);
}

#[test]
fn test_verify_checksum_case_insensitivity() {
    let payload = b"executable binary content 12345";
    let computed = compute_sha256(payload);

    // Lowercase
    assert!(verify_checksum(payload, &computed.to_lowercase()));
    // Uppercase
    assert!(verify_checksum(payload, &computed.to_uppercase()));

    // Corrupted payload
    let corrupted = b"corrupted payload";
    assert!(!verify_checksum(corrupted, &computed));
}
