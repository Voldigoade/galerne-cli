use std::fs;
use std::path::PathBuf;

use galerne::shell::installer::{install_marker_block, remove_marker_block, ShellType};

fn temp_config_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("galerne_shell_tests");
    let _ = fs::create_dir_all(&dir);
    let file = dir.join(name);
    let _ = fs::remove_file(&file);
    file
}

#[test]
fn test_shell_marker_block_insertion_and_idempotency() {
    let file = temp_config_file("test_bashrc");

    let initial_content = "# User configuration\nexport FOO=bar\n";
    fs::write(&file, initial_content).expect("wrote initial config");

    // 1. First installation
    let changed = install_marker_block(&file, ShellType::Bash).expect("install should succeed");
    assert!(changed, "First installation must modify file");

    let content_after = fs::read_to_string(&file).expect("read file");
    assert!(content_after.contains("# >>> galerne shell integration >>>"));
    assert!(content_after.contains("# <<< galerne shell integration <<<"));
    assert!(
        content_after.contains("export FOO=bar"),
        "Must preserve user config"
    );

    // 2. Second installation (idempotency check)
    let changed_again = install_marker_block(&file, ShellType::Bash).expect("second install");
    assert!(!changed_again, "Second run should be a no-op if identical");

    let count = content_after
        .matches("# >>> galerne shell integration >>>")
        .count();
    assert_eq!(count, 1, "Must never duplicate marker blocks");
}

#[test]
fn test_shell_marker_block_clean_removal() {
    let file = temp_config_file("test_remove_bashrc");

    let initial_content = "alias ll='ls -la'\n";
    fs::write(&file, initial_content).expect("wrote initial config");

    install_marker_block(&file, ShellType::Bash).expect("install");
    assert!(fs::read_to_string(&file)
        .unwrap()
        .contains("galerne shell integration"));

    // Remove marker block
    let removed = remove_marker_block(&file).expect("remove marker block");
    assert!(removed, "Removal should succeed");

    let final_content = fs::read_to_string(&file).expect("read file");
    assert!(!final_content.contains("galerne shell integration"));
    assert_eq!(final_content.trim(), "alias ll='ls -la'");
}

#[test]
fn test_shell_blocks_contain_all_advertised_tools() {
    let tools = [
        "cargo", "npm", "pnpm", "yarn", "docker", "make", "ninja", "cmake", "go", "pytest",
    ];

    let bash_block = ShellType::Bash.generate_block();
    let zsh_block = ShellType::Zsh.generate_block();
    let ps_block = ShellType::PowerShell.generate_block();

    for tool in tools {
        assert!(
            bash_block.contains(&format!("function {}()", tool)),
            "Bash integration missing wrapper for {}",
            tool
        );
        assert!(
            zsh_block.contains(&format!("function {}()", tool)),
            "Zsh integration missing wrapper for {}",
            tool
        );
        assert!(
            ps_block.contains(&format!("function {} {{", tool)),
            "PowerShell integration missing wrapper for {}",
            tool
        );
    }
}
