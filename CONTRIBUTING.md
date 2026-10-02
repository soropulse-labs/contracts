# Contributing to SoroPulse contracts

Use Rust 1.97.1 and the pinned `Cargo.lock`. This repository is independent of the SoroPulse backend and dashboard. Keep the reference contract focused on free reservations and contract events.

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
stellar contract build --package soropulse-ticket-reservation --locked
```

Add tests for behavior and authorization changes. Update `docs/EVENT_INTERFACE.md`, the versioned fixtures, and the schema version if a topic, field type, or field meaning changes. Explain storage or TTL changes in `docs/STORAGE.md`. Do not commit identities, secrets, deployment keys, or production credentials. Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).
