use std::fs;
use std::path::Path;

use galerne::interceptor::project::{
    derive_project_id, find_project_root, get_or_create_salt, resolve_current_project_id,
};

#[test]
fn test_find_project_root_in_git_repository() {
    let base = std::env::temp_dir().join(format!("galerne_proj_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);

    let repo_root = base.join("my_repo");
    let nested_subdir = repo_root.join("nested").join("sub").join("dir");
    fs::create_dir_all(&nested_subdir).expect("create nested dirs");

    // Simulate .git directory at repo_root
    let git_dir = repo_root.join(".git");
    fs::create_dir_all(&git_dir).expect("create .git dir");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("write mock HEAD");

    let discovered = find_project_root(&nested_subdir);
    assert_eq!(
        discovered, repo_root,
        "find_project_root must locate the ancestor containing .git"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_find_project_root_in_git_worktree_or_submodule() {
    let base = std::env::temp_dir().join(format!("galerne_worktree_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);

    let worktree_root = base.join("worktree_repo");
    let nested = worktree_root.join("src");
    fs::create_dir_all(&nested).expect("create nested dirs");

    // In git worktrees/submodules, .git is a file, not a directory
    let git_file = worktree_root.join(".git");
    fs::write(&git_file, "gitdir: /some/path/to/commondir\n").expect("write .git file");

    let discovered = find_project_root(&nested);
    assert_eq!(
        discovered, worktree_root,
        "find_project_root must support .git file (worktree/submodule)"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_find_project_root_fallback_to_cwd() {
    let base = std::env::temp_dir().join(format!("galerne_nogit_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("create base");

    let discovered = find_project_root(&base);
    assert_eq!(
        discovered, base,
        "find_project_root must fall back to the given directory when no .git exists"
    );

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn test_derive_project_id_properties() {
    let salt = "fixed_test_salt_1234567890abcdef";
    let path_a = Path::new("/home/user/workspace/project_alpha");
    let path_b = Path::new("/home/user/workspace/project_beta");

    let id_a1 = derive_project_id(path_a, salt);
    let id_a2 = derive_project_id(path_a, salt);
    let id_b = derive_project_id(path_b, salt);

    // 1. Deterministic
    assert_eq!(id_a1, id_a2, "Project ID derivation must be deterministic");

    // 2. Distinct
    assert_ne!(
        id_a1, id_b,
        "Different projects must produce distinct project IDs"
    );

    // 3. Opaque & privacy-conscious: no path strings in ID
    assert!(
        id_a1.starts_with("p_"),
        "Project ID must have a clean prefix 'p_'"
    );
    assert!(
        !id_a1.contains("workspace") && !id_a1.contains("alpha") && !id_a1.contains("/"),
        "Project ID must not leak raw project path information"
    );

    // 4. Salt effect
    let salt_2 = "different_salt_abcdef1234567890";
    let id_a_salted = derive_project_id(path_a, salt_2);
    assert_ne!(
        id_a1, id_a_salted,
        "Different salts must produce distinct project IDs"
    );
}

#[test]
fn test_get_or_create_salt_persistence() {
    let temp_data_dir =
        std::env::temp_dir().join(format!("galerne_salt_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_data_dir);
    fs::create_dir_all(&temp_data_dir).expect("create temp data dir");

    let salt1 = get_or_create_salt(&temp_data_dir).expect("first get_or_create_salt");
    assert!(!salt1.is_empty(), "Salt must not be empty");

    let salt_path = temp_data_dir.join("salt");
    assert!(salt_path.exists(), "Salt file must exist on disk");

    let salt2 = get_or_create_salt(&temp_data_dir).expect("second get_or_create_salt");
    assert_eq!(
        salt1, salt2,
        "get_or_create_salt must return identical persistent salt"
    );

    let _ = fs::remove_dir_all(&temp_data_dir);
}

#[test]
fn test_resolve_current_project_id() {
    let temp_data_dir =
        std::env::temp_dir().join(format!("galerne_resolve_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_data_dir);
    fs::create_dir_all(&temp_data_dir).expect("create temp data dir");

    let pid = resolve_current_project_id(&temp_data_dir);
    assert!(pid.starts_with("p_"));
    assert_eq!(pid.len(), 18); // "p_" + 16 hex chars

    let _ = fs::remove_dir_all(&temp_data_dir);
}
