# ADR-0240: Telegram clean-room provider boundary

Status: accepted, implementation in progress

## Decision

Telegram is migrated as an independent integration owner with separate `api`,
`core`, provider adapter, `persistence`, and `runtime` packages. Telegram
packages may depend only on their own contracts, platform contracts, and the
exact neutral `makosh-communications-ingress` contract. They do not import a
communications domain implementation, Gateway implementation, or business
domain.

`makosh-telegram-tdlib` owns the TDLib wire/transport boundary. It exposes
typed provider requests and responses but does not decide lifecycle, evidence
promotion, or business actions. `makosh-telegram-core` owns Telegram lifecycle
policy and maps provider observations into neutral ingress drafts. Telegram
state and operational projections remain Telegram-owned.

Credentials are represented by references and leases; plaintext credentials do
not cross Telegram contracts, logs, or runtime command arguments.

## Implementation state

The package topology and typed account, operation, lifecycle, TDLib transport,
persistence, and observation boundaries are implemented. Telegram now has an
owner-local durable operation schema/adapter with typed command payloads,
idempotency uniqueness, and fenced `SKIP LOCKED` claiming, but the adapter is
wired into a runtime worker, and a long-lived authorization/provider polling
loop owns the TDLib cursor. Provider realtime frames now have an owner-local
durable event journal with cursor/sequence replay, and the integration-owned provider query contract now
covers chat/history loading, cached message projections, message/chat search,
participants, topics, and reactions without introducing business-domain types
or a Gateway dependency. Account, chat, message, file, attachment, participant, and topic
projections now have owner-local durable storage and explicit restore/load
paths; version, tombstone, reaction, and topic-message identity history have
the same durable owner boundary, and message-version/tombstone, attachment,
file, topic-message, avatar, participant lifecycle, reaction summary, operation, command, message-reference, reply-chain,
and forward-chain reads are
exposed through the typed provider query contract. Chat state and message
mutation history are also
durable Telegram-owned projections and restore through the same lifecycle
boundary. Account provisioning, listing, point lookup, retirement, and manual
operation retry are exposed through the typed client port. Provider events reconcile only unambiguous
operation/target pairs; TDLib `old_message_id` is not treated as an operation
identifier. The admitted process bootstrap now authenticates managed identity and
resolves user-session credentials through the opaque Vault route before TDLib
construction. Provider operational Gateway/client transport is still
migration work and must not be represented as complete; the integration-owned
typed client port, framed Unix transport, and durable replay adapter are
implemented for the approved module IPC boundary, but no generic opaque
payload endpoint or direct Gateway-to-Telegram dependency is introduced. The
platform module-client envelope carries exact module/owner/contract/request
identity; payload decoding remains owned by the Telegram integration.

The authorization socket uses a generated `makosh.telegram.v1` protobuf
payload contract. Authorization payloads are not decoded as operational client
requests and do not use a JSON or opaque-byte fallback.

Telegram lifecycle client requests now use the same generated protobuf contract
for account provisioning, lookup, retirement, retry, media-session registration,
runtime start/stop, and replay. All currently defined operational provider
commands and provider query request variants now have explicit generated
protobuf mappings. Lifecycle response account/list/accepted/operation/media-
session variants have the same treatment. The query response variants for
chats, chat avatars, history, and history pages now have explicit mappings,
including their provider-owned nested message projections. Other query
responses for chat state/positions, operational state, topic message IDs,
reactions, reaction summaries, and chat folders now have the same treatment.
Message projections and their cached/reply/forward chain variants, versions,
tombstones, mutations, references, attachments, and files now have explicit
mappings as well. Participants, participant pages, topics, and topic lookups
now have explicit mappings as well. Operations and command records now use the
generated operation and provider-command messages, with fail-closed record
encoding. Realtime replay frames now have a typed frame/event contract covering
every current Telegram provider event variant; event payloads are not opaque or
JSON-backed. The Telegram client envelope has no JSON fallback: unknown request
or response variants fail with an explicit protocol error.
