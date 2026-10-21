# Galerne

Galerne is a native Rust terminal game built around developer wait time. It can supervise eligible build and test commands, let you play while a worthwhile wait continues, then return the command's output and result to your terminal. You can also launch it directly with `galerne` or `galerne play`.

The `gameplay-reboot` branch is active development, not a release. Its side-scrolling traversal is a foundation for a persistent adventure; world design and visual quality are still in progress. The branch is not ready to represent a finished game or a human playtest milestone.

## What works today

- Project-aware duration estimates with a conservative wait before unknown tasks open the game.
- Live command output while Galerne is waiting to activate.
- Bypass for unsupported and interactive commands.
- Leaving the game view does not stop a supervised command. A later Ctrl+C cancels the command.
- Manual play with jump, slide, dash, wind sections, checkpoints, collectibles, and persistent run state.
- Local saves with checksums, backups, migrations, and recovery for corrupt data. A save from a newer unsupported schema is preserved.
- English, French, Spanish, and Simplified Chinese game interface.
- Linux and Windows targets.

These features are not a claim that the current game is complete or visually approved.

## Install

For the published stable build, use the latest artifact on [GitHub Releases](https://github.com/Voldigoade/galerne/releases/latest). The development branch may contain unreleased changes and requires building from source.

### Build from source

Prerequisites: Rust and Cargo.

```sh
git clone https://github.com/Voldigoade/galerne.git
cd galerne
git switch gameplay-reboot
cargo build --release
```

The executable is `target/release/galerne` (`galerne.exe` on Windows). Run `galerne setup` to configure shell integration where supported.

## Use

```sh
galerne                 # Open the dashboard
galerne play            # Begin or resume a manual journey
galerne stats           # Show saved journey statistics
galerne doctor          # Check the terminal, saves, and shell integration
galerne enable          # Enable task activation
galerne disable         # Disable task activation
galerne update          # Explicitly check for and install an update
galerne wrap <command>  # Explicitly supervise a command
```

Automatic activation depends on the configured shell integration and the task classifier. Unsupported or interactive commands run normally.

### Controls

- Jump: `Space`, `Up`, or `W`
- Slide: `Down` or `S`
- Dash: `Right`, `X`, or `D`
- Leave the game view: `Esc` or `q`

When Galerne is supervising a command, leaving the game view lets the command continue and restores its output. Ctrl+C in the game view also leaves the game; Ctrl+C after returning to command output interrupts the command.

## Data and network use

Galerne stores saves and task-duration history locally. It has no resident daemon, telemetry, analytics, account, or cloud-save requirement. Galerne contacts the network only when you explicitly run `galerne update`.

Save data is stored in the platform's standard user data directory. The dashboard's Saves panel shows the exact path. Save files are checksummed and backed up; files with a future schema are not overwritten by this version.

## Performance

This repository does not currently publish performance figures for the development branch. Release size, startup time, memory, CPU use, and frame timing will be measured on a documented build before the human playtest milestone.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

The live release-metadata test needs network access. The rest of the test suite can be run without it.

## License

MIT. See [LICENSE](LICENSE).
