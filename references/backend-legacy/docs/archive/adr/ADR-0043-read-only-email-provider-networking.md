# ADR-0043 Read-Only Email Provider Networking

Status: Superseded by ADR-0055

Superseded because: the read-only constraint was a temporary safety measure for the initial implementation phase. Макошь is a personal local-first system and needs full email functionality including sending, replying, flag mutations, and server-side state changes. The read-only restriction now applies ONLY to automated integration tests — production code must support both read and write provider operations.

See ADR-0055 for the current policy.
