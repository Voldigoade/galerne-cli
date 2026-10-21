//! Command pattern detection and classification for developer tasks.

use serde::{Deserialize, Serialize};

/// High-level categories of developer wait-time operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandClass {
    CargoBuild,
    CargoTest,
    CargoClippy,
    CargoCheck,
    CargoBench,
    NpmInstall,
    NpmBuild,
    NpmTest,
    PnpmInstall,
    YarnInstall,
    DockerBuild,
    Make,
    Ninja,
    CmakeBuild,
    GoBuild,
    GoTest,
    Pytest,
}

impl CommandClass {
    pub fn label(&self) -> &'static str {
        match self {
            CommandClass::CargoBuild => "cargo build",
            CommandClass::CargoTest => "cargo test",
            CommandClass::CargoClippy => "cargo clippy",
            CommandClass::CargoCheck => "cargo check",
            CommandClass::CargoBench => "cargo bench",
            CommandClass::NpmInstall => "npm install",
            CommandClass::NpmBuild => "npm run build",
            CommandClass::NpmTest => "npm test",
            CommandClass::PnpmInstall => "pnpm install",
            CommandClass::YarnInstall => "yarn install",
            CommandClass::DockerBuild => "docker build",
            CommandClass::Make => "make",
            CommandClass::Ninja => "ninja",
            CommandClass::CmakeBuild => "cmake --build",
            CommandClass::GoBuild => "go build",
            CommandClass::GoTest => "go test",
            CommandClass::Pytest => "pytest",
        }
    }
}

/// Inspects command tokens and matches them against eligible build/test operations.
/// Explicitly filters out interactive commands, REPLs, and watch modes.
pub fn match_command(args: &[String]) -> Option<CommandClass> {
    if args.is_empty() {
        return None;
    }

    let bin = PathClean::basename(&args[0]).to_lowercase();

    for arg in args.iter().skip(1) {
        let lower = arg.to_lowercase();
        if lower == "--watch" || lower == "-w" || lower == "watch" || lower == "--interactive" {
            return None;
        }
    }

    if matches!(
        bin.as_str(),
        "python"
            | "python3"
            | "node"
            | "bash"
            | "zsh"
            | "sh"
            | "fish"
            | "pwsh"
            | "powershell"
            | "cmd"
            | "gdb"
            | "lldb"
            | "vim"
            | "nvim"
            | "nano"
            | "less"
            | "more"
            | "top"
            | "htop"
            | "btop"
    ) {
        return None;
    }

    if bin == "cargo" {
        if args.len() < 2 {
            return None;
        }
        let sub = args[1].to_lowercase();
        return match sub.as_str() {
            "build" => Some(CommandClass::CargoBuild),
            "test" => Some(CommandClass::CargoTest),
            "clippy" => Some(CommandClass::CargoClippy),
            "check" => Some(CommandClass::CargoCheck),
            "bench" => Some(CommandClass::CargoBench),
            // Explicitly do not wrap `cargo run` (child application may be interactive)
            _ => None,
        };
    }

    if bin == "npm" {
        if args.len() < 2 {
            return None;
        }
        let sub = args[1].to_lowercase();
        if sub == "install" || sub == "i" || sub == "ci" {
            return Some(CommandClass::NpmInstall);
        }
        if sub == "test" || sub == "t" {
            return Some(CommandClass::NpmTest);
        }
        if sub == "run" && args.len() >= 3 && args[2].to_lowercase() == "build" {
            return Some(CommandClass::NpmBuild);
        }
    }

    if bin == "pnpm" && args.len() >= 2 {
        let sub = args[1].to_lowercase();
        if sub == "install" || sub == "i" {
            return Some(CommandClass::PnpmInstall);
        }
    }

    if bin == "yarn"
        && (args.len() == 1 || (args.len() >= 2 && args[1].to_lowercase() == "install"))
    {
        return Some(CommandClass::YarnInstall);
    }

    if bin == "docker" {
        if args.len() >= 2 && args[1].to_lowercase() == "build" {
            return Some(CommandClass::DockerBuild);
        }
        if args.len() >= 3
            && args[1].to_lowercase() == "compose"
            && args[2].to_lowercase() == "build"
        {
            return Some(CommandClass::DockerBuild);
        }
    }

    if bin == "make" {
        return Some(CommandClass::Make);
    }
    if bin == "ninja" {
        return Some(CommandClass::Ninja);
    }
    if bin == "cmake" && args.len() >= 2 && args[1].to_lowercase() == "--build" {
        return Some(CommandClass::CmakeBuild);
    }

    if bin == "go" && args.len() >= 2 {
        let sub = args[1].to_lowercase();
        if sub == "build" {
            return Some(CommandClass::GoBuild);
        }
        if sub == "test" {
            return Some(CommandClass::GoTest);
        }
    }

    if bin == "pytest" {
        return Some(CommandClass::Pytest);
    }

    None
}

struct PathClean;

impl PathClean {
    fn basename(s: &str) -> String {
        let p = std::path::Path::new(s);
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(s)
            .to_string()
    }
}
