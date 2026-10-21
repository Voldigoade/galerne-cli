use galerne::updater::version::SemanticVersion;

#[test]
fn test_semantic_version_parsing() {
    assert_eq!(
        SemanticVersion::parse("0.1.0"),
        Some(SemanticVersion {
            major: 0,
            minor: 1,
            patch: 0,
            is_prerelease: false
        })
    );
    assert_eq!(
        SemanticVersion::parse("v0.2.1"),
        Some(SemanticVersion {
            major: 0,
            minor: 2,
            patch: 1,
            is_prerelease: false
        })
    );
    assert_eq!(
        SemanticVersion::parse("v1.10.3-rc1"),
        Some(SemanticVersion {
            major: 1,
            minor: 10,
            patch: 3,
            is_prerelease: true
        })
    );
    assert_eq!(
        SemanticVersion::parse("v2.0.0-beta.2"),
        Some(SemanticVersion {
            major: 2,
            minor: 0,
            patch: 0,
            is_prerelease: true
        })
    );

    assert_eq!(SemanticVersion::parse("invalid"), None);
    assert_eq!(SemanticVersion::parse("v1.2"), None);
}

#[test]
fn test_semantic_version_numeric_comparison() {
    let v9 = SemanticVersion::parse("v0.9.0").unwrap();
    let v10 = SemanticVersion::parse("v0.10.0").unwrap();
    assert!(
        v10 > v9,
        "v0.10.0 must be greater than v0.9.0 (numeric comparison, not lexicographic)"
    );

    let v011 = SemanticVersion::parse("v0.1.1").unwrap();
    let v020 = SemanticVersion::parse("v0.2.0").unwrap();
    assert!(v020 > v011);

    let v1 = SemanticVersion::parse("v1.0.0").unwrap();
    let v2 = SemanticVersion::parse("v2.0.0").unwrap();
    assert!(v2 > v1);
}

#[test]
fn test_is_newer_than_and_prerelease_rules() {
    let current = SemanticVersion::parse("v0.1.1").unwrap();

    let newer_stable = SemanticVersion::parse("v0.2.0").unwrap();
    assert!(
        newer_stable.is_newer_than(&current),
        "Newer stable release must be recognized"
    );

    let same_version = SemanticVersion::parse("v0.1.1").unwrap();
    assert!(
        !same_version.is_newer_than(&current),
        "Identical version is not newer"
    );

    let older_version = SemanticVersion::parse("v0.1.0").unwrap();
    assert!(
        !older_version.is_newer_than(&current),
        "Older version is not newer"
    );

    let newer_prerelease = SemanticVersion::parse("v0.2.0-rc1").unwrap();
    assert!(
        !newer_prerelease.is_newer_than(&current),
        "Prerelease on stable channel must not be treated as a newer upgrade"
    );
}
