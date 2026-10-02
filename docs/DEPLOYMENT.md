# Testnet deployment

Use Stellar CLI 28.1.0, Rust 1.97.1, `jq`, and the pinned workspace. Only testnet is supported by the scripts. Private keys stay in Stellar CLI identity storage; do not commit them.

```bash
stellar network use testnet
stellar keys generate soropulse-organizer --network testnet --fund
stellar keys generate soropulse-attendee --network testnet --fund
cargo build -p soropulse-ticket-reservation --target wasm32v1-none --release --locked
ORGANIZER_IDENTITY=soropulse-organizer ./scripts/deploy-testnet.sh
```

The deploy script prints the contract ID and the next commands. Set `CONTRACT_ID` to that actual ID, then run:

```bash
CONTRACT_ID=C... ORGANIZER_IDENTITY=soropulse-organizer ATTENDEE_IDENTITY=soropulse-attendee ./scripts/smoke-testnet.sh
```

The smoke script creates event 7, reserves as the attendee, reads the reservation, and fetches contract events. Use a new event ID on repeated smoke runs because IDs cannot be reused (for example, set `EVENT_ID=8`). It does not use the backend or frontend.

After a verified deployment, create `deployments/testnet.json` following `docs/deployment-manifest.schema.json`. Record the network passphrase, contract ID, SHA-256 WASM hash, source commit, deployment transaction hash, UTC timestamp, CLI and SDK versions, and schema version. If the CLI does not show the deployment transaction hash, retrieve it from a testnet explorer or the RPC transaction history before publishing the manifest. Do not publish a placeholder as verified. Testnet resets require a fresh deployment and manifest.
