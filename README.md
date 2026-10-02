# SoroPulse contracts

A standalone Soroban reference contract for SoroPulse, the open-source Stellar event delivery and recovery toolkit. This repository contains one free ticket-reservation contract. The `soropulse-backend` and `soropulse-frontend` repositories are separate projects; neither is needed to build, test, or deploy this contract. Applications can use SoroPulse with their own compatible contracts.

A reservation emits a versioned `ticket` event for an off-chain consumer. The contract does not deliver webhooks or track off-chain receipts.

## Quick start

Install Rust 1.97.1, the `wasm32v1-none` target, and Stellar CLI 28.1.0. `rust-toolchain.toml` and `Cargo.lock` pin the Rust toolchain and dependency graph. From a fresh clone:

```bash
git clone https://github.com/soropulse-labs/contracts.git
cd contracts
rustup toolchain install 1.97.1 --component rustfmt clippy --target wasm32v1-none
cargo install --locked stellar-cli@28.1.0
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
stellar contract build --package soropulse-ticket-reservation --locked
```

The deployable artifact is `target/wasm32v1-none/release/soropulse_ticket_reservation.wasm`. See [testnet deployment](docs/DEPLOYMENT.md) for a funded testnet identity, deployment, a demo reservation, and event retrieval.

## Contract interface

| Method | Result | Authorization |
| --- | --- | --- |
| `__constructor(organizer: Address)` | — | Organizer signs the deployment |
| `create_event(caller: Address, event_id: u64, title: String, capacity: u32)` | `Result<(), Error>` | Caller must be the organizer |
| `reserve(event_id: u64, attendee: Address)` | `Result<Reservation, Error>` | Attendee signs |
| `close_event(caller: Address, event_id: u64)` | `Result<(), Error>` | Caller must be the organizer |
| `get_event(event_id: u64)` | `Result<TicketEvent, Error>` | Public |
| `get_reservation(event_id: u64, attendee: Address)` | `Result<Option<Reservation>, Error>` | Public |
| `has_reserved(event_id: u64, attendee: Address)` | `Result<bool, Error>` | Public |
| `get_organizer()` / `schema_version()` | `Address` / `u32` | Public |
| `refresh_event(event_id: u64)` | `Result<(), Error>` | Public |

Titles are 1–128 UTF-8 bytes. Capacity is 1–128. Business event IDs must increase strictly, so an ID cannot be reused if its record is archived. Closing is idempotent and emits only one close event. Reservation IDs are unique per contract deployment.

[Event schema](docs/EVENT_INTERFACE.md) · [storage lifecycle](docs/STORAGE.md) · [build status](docs/BUILD_STATUS.md) · [contributing](CONTRIBUTING.md) · [security](SECURITY.md)
