# Contributing to Stellar Tip Jar Contracts

Thanks for helping improve the Stellar Tip Jar smart contracts.

## How to Contribute

1. Fork the repository.
2. Create a feature branch from `main`.
3. Implement your change with tests and documentation updates.
4. Open a pull request using the checklist in this guide.

## Branching Strategy

- `main`: stable branch, always reviewable and releasable.
- `feature/<short-description>`: new features.
- `fix/<short-description>`: bug fixes.
- `chore/<short-description>`: non-functional maintenance.

Keep branches focused on one concern and avoid mixing unrelated changes.

## How to Run Tests

From the repository root:

```bash
cargo test -p tipjar
```

To build the WASM artifact:

```bash
cargo build -p tipjar --target wasm32v1-none --release
```

## Coding Standards

- Use idiomatic Rust and keep functions small and focused.
- Prefer explicit error handling (`#[contracterror]`, `panic_with_error!`) over opaque panics.
- Add comments for non-obvious on-chain logic and state transitions.
- Keep storage keys stable and documented, because on-chain upgrades depend on data compatibility.
- Add or update tests for every behavior change.

## Pull Request Guidelines

Before opening a PR:

1. Rebase on latest `main`.
2. Run `cargo test -p tipjar` and ensure it passes.
3. Include tests for new logic and edge cases.
4. Update `README.md`/docs when behavior or commands change.
5. Add a clear PR description with:
   - problem statement
   - implementation summary
   - test evidence

PR review checklist:

- [ ] Smart contract behavior is deterministic and storage-safe.
- [ ] Events and errors are well defined.
- [ ] Changes are covered by tests.
- [ ] No secrets or private keys are committed.
