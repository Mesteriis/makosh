# ADR-0293: Scoped Vault credential retirement and deletion

Статус: Принято
Дата: 2026-07-26
Состояние реализации: `vault_credential_retirement_v1` реализован. Полный
`mail_account_lifecycle_v1` остаётся отдельным integration gate.

Уточняет:

- [ADR-0215: module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0223: encrypted Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0292: managed integration settings apply and credential binding](ADR-0292-managed-integration-settings-apply-and-credential-binding.md).

## Контекст

Vault protocol уже резервирует actions `retire` и `delete`, а ADR-0223
обязывает explicit credential retirement и tombstone deletion. Production
Vault transport, managed client, service и SQLCipher store пока реализуют
только `resolve`, `create` и `replace_cas`.

Integration-owned lifecycle state сам по себе не является secret revocation
boundary. Verified, но скомпрометированный current runtime с прежним
`Resolve` grant способен запросить известную revision, пока Vault record
остаётся active. Поэтому logout/delete нельзя честно закрыть только флагом в
Mail PostgreSQL или отключением provider loop в памяти.

## Решение

### Exact operations

Vault transport получает две независимые mutation operations:

```text
RetireCredential(expected_revision)
DeleteCredential(expected_revision)
```

Обе операции используют существующий exact binding:

```text
logical_owner_id
configuration_instance_id
purpose_id
secret_class
secret_revision
runtime_instance_id
runtime_generation
grant_epoch
vault_runtime_generation
```

Client не передаёт record path, arbitrary secret reference, provider account
label или wildcard purpose. Kernel согласует только declared
`VaultPurposeRequestV1`, effective GrantSet, current runtime fences и exact
action. Kernel не декодирует provider lifecycle command и не получает
credential plaintext.

### Retire

`retire`:

- требует exact current active record и `Retire` action lease;
- CAS-сверяет ожидаемую revision через lease scope;
- удаляет active ciphertext record из current lookup;
- atomically создаёт durable `retired` tombstone для exact scope/revision;
- закрывает новые resolve/replace/create для этой exact revision;
- не уменьшает revision и не разрешает silent reactivation.

Retirement не удаляет historical backups и не обещает physical erase уже
созданных filesystem snapshots.

### Delete

`delete`:

- требует exact current active либо retired record и отдельный `Delete`
  action lease;
- atomically создаёт/повышает tombstone state до `deleted`;
- гарантирует, что exact scope/revision больше нельзя resolve, replace или
  recreate;
- не возвращает payload или record ID.

Если active record уже удалён успешным `retire`, `delete` разрешён только при
совпадающем retired tombstone. Fresh exact action lease может идемпотентно
подтвердить уже достигнутый такой же tombstone state после потерянного
transport response. Это не новый side effect и не hidden automatic retry:
каждая transport operation остаётся single-use, а переход назад или к
несовместимому state возвращает conflict.

### Store model

SQLCipher schema получает отдельный metadata-only table:

```text
vault_secret_tombstones
  logical_owner_id
  configuration_instance_id
  purpose_id
  secret_class
  secret_revision
  state = retired | deleted
  changed_at_unix_seconds
```

Tombstone не содержит record ID, nonce, ciphertext, payload length, provider
identity или client-visible label. Active secret table и tombstone table
проверяются в одной actor transaction. Create/replace обязаны fail closed при
конфликтующем tombstone.

### Failure semantics

- mutation выполняется single-writer Vault actor;
- lease/action consumed once до повторного side effect;
- transaction failure сохраняет либо прежний active record, либо complete
  tombstone, но не оба состояния как current;
- stale revision, wrong purpose/class/audience/generation/epoch отклоняются до
  store mutation;
- Vault restart инвалидирует transient leases, durable tombstone сохраняется;
- никаких automatic fallback, recreate или lower-revision retry.

### Units of assembly

Responsibilities остаются раздельными:

```text
vault protocol
  typed encrypted transport commands

managed Vault client
  correlated HPKE request/response and exact action lease

Vault runtime service
  authorization consumption and store orchestration

SQLCipher store
  active-record/tombstone transaction

integration lifecycle
  provider/account decision and purpose selection

release assembly
  immutable artifact composition only
```

Integration не импортирует Vault store/runtime. Vault не импортирует Mail,
Telegram, WhatsApp, Zulip или Communications.

## Phase gate `vault_credential_retirement_v1`

Gate требует одновременно:

1. deterministic protocol encode/decode/digest для retire/delete;
2. exact action lease and GrantSet authorization;
3. actor/store transaction with durable tombstone;
4. resolve/replace/recreate denial after retire;
5. delete after matching retire and direct delete of active record;
6. stale revision, wrong purpose/class/audience/generation/epoch negatives;
7. restart persistence evidence;
8. no secret bytes/record IDs in tombstone, response, error, log or health;
9. managed-client correlated HPKE conformance;
10. architecture, SRP, Cargo, Clippy and workspace gates.

## Фактическая реализация

- `VaultTransportCommandV1` имеет разные deterministic `RetireLease` и
  `DeleteLease` commands с различными operation digests.
- Correlated managed Vault client получает exact `Retire`/`Delete` action
  lease и переносит только HPKE-bound command/result.
- Vault service consume-ит action одноразово и строит scope только из lease.
- SQLCipher schema version 3 добавляет metadata-only tombstones и trigger,
  запрещающий recreate exact retired/deleted revision.
- Single-writer actor atomарно удаляет active ciphertext и создаёт retired
  tombstone; delete повышает retired tombstone либо сразу tombstone-ит active
  record.
- Conformance доказывает resolve/recreate denial, idempotent exact-state
  reconcile, direct delete, retire → restart → delete и отсутствие direct
  SQLite access вне actor.

Validation evidence:

```bash
cargo +1.97.0 test --locked -p makosh-vault-protocol -p makosh-vault-testkit
make test-architecture
make architecture-policy-check architecture-evidence-check srp-policy-check cargo-boundaries-check
make clippy
make test-workspace
make test-integration
```

## Отклонённые варианты

### Только owner-local lifecycle flag

Отклонено: integration runtime с прежним grant может обойти собственную
PostgreSQL projection и повторно resolve-ить Vault revision.

### Удалить строку без tombstone

Отклонено: та же revision может быть незаметно создана повторно, а delete
теряет monotonic evidence.

### Передавать Vault record ID через client lifecycle API

Отклонено: это secret-location carrier и confused-deputy boundary.

### Общий Kernel logout handler

Отклонено: Kernel начал бы интерпретировать provider account semantics и
purpose selection.

## Последствия

- integration logout/delete получает реальный secret revocation primitive;
- Mail lifecycle сможет quiesce provider state и отдельно retire exact
  IMAP/SMTP/Gmail purposes;
- Settings остаются non-secret configuration;
- tombstones увеличивают metadata retention, но не раскрывают secret values;
- physical secure erase старых backups по-прежнему не обещается.
