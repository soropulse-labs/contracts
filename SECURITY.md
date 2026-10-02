# Security

This is a reference contract for free tickets, not an audited production ticketing system. Capacity is bounded to 128 per event. Event and attendee state share a persistent record so they expire and restore together. A missing archived record is an error; restore it before further operations. The contract cannot guarantee indefinite retention without TTL maintenance.

Organizer control is fixed at deployment. The organizer signs creation and closure, and each attendee signs their reservation. There are no token transfers or on-chain webhook calls.

Please do not post an exploitable issue publicly. Send a private security advisory through the GitHub repository's **Security → Report a vulnerability** flow. Include a minimal reproduction and the affected commit or deployed contract ID.
