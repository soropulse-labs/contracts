# Event interface v1

Filter Stellar RPC `getEvents` by the deployed **contract ID**. The first topic identifies the event; the second is the `u32` schema version. Soroban contract events are emitted by the contract and discarded on failed calls. Use the RPC event ID or its ledger/transaction/event position for ingestion identity. A business `event_id` is an organizer-supplied ticketed event identifier; it is not a blockchain event ID. A transaction may contain several contract events. `reservation_id` is unique only within this contract deployment.

| Emission | Ordered topics (`SCVal`) | Data map (`SCVal`) |
| --- | --- | --- |
| Successful `create_event` | `Symbol("created"), U32(1), U64(event_id)` | `title: String`, `capacity: U32` |
| Successful `reserve` | `Symbol("ticket"), U32(1), U64(event_id), U64(reservation_id)` | `attendee: Address`, `reserved_after: U32` |
| First `close_event` | `Symbol("closed"), U32(1), U64(event_id)` | `reserved: U32` |

The Rust SDK 28 `#[contractevent]` declarations in `contracts/ticket-reservation/src/lib.rs` define these topics and data. `docs/fixtures/` contains serialized ScVal XDR from the local test harness, with provenance. It is not testnet capture.

Adding a new event type can retain version 1. Changing any existing topic order, field type, required data key, or field meaning requires a new schema version and new fixtures. Consumers should reject unknown versions or route them to a separate decoder.

The backend should:

1. Configure the target contract ID and testnet network passphrase from the verified deployment manifest.
2. Filter contract events by that ID and the `ticket` first topic.
3. Decode version 1 data, persist the blockchain event ID alongside `event_id` and `reservation_id`, and deliver downstream according to backend policy.
4. Treat retries, receipts, replay, and application deduplication as backend responsibilities.

The RPC event retention window is limited; a backend should ingest continuously and keep its own durable history. For another compatible contract, configure its own contract ID and adapter without deploying this reference contract.

## Release consumption

For a tagged release, pin the Git tag **and its commit SHA**, then fetch the attached WASM artifact and compare its SHA-256 with the release or verified testnet manifest. For an unreleased integration, pin a commit SHA and build its locked workspace with the documented Stellar CLI version. The backend needs the event schema and contract ID; it does not need a Rust workspace import or a local deployment of this sample contract.
