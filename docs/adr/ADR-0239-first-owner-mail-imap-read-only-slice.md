# ADR-0239: First owner Mail/IMAP read-only vertical slice

Статус: Принято
Дата: 2026-07-20
Состояние реализации: Частично реализовано. Этот ADR authorizes только exact
Mail/Communications exception из executable phase policy: он не открывает
`first_owner_v1` для других owner packages и пока ограничен policy + inventory
скелетом (контракты, `Cargo.toml`, точка входа рантаймов), а не полным
runtime-диспетчером.

Зависит от:

- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0220: Канонический durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0223: Vault и credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0225: Фазовые ворота первого production slice](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md);
- [ADR-0233: Whole-instance backup и fenced restore](ADR-0233-whole-instance-backup-and-fenced-restore.md);
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md).

## Решение

Mail/Communications read-only slice исключён из общего `first_owner_v1` gate,
чтобы он мог дать конкретный owner contract для `client_gateway_v1`. Остальные
owner packages по-прежнему требуют полного `first_owner_v1`. Первым exception
owner будет read-only Mail integration
с единственным IMAP adapter и её canonical evidence consumer — Communications.
Это один vertical slice, а не разрешение общего email-клиента, generic provider
framework или будущих POP3/SMTP capabilities.

### Exact atomic inventory

Один atomic policy change добавляет только следующие packages:

```text
makosh-mail-api
makosh-mail-core
makosh-mail-imap
makosh-mail-persistence
makosh-mail-runtime
makosh-communications-ingress
makosh-communications-api
makosh-communications-domain
makosh-communications-persistence
makosh-communications-runtime
```

`makosh-mail-api`, `makosh-communications-api` и
`makosh-communications-ingress` содержат только versioned typed public
contracts. `mail-core` содержит synchronous reconciliation, idempotency and
cursor ports; он не зависит от IMAP, Vault, PostgreSQL, NATS или
Communications implementation. `mail-imap` — rustls protocol adapter и не
видит persistence, Gateway или domain implementation. Каждый persistence
package владеет только своим schema, inbox/outbox и storage port. Только
соответствующий `*-runtime` композирует private packages; Mail runtime вызывает
Communications только через public `makosh-communications-ingress`.

Kernel/Gateway не получают dependency на эти packages. Нельзя создавать
`makosh-mail`, `makosh-communications` или shared provider-SDK god-crate,
cross-owner SQL, direct module socket, legacy re-export или любую production
path/import/generated reference под `references/**`.

### Mail operational contract

Public Mail API содержит ровно `BeginImapConnection`,
`CompleteImapConnection`, `GetConnection`, `SyncNow`, `GetSyncStatus` и
`GetOperationStatus`. Идентификатор операции стабилен, результат terminal
queryable и не раскрывает secret/provider body. Состояния connection: only
`provisioning`, `ready`, `syncing`, `degraded`, `retired`.

Onboarding создаёт pending IMAP configuration. Tauri host adapter временно
HPKE-seals пароль для Vault, zeroizes input и передаёт Mail/Gateway/Kernel
только ciphertext and opaque references. Owner device proof, fresh challenge,
Vault lock and replay failure deny admission without a fallback path.

Поддерживается только IMAPS (`993`) over rustls, one `INBOX`, `EXAMINE` и
`BODY.PEEK`. Plaintext, STARTTLS, POP3, SMTP, IDLE, folder discovery и provider
writes отсутствуют и не advertised in descriptor/capabilities.

One protocol step has a 10-second deadline; entire sync has a 5-minute
deadline. Each configuration has at most one concurrent sync, and this applies
globally. Historical first-slice bounds allowed a configurable window of up to
1,000,000 UIDs, up to 1,000,000 windows and 255 retries per adapter request.
Эти параметры и synchronous client wait заменены
[ADR-0325](ADR-0325-bounded-asynchronous-mail-sync-execution.md): provider
fetch выполняется bounded chunks/pages, имеет не более трёх transient retries,
hard 300-second run deadline и отдельные durable acceptance/terminal contracts.
Mail owns cursor, retry, provider locator/source mapping and its outbox.

Raw message is bounded to 1 MiB; only decoded `text/plain` up to 256 KiB may
be persisted. HTML and raw MIME are neither stored nor emitted. Attachment
bytes are not stored or emitted by this slice; ADR-0246 separately permits
only bounded attachment descriptors through Communications ingress.
Oversized or no-text input yields metadata-only evidence. On UIDVALIDITY reset,
source record may be reused only for an unambiguous `(Message-ID,
INTERNALDATE, content digest)` match; otherwise a new evidence identity is
created.

Retry execution is driven by an adapter-local retry policy object inside
`makosh-mail-imap`, not by ad-hoc hardcoded literals in the loop. This keeps
`max_attempts` and per-attempt delay independently tunable and test-covered.
The historical implementation used `MAX_SYNC_ATTEMPTS = 255` and
`RETRY_DELAY_MILLIS = 120`; ADR-0325 supersedes that implementation contract.

### Communications and Blob boundary

Mail publishes one idempotent neutral observation through ingress. It contains
no provider locator and no private content. Communications owns canonical
evidence, its inbox, body metadata and read models. When a body is admitted,
Mail receives a Communications-owned pending Blob reservation and one-use
write ticket, writes bytes directly to Blob, and only then publishes the
observation. Orphan reservation, hash conflict and duplicate publication are
terminal, typed outcomes; no cross-owner Blob/database access is allowed.

Generated owner clients route all UI requests via Core Gateway. The first UI is
read-only and reuses visual primitives only; it exposes setup, connection and
rendered `text/plain` evidence. Compose, AI, search, import, attachment and
provider-write affordances are absent.

### Admission evidence

The atomic gate change requires executable evidence for all platform
prerequisites plus this slice:

- disposable implicit-TLS Dovecot E2E:
  `UI setup → Tauri HPKE → Vault → IMAP read-only sync → Blob → durable observation → Communications → Gateway generated client → text/plain render`;
- duplicate sync, lost mutation response/restart-replay, UIDVALIDITY reset,
  oversized/no-text, wrong/replayed challenge, locked Vault, orphan Blob
  reservation and hash conflict tests;
- compile-isolation tests for every package edge and negative reference/legacy
  import checks;
- evidence record for every promoted/AI-assisted object with explicit source,
  evidence, confidence, observed/created time, causation, correlation and
  actor/system source. Generic metadata maps are forbidden.

The release binding, descriptor capabilities and StorageBundle are created only
in that same change. A green isolated unit test, an ADR, or code copied from
reference does not admit the slice.

## Reference use

`references/backend-legacy/` and legacy frontend are historical evidence only:
they may inform protocol algorithm, fixture and visual primitive selection.
Every adopted behaviour must receive a new typed boundary, independent
regression test and this ADR as evidence link. No legacy source, migration,
generated output, compatibility facade or runtime dependency enters the
clean-room graph.

## Consequences

Mail is intentionally narrow, useful and reversible: it proves provider to
canonical evidence to client flow without letting raw mail create durable
business truth. POP3, SMTP, OAuth variants, additional folders, HTML and any
domain promotion each require their own capability/ADR and evidence rather
than silently broadening this slice. Attachment descriptors are governed by
ADR-0246 and do not admit attachment bytes.
