#!/usr/bin/env bash
set -euo pipefail

: "${CONTRACT_ID:?Set CONTRACT_ID to a deployed testnet contract ID}"
: "${ORGANIZER_IDENTITY:?Set ORGANIZER_IDENTITY to the organizer CLI identity}"
: "${ATTENDEE_IDENTITY:?Set ATTENDEE_IDENTITY to the attendee CLI identity}"
EVENT_ID=${EVENT_ID:-7}
organizer_address=$(stellar keys address "$ORGANIZER_IDENTITY")
attendee_address=$(stellar keys address "$ATTENDEE_IDENTITY")
start_ledger=$(stellar ledger latest --network testnet --output json | jq -r '.sequence')

stellar contract invoke --id "$CONTRACT_ID" --source-account "$ORGANIZER_IDENTITY" --network testnet -- create_event --caller "$organizer_address" --event_id "$EVENT_ID" --title 'SoroPulse Demo' --capacity 10
stellar contract invoke --id "$CONTRACT_ID" --source-account "$ATTENDEE_IDENTITY" --network testnet -- reserve --event_id "$EVENT_ID" --attendee "$attendee_address"
stellar contract invoke --id "$CONTRACT_ID" --source-account "$ATTENDEE_IDENTITY" --network testnet -- get_reservation --event_id "$EVENT_ID" --attendee "$attendee_address"
stellar events --id "$CONTRACT_ID" --network testnet --start-ledger "$start_ledger" --count 10 --output json
