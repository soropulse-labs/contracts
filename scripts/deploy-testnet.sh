#!/usr/bin/env bash
set -euo pipefail

: "${ORGANIZER_IDENTITY:?Set ORGANIZER_IDENTITY to a Stellar CLI identity with its secret stored locally}"
if ! stellar keys address "$ORGANIZER_IDENTITY" >/dev/null 2>&1; then
  echo "Identity '$ORGANIZER_IDENTITY' is missing; create it with: stellar keys generate $ORGANIZER_IDENTITY --network testnet --fund" >&2
  exit 1
fi

organizer_address=$(stellar keys address "$ORGANIZER_IDENTITY")
wasm=target/wasm32v1-none/release/soropulse_ticket_reservation.wasm
stellar network use testnet
stellar contract build --package soropulse-ticket-reservation --locked
contract_id=$(stellar contract deploy --wasm "$wasm" --source-account "$ORGANIZER_IDENTITY" --network testnet -- --organizer "$organizer_address")
printf 'Contract ID: %s\n' "$contract_id"
printf 'WASM SHA-256: '
sha256sum "$wasm"
printf 'Source commit: '
git rev-parse HEAD
printf 'Run: CONTRACT_ID=%s ORGANIZER_IDENTITY=%s ATTENDEE_IDENTITY=<identity> ./scripts/smoke-testnet.sh\n' "$contract_id" "$ORGANIZER_IDENTITY"
