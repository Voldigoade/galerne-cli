#[cfg(unix)]
mod live_update_tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use galerne::updater::{
        apply_linux_update, compute_sha256, download_bytes, fetch_latest_release,
        parse_sha256_file, select_platform_assets, verify_checksum, SemanticVersion, OFFICIAL_REPO,
    };

    #[test]
    fn test_live_release_metadata_and_asset_download_verification() {
        // 1. Query official release metadata from GitHub
        let metadata = fetch_latest_release(OFFICIAL_REPO).expect("fetch latest release");
        assert!(
            metadata.tag_name.starts_with('v'),
            "Latest published release tag must start with 'v'"
        );
        assert!(!metadata.prerelease, "Release must be stable");

        let latest_ver = SemanticVersion::parse(&metadata.tag_name).expect("parse tag version");
        assert_eq!(latest_ver.major, 0);
        assert_eq!(latest_ver.minor, 2);

        // 2. Resolve Linux x86_64 assets
        let (bin_asset, sha_asset) =
            select_platform_assets(&metadata, "linux", "x86_64").expect("select linux assets");
        assert_eq!(bin_asset.name, "galerne-x86_64-unknown-linux-gnu");
        assert_eq!(sha_asset.name, "galerne-x86_64-unknown-linux-gnu.sha256");

        // 3. Download release binary and checksum from GitHub
        let bin_bytes =
            download_bytes(&bin_asset.browser_download_url).expect("download binary asset");
        let sha_bytes =
            download_bytes(&sha_asset.browser_download_url).expect("download sha256 asset");

        let sha_str = String::from_utf8_lossy(&sha_bytes);
        let expected_digest = parse_sha256_file(&sha_str).expect("parse sha256 file");

        // 4. Verify checksum integrity
        assert!(
            verify_checksum(&bin_bytes, &expected_digest),
            "Downloaded binary SHA-256 must match release .sha256 digest (computed: {}, expected: {})",
            compute_sha256(&bin_bytes),
            expected_digest
        );

        // 5. Test applying atomic update to a mock installed binary
        let temp_dir =
            std::env::temp_dir().join(format!("galerne_live_apply_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("create temp dir");

        let target_exe = temp_dir.join("galerne");
        fs::write(&target_exe, b"#!/bin/sh\necho 'galerne 0.1.0'\nexit 0\n")
            .expect("write mock exe");
        fs::set_permissions(&target_exe, fs::Permissions::from_mode(0o755)).expect("set perms");

        apply_linux_update(&target_exe, &bin_bytes).expect("apply update with real binary");

        // 6. Execute updated binary and assert galerne version output
        let output = std::process::Command::new(&target_exe)
            .arg("--version")
            .output()
            .expect("execute updated binary");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("galerne 0.2."),
            "Updated binary must report galerne 0.2.x, got: '{}'",
            stdout.trim()
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
