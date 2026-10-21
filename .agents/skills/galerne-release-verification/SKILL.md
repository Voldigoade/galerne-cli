---
name: galerne-release-verification
description: Use before declaring Galerne work complete, integrating gameplay changes, or preparing a release.
---

# Galerne Release Verification

Use evidence from the revision under review. A green check on `main`, a prior release, or a different local tree does not verify the current branch.

## Automated checks

Run the repository gates from `docs/DEVELOPMENT.md`:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --doc
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

Report each command and its result. If an environment-dependent test fails, name it and show the cause; do not describe a skipped or partial suite as fully passing. The live updater test currently contacts GitHub, so a network-restricted run cannot establish its result.

## Platform and product evidence

- Check the actual workflow result for the revision. The current CI workflow runs on pushes to `main` and pull requests targeting `main`; it does not automatically validate `gameplay-reboot` pushes.
- Keep Linux and Windows evidence distinct. Compile-time tests on one platform do not establish terminal behavior on the other.
- For changes to supervision, interrupts, saves, migrations, or updates, inspect the matching tests for the failure paths and preservation invariants. Exercise terminal restoration in a real terminal when it is part of the change.
- For gameplay or art changes, inspect actual rendered frames and arrange human playtesting. Automated snapshots and collision tests cannot approve art direction or fun.
- Audit affected user-facing text and documentation against the current game. Check all four target locales where UI copy changes.

## Performance and release claims

Measure the revision being discussed with recorded commands and environment. Inspect benchmark scope before treating its output as product-wide performance evidence. Treat existing README numbers and the v0.2.1 measurement record as historical until reproduced on the current revision. Run `dev-health` before a large build or memory-intensive check and follow the repository resource limits.

`main` and published tags are production state. Do not merge, tag, or publish without explicit user direction. Never claim a release, platform, performance target, visual result, or test suite was verified unless the corresponding evidence exists.
