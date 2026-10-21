# Repository Engineering Rules

## Product and Platforms

- Galerne is a native Rust terminal game for developer wait time. Keep manual play useful and gameplay local and daemon-free.
- Linux and Windows are first-class targets. Keep terminal, process, path, and installer behavior portable across both.
- Normal gameplay has no network, account, cloud-save, telemetry, analytics, advertising, or background-update requirement. Network use belongs to an explicit user action such as `galerne update`.
- Do not introduce a browser, webview, Node/Python/Java runtime requirement, or audio dependency for V1 without an approved design.
- Preserve established infrastructure unless inspection shows a concrete defect. Read the relevant project skill and tests before changing terminal supervision, saves, activation, or updates.

## Production Safety

- `main` is the stable production branch. Do not change it, merge development work into it, move release tags, or publish a release without explicit user direction.
- Keep development on the requested working branch. Do not create a release version or tag as part of ordinary development.

## Engineering Quality

- Keep dependencies and abstractions lean. Add them only to solve a demonstrated need.
- Use idiomatic Rust and `rustfmt`. Keep tests tied to real behavior and failure modes; do not weaken assertions to make a run green.
- Keep the repository free of dead or commented-out obsolete code, fake tests, placeholders represented as complete, and committed secrets.
- Update affected user-facing docs when behavior changes. Treat old plans and specs as historical unless current code and an explicit decision support them; record architectural decisions in `docs/decisions/` when future maintainers would otherwise repeat the investigation.
- Comments explain invariants, platform behavior, safety constraints, or non-obvious trade-offs. Remove comments that merely narrate the next statement or function.
- Use specific product language and authored names. Avoid generic template-like lore or copy, AI attribution, agent chatter, and unsupported marketing claims.
- Measure performance on the revision being discussed. Include the command and environment for reported numbers; distinguish targets from measured results.
- Before heavy builds or memory-intensive checks, run `dev-health`, use the host's default Cargo job count, and stop work you started if available memory falls below 8 GiB.

## Human Quality Gates

- Automated tests and visual-buffer assertions do not establish that gameplay is fun, art is beautiful, or the character is recognizable. Inspect rendered frames and require human playtesting for gameplay changes.
- Do not describe gameplay or visual work as accepted, or publish a major gameplay release, without explicit human approval.
