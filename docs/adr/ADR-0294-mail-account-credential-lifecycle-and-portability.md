# ADR-0294: Mail account credential lifecycle and portability

Статус: Принято
Дата: 2026-07-26
Состояние реализации: Phase 1 `mail_account_credential_binding_v1`, Phase 2
`mail_account_retire_delete_v1`, desktop `mail_account_portability_v1` и
umbrella `mail_account_lifecycle_v1` реализованы.

Уточняет:

- [ADR-0204: integration/provider boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0236: integration owners and configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0263: Mail settings and Storage admission](ADR-0263-mail-integration-settings-and-storage-admission.md);
- [ADR-0278: Gmail OAuth](ADR-0278-mail-gmail-oauth-setup-and-refresh-gate.md);
- [ADR-0292: managed settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0293: scoped Vault retirement](ADR-0293-scoped-vault-credential-retirement-and-deletion.md).

Уточняется:

- [ADR-0297: fresh-owner-proof effective Settings export](ADR-0297-fresh-owner-proof-effective-module-settings-export.md).

## Контекст

Mail runtime уже имеет exact sync, delivery и Gmail OAuth capabilities, но
account lifecycle остаётся неполным:

- IMAP/SMTP credential revisions хранятся в Kernel Settings snapshot;
- managed runtime не может стать ready до provisioned IMAP password;
- нет Mail-owned sanitized binding/status query;
- logout/delete не quiesce-ят provider I/O и не retire/delete-ят Vault
  credentials;
- legacy import/export semantics не классифицированы между Settings, Vault,
  Mail и app composition.

Credential revision является Vault binding metadata, а не non-secret
configuration. ADR-0263 в этой части заменяется ADR-0292 и этим решением:
Mail endpoint/account configuration остаётся Settings, credential binding
переходит в Mail-owned persistence.

## Решение

### Owner boundaries

Один Mail configuration instance составляется из независимых authority:

```text
Kernel Settings Registry
  connection ID, IMAP/Gmail/SMTP endpoints, account identifiers,
  sync policy and OAuth public configuration

Vault
  IMAP password, SMTP password, Gmail access token and refresh credential

Mail persistence
  purpose-specific credential revision binding, lifecycle state,
  Gmail OAuth attempt/binding, provider cursors and operational state

first-party app
  import/export/logout/delete user flow composition
```

Communications не хранит Mail settings, credentials, folders, provider state
или lifecycle. Kernel/Gateway не декодируют Mail account commands.
Все non-secret Mail Settings owner-editable с fresh owner proof; обычный
sanitized account query не дублирует endpoint, username/email или CA material.

### Exact credential purposes

Mail использует только закрытый enum:

```text
imap_password
smtp_password
gmail_access_token
gmail_refresh_credential
```

Client не передаёт arbitrary purpose, secret reference, record ID, Vault
location, password/token или provider payload.

### Phase 1: credential binding

`mail_account_credential_binding_v1` добавляет два независимых generated
contracts:

```text
mail.account.credential.bind.v1
  /makosh.mail.account.v1.MailAccountCredentialBindingService/Bind

mail.account.query.v1
  /makosh.mail.account.v1.MailAccountQueryService/Get
```

Bind принимает:

```text
connection_id
purpose = imap_password | smtp_password
expected_binding_revision
credential_revision
```

Gmail revisions не bind-ятся client command: их создаёт уже принятый typed
OAuth workflow ADR-0278.

Mail Storage хранит purpose-specific CAS binding:

```text
connection_id
configuration_instance_id
purpose
credential_revision
binding_revision
state = pending_restart | active | retired | deleted
applied_runtime_generation?
```

Bind немедленно quiesce-ит соответствующий provider path текущего runtime.
Credential применяется только новым managed generation через generic
ADR-0292 Settings successor. Runtime resolve-ит exact bound revision,
atomically отмечает binding active и только затем открывает provider I/O.

Mail runtime может быть ready в configuration-only state:

- Storage, lifecycle/query and Gmail OAuth setup routes доступны;
- IMAP sync отключён без active IMAP binding;
- SMTP delivery отключена без active SMTP binding;
- Gmail sync/delivery отключены без active OAuth binding;
- Communications observations не создаются без provider execution.

### Phase 2: retire and delete

`mail_account_retire_delete_v1` вводит отдельные durable command/status
contracts. Mail сначала durably quiesce-ит account, затем для каждой bound
purpose вызывает ADR-0293 exact `retire` или `delete`.

Multiple purpose mutation хранит per-purpose progress. Потерянный Vault
response reconciles explicit retry through idempotent exact-state action; нет
silent automatic retry. Terminal states:

```text
completed
rejected
outcome_unknown
```

Retire/delete не удаляют Communications evidence и не обращаются к
Communications storage. Delete создаёт Mail account tombstone; physical
provider-side deletion выполняется только если отдельный provider contract
честно поддерживает такую semantics.

### Phase 3: portability

`mail_account_portability_v1` является first-party app composition, а не новым
domain или generic provider facade.

Export объединяет:

- effective non-secret Mail Settings snapshot;
- sanitized Mail account status and connector profile;
- optional provider resource mapping metadata;
- schema/contract versions.

Export никогда не содержит credential bytes, Vault record IDs, wrapping keys,
OAuth codes/verifiers, provider message content или sync cursors.

Import выполняет explicit sequence. Для нового connection требуется первый
configuration-only successor: Mail Bind/Query route проверяет connection ID
текущего runtime и не может честно принять binding ещё не применённой
конфигурации.

```text
validate typed export
  -> create/update Mail Settings desired revision
  -> generic managed Settings apply (configuration-only successor)
  -> query current Mail binding revisions
  -> sealed owner Vault provisioning
  -> Mail credential bind
  -> generic managed Settings apply (credential successor)
  -> query Mail readiness
```

Для Gmail configuration-only successor предшествует OAuth Start. OAuth
Start/Accepted/terminal status являются разными receipts; provider
authorization code не сохраняется в portability state. Повторный Settings
successor для IMAP/SMTP активирует только exact bound credential revision.

App не пишет Mail/Kernel/Vault stores и не создаёт hidden global transaction.
Partial state остаётся видимым и resumable через exact receipts.

Для sealed IMAP/SMTP import Mail descriptor объявляет две отдельные
owner-provisioning capabilities:

```text
mail.imap.credential-provisioning.v1
mail.smtp.credential-provisioning.v1
```

Каждая capability допускает только exact configuration-instance purpose,
`provider_credential` и `create | replace_cas`. Они не дают Mail runtime
generic Vault write, не заменяют отдельные resolve/lifecycle capabilities и не
переносят credential bytes в Mail contract.

### Account query

Реализованный Phase 1 sanitized query возвращает только:

- connection ID and connector profile;
- effective settings revision;
- aggregate account readiness;
- per-purpose binding state and revisions;
- applied runtime generation;
- sync/delivery readiness reason codes.

Phase 2 не встраивает operation receipt в общий Account Query: этот query
совместимо проецирует lifecycle-derived readiness и per-purpose terminal
state, а exact pending/terminal receipt возвращает отдельный
`mail.account.lifecycle.query.v1`. Такое разделение не смешивает status
настроенного account с журналом конкретной lifecycle operation.

Endpoint host, username/email и CA material не возвращаются обычным status
query. Typed export требует отдельной fresh-owner-proof operation.

### Units of assembly

```text
makosh-mail-api
  generated account contracts and wire mapping

makosh-mail-persistence
  credential binding CAS, lifecycle journal and account tombstone

makosh-mail-runtime
  provider quiesce, Vault orchestration and readiness

makosh-mail-imap / gmail / smtp
  provider protocol adapters only

Kernel settings apply
  provider-neutral successor replacement

app portability composition
  first-party client workflow only

makosh-mail-assembly
  immutable release artifacts only
```

Runtime не становится assembly, integration не становится domain, app
composition не получает owner storage.

## Phase gates

### `mail_account_credential_binding_v1`

1. Settings schema без credential revision/reference;
2. exact Bind and Query generated contracts;
3. purpose-specific owner-local CAS binding;
4. configuration-only runtime;
5. bind quiesce and successor-only activation;
6. stale binding/settings/runtime/grant/storage/Vault negatives;
7. sanitized status without secret carriers;
8. live IMAP/SMTP rotation and no-provider-I/O evidence;
9. architecture/SRP/Cargo/Clippy/workspace gates.

### `mail_account_retire_delete_v1`

1. exact durable retire/delete/status contracts;
2. per-purpose durable progress and explicit retry;
3. provider quiesce before first Vault mutation;
4. ADR-0293 tombstone evidence for every bound purpose;
5. Gmail access/refresh credentials handled separately;
6. restart/revoke/stale revision negatives;
7. no Communications deletion or direct store access;
8. sanitized terminal state and privacy negatives.

### `mail_account_portability_v1`

1. typed versioned non-secret export;
2. fresh owner proof;
3. sealed provisioning dependency;
4. resumable multi-receipt import;
5. no secret/session/content/cursor carriers;
6. desktop generated client and integration-owned UI.

Состояние: Implemented для first-party desktop.

### `mail_account_lifecycle_v1`

Umbrella открыт после всех трёх gates выше и существующего
`mail_gmail_oauth_v1`. Это не открывает Mail read/composition/command gates.

## Evidence реализованных Phase 1 и Phase 2

- `makosh-mail-api` поставляет exact generated Bind/Query contracts без
  secret bytes, Vault record IDs и arbitrary purposes;
- Mail Settings schema major 2 содержит только owner-editable non-secret
  configuration и не содержит credential revisions;
- Mail Storage bundle revision 7 хранит purpose-specific CAS binding;
- runtime стартует configuration-only, quiesce-ит изменённый path после Bind
  и активирует exact Vault revision только в successor generation;
- `mail_account_credential_flow` live-conformance проверяет IMAP/SMTP rotation,
  отсутствие provider I/O в `pending_restart`, generic Settings Apply,
  activation revision 2 и stale-generation fencing;
- executable architecture gate:
  `tests/architecture/mail-account-credential-binding.test.mjs`.
- `makosh-mail-api` поставляет четыре независимых exact lifecycle contracts:
  Retire, Delete, explicit Retry и Status; command payload не переносит secret
  bytes, Vault record IDs или arbitrary purposes;
- Mail Storage bundle revision 8 хранит lifecycle operation journal,
  per-purpose progress и отдельный account tombstone;
- Mail runtime quiesce-ит IMAP, SMTP и Gmail provider paths до первой Vault
  mutation, а restart с любым persisted lifecycle state остаётся
  configuration-only;
- sanitized account status включает latest lifecycle revision и operation ID,
  поэтому first-party client после reload продолжает exact status/retry/CAS
  transition без скрытого локального состояния;
- IMAP password, SMTP password, Gmail access token и Gmail refresh credential
  получают отдельные exact lifecycle capabilities с правильным secret class;
- ambiguous Vault response становится `outcome_unknown`, exact command replay
  не повторяет mutation, а продолжение возможно только отдельным Retry после
  successor Vault/Storage/runtime generations;
- live `mail_account_credential_flow` доказывает retire/delete всех четырёх
  purposes, lifecycle status, Mail tombstone, post-delete mutation rejection,
  restart fencing и отсутствие дополнительного IMAP/SMTP I/O;
- executable architecture gate:
  `tests/architecture/mail-account-retire-delete.test.mjs`.
- `makosh-mail-api` поставляет generated `MailAccountExportV1` с exact
  IMAP/Gmail oneof, optional SMTP, schema/effective revision и sanitized
  readiness enums без generic maps;
- Mail export validator повторно использует Mail account и OAuth configuration
  validators, а schema major/revision принадлежат public Mail API contract;
- desktop app получает effective Settings только через fresh-proof
  `OwnerModuleSettingsService`, затем сопоставляет typed values с Mail-owned
  export contract и strict protobuf JSON;
- import хранит Settings update, configuration successor, per-purpose Vault,
  Mail Bind, credential successor и Gmail OAuth receipts отдельно; bind failure
  продолжается с уже полученного Vault receipt без повторного provisioning;
- integration-owned `MailPortabilityPanel` не импортирует Communications,
  Kernel implementation, Vault storage или Mail runtime;
- executable architecture gate:
  `tests/architecture/mail-account-portability.test.mjs`.

## Отклонённые варианты

### Credential revisions в Settings

Отклонено: нарушает ADR-0222/0292 и смешивает configuration с Vault binding.

### Общий IntegrationAccount domain

Отклонено: скрывает разные provider lifecycle semantics и создаёт новый
cross-integration facade.

### Communications Mail account API

Отклонено: provider authorization, folders, sync и delivery принадлежат Mail.

### Runtime hot-swap credential без successor

Отклонено: старые in-flight provider requests не получают однозначного
generation fence.

### Export credentials в файл

Отклонено: portability переносит только non-secret configuration; secret
transfer требует отдельной Vault backup/recovery ceremony.

## Последствия

- Mail Settings становятся non-secret;
- IMAP/SMTP/Gmail lifecycle остаётся у Mail integration;
- logout/delete используют реальный Vault revocation primitive;
- import/export не создают общий Channels facade;
- полный Mail lifecycle требует нескольких независимых commits/gates, а не
  одного всесильного runtime handler.
