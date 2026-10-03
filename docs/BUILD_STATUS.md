# Build status

The inherited Conduit application, contracts, indexer, frontend, tests, and workflows were removed from the current tree. Their original commits remain in Git history. The current tree is a standalone SoroPulse contract workspace.

Verified locally with Rust 1.97.1, Soroban SDK 28.0.0, and Stellar CLI 27.1.0 on 2026-10-02:

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo test --workspace --locked` — 12 tests passed.
- `stellar contract build --package soropulse-ticket-reservation --locked` — passed; optimized WASM 7,510 bytes.
- `bash -n scripts/deploy-testnet.sh scripts/smoke-testnet.sh` — passed.

Testnet deployment was verified at `CAUKWP7TRYYXAF2C5NDAZEAQVWCUMXX76KBPK64MQBZHU7EUJ5JVPHGG`. Deployment transaction `bf56552c2ce9ec0268442ee85b166a967daf5fa517e9719e663893e6965d5b65` succeeded in ledger 4,990,360. The deployed WASM hash is `aabcfdfcf1f112f1a9dac5e61745b91b6b9fd67bbe828ababb137936e5154d1d` (the unoptimized build artifact). Demo event 7 was created, reserved by a separate attendee identity, read back, and its `TicketReserved` event retrieved. The `scripts/smoke-testnet.sh` flow also passed with event 8 and returned both live contract events.

The verified deployment manifest is recorded in `deployments/testnet.json` and references source commit `b478355c44722db0ddef5c31ec14324f013d939c`. A fresh testnet reset requires redeployment and manifest refresh.

GitHub Actions [run 37079247244](https://github.com/soropulse-labs/contracts/actions/runs/37079247244) on commit `9a8cf98` passed formatting, Clippy, all 12 contract tests, the optimized WASM build using Stellar CLI 28.1.0, and artifact upload. The first two CI attempts failed while installing the CLI because the runner lacked the D-Bus and libudev development packages; the workflow now installs both before compiling the CLI.
