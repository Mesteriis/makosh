# ADR-0320: Mail multi-account configuration instances and runtime multiplexing

Статус: Принято
Дата: 2026-07-28
Состояние реализации: Реализовано. `kernel_configuration_targets_v1`,
`managed_configuration_catalog_v1`, `mail_multi_account_runtime_v1` и
`mail_multi_account_frontend_v1` используют target-scoped Settings,
deterministic runtime catalog и Mail-owned account catalog. Live browser proof
подтверждает два независимо выбираемых Mail targets с честной readiness;
recovery и add-account composition используют те же public owner contracts.

Уточняет:

- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0236: integration owners and configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md);
- [ADR-0267: managed integration state roots](ADR-0267-kernel-staged-runtime-artifacts-and-integration-state-roots.md);
- [ADR-0292: managed integration settings apply](ADR-0292-managed-integration-settings-apply-and-credential-binding.md);
- [ADR-0294: Mail account lifecycle and portability](ADR-0294-mail-account-credential-lifecycle-and-portability.md);
- [ADR-0319: legacy provider account recovery](ADR-0319-owner-authorized-legacy-provider-account-recovery.md).

## Контекст

Mail contract, persistence и provider operations уже адресуются стабильным
`connection_id`, но текущий managed bootstrap применяет только один Settings
snapshot и запускает Mail runtime с одной `configuration_instance_id`.
Control Store хранит desired/effective revisions только по
`registration_id`, а public Settings update/export не адресуют target.

Это не позволяет одновременно восстановить две активные Mail учётные записи:
второй Settings apply перезапишет первую конфигурацию. Запуск второго
approved Mail registration/process не является корректным обходом:
ADR-0236 выбирает один runtime owner, мультиплексирующий bounded configuration
instances, а Core Gateway имеет один exact Mail route family.

Нельзя переносить account configuration в Communications, generic
`AccountService`, frontend local storage или arbitrary JSON. Нельзя также
показывать две строки UI поверх одной runtime configuration: это fake
multi-account и потеря provider authority.

## Решение

### Authority и units of assembly

```text
Kernel Settings Registry
  opaque configuration targets, desired/effective revisions and apply state

Kernel managed launch
  canonical bounded catalog of effective configuration snapshots

Mail integration
  connection identity, account lifecycle, provider readiness and operations

Vault
  purpose-scoped credential records for one configuration instance

first-party app
  Mail-specific account setup/import composition

Communications
  provider-neutral evidence only
```

Это не создаёт новый domain. Kernel остаётся provider-neutral, Mail не читает
private Control Store, Core Gateway не интерпретирует Mail settings, а
integration runtime не становится release assembly.

### Configuration target lifecycle

Public `OwnerModuleSettingsService` получает отдельную generic operation
`CreateConfigurationTarget`. Она:

- требует fresh active owner-device proof;
- адресует exact approved module registration с admitted Settings schema;
- требует effective grant `settings.configuration-catalog.v1`; наличие
  configuration-instance scoped schema само по себе не открывает multi-account;
- генерирует stable opaque `configuration_instance_id` в Kernel;
- materializes только defaults definitions со scope
  `configuration_instance`;
- создаёт target в `blocked_config`, если required values отсутствуют;
- возвращает opaque target ID и revision, но не provider/account metadata;
- является idempotent по `operation_id`.

Settings `Update`, `Apply` и `ExportEffective` обязательно адресуют
`registration_id + configuration_instance_id`. Snapshot `target_id` равен
configuration instance, а не registration. CAS revisions независимы между
targets.

Удаление target из Kernel не является Mail logout/delete. Сначала выполняется
typed Mail lifecycle ADR-0294; отдельное future решение может ввести
owner-authorized Settings target tombstone после terminal Mail deletion.
`mail_multi_account_v1` не удаляет target автоматически.

### Control Store

Settings schema artifact и binding остаются registration-scoped. Target state
становится отдельной relation:

```text
registration_id
configuration_instance_id
desired_revision
effective_revision
apply_state
sanitized_reason_code
```

Desired/effective snapshot key является
`(registration_id, configuration_instance_id, revision)`.
Все CAS, recovery and restart transitions включают оба identity fields.
Registration-scoped desired/effective/apply columns старого schema binding
временно остаются read-compatible mirror только для legacy target
`configuration_instance_id == registration_id`; authoritative состояние всех
targets находится в новой relation.

Существующий singular row не копируется во все targets. Schema migration
создаёт ровно один target для каждой существующей admitted integration,
используя deterministic compatibility identity
`configuration_instance_id == registration_id`. Старый Control Store не
сохранял launch `configuration_instance_id` как durable Settings identity,
поэтому утверждать его восстановление нельзя. Все новые targets получают
Kernel-generated opaque identity; неоднозначность fail closed до runtime
replacement.

### Managed runtime catalog

Один managed Mail process получает bounded, deterministically sorted catalog:

```text
ManagedConfigurationCatalogV1
  registration_id
  entries[]
    configuration_instance_id
    effective SettingsSnapshotV1
    isolated integration state root
```

Kernel включает только current effective targets этого registration и
successor target, который проходит apply. Duplicate target IDs, duplicate
Mail `connection_id`, mixed schema major, missing state root, stale revisions
и catalog over limit отклоняются до process launch.

Любой target apply заменяет весь Mail process через existing successor
fencing. Runtime generation остаётся общей process identity, но Vault lease,
provider state, cursor, lifecycle и readiness scoped к конкретной
configuration instance. Старый process fenced до публикации нового catalog.

V1 limit — не более 32 active configuration instances на один Mail
registration. Это resource bound, а не product licensing.

Другие integration runtimes продолжают singular launch до отдельного owner
slice. Generic Kernel catalog contract не означает, что Telegram, Zulip или
WhatsApp автоматически становятся multi-account.

### Mail runtime

Mail runtime декодирует каждую snapshot независимо и строит owner-local
account registry keyed by `connection_id`:

```text
connection_id
  -> configuration_instance_id
  -> typed MailAccountConfigurationV1
  -> purpose-specific binding/readiness
  -> provider adapter state
```

Один `connection_id` не может принадлежать двум configuration instances.
Ошибка конфигурации, auth, cursor или provider throttling одной учётки не
делает соседнюю учётку unavailable. Process-level health и account readiness
остаются разными осями.

Каждая sync, delivery, OAuth, binding, lifecycle и operational command
сначала resolve-ит exact account по `connection_id`; provider I/O и Vault
request используют только соответствующий `configuration_instance_id`.
Automatic fallback на другую учётку запрещён.

Account-local mutable runtime state выделяется в отдельный SRP unit. Общие
transport/event/control resources остаются process-level и не содержат
provider/account branching.

### Mail account catalog

Mail добавляет exact generated
`mail.account.catalog.query.v1`:

```text
/makosh.mail.account.v1.MailAccountCatalogService/List
```

List возвращает bounded deterministic список существующих Mail account
statuses. Он содержит только уже допустимые sanitized поля ADR-0294:
`connection_id`, connector profile, settings/runtime/lifecycle revisions,
readiness и purpose binding states.

Kernel Module Registry не становится account catalog. Frontend получает
module/runtime health отдельно, а Mail accounts — только через Mail generated
client. Endpoint, username/email, CA material, secret references, Vault IDs и
provider content в catalog отсутствуют.

### Frontend setup и selection

Mail Settings показывает account list, отдельный `Add mail account` flow и
selected account editor:

```text
CreateConfigurationTarget
  -> target-scoped Settings update
  -> target apply
  -> Mail catalog/status
  -> provider-specific credential or Gmail OAuth flow
```

`Add mail account` является Mail-owned wizard с явным выбором профиля:

- Gmail: address/profile settings, target apply, затем real OAuth через typed
  Mail lifecycle;
- iCloud: address, app-specific password provisioning и pinned IMAP/SMTP
  defaults;
- custom IMAP/SMTP: explicit endpoints, TLS policy, username и отдельные
  inbound/outbound credential purposes.

Wizard не пишет canonical account list, secret или provider session во
frontend state. Каждый шаг либо хранит локальный draft до submit, либо получает
typed receipt от Settings/Mail/Vault authority. Закрытие и повторное открытие
wizard восстанавливает состояние из provider-owned query, а не из общего
`AccountService`.

Compose, folders, sync and account operations всегда несут selected
`connection_id`. UI не создаёт generic provider account model и не хранит
canonical account list локально.

ADR-0294 portability принимает explicit target configuration instance.
Legacy recovery ADR-0319 создаёт два разных targets, поэтому iCloud и Gmail
имеют независимые Settings/Vault/readiness receipts.

### Failure, privacy и recovery

- apply одного target не меняет desired/effective revisions соседнего;
- catalog replacement не считает healthy process доказательством ready
  accounts;
- partial successor возвращает sanitized per-target blocker;
- source/provider identifiers и secret material не попадают в Kernel tables,
  launch logs, health или account catalog;
- restart восстанавливает account registry из effective target catalog и
  owner-local persistence;
- current singular configuration остаётся доступна через deterministic
  one-target migration;
- legacy Gmail всё равно требует real OAuth по ADR-0319;
- Telegram multi-account этим решением не открывается и проходит real TDLib QR
  по ADR-0310.

## Phase gates

### `kernel_configuration_targets_v1`

1. Generated public create/update/apply/export contracts с fresh owner proof.
2. Composite target persistence, per-target CAS и singular-row migration.
3. Schema binding registration-scoped, snapshots target-scoped.
4. Idempotent create receipt, stale/foreign/deleted/over-limit negatives.
5. No provider fields, arbitrary metadata or secret carriers.
6. Recovery-only and non-integration registrations remain unchanged.

### `managed_configuration_catalog_v1`

1. Bounded deterministic launch catalog and isolated state roots.
2. Whole-process successor fencing for one target apply.
3. Duplicate connection/target, mixed schema and stale revision negatives.
4. Singular runtime compatibility for integrations outside the gate.
5. Runtime/grant/storage/Vault generation fencing remains exact.

### `mail_multi_account_runtime_v1`

1. Mail-owned account registry and generated bounded List contract.
2. Per-account settings, credential binding, OAuth, lifecycle and readiness.
3. Sync/delivery/operational routing by exact `connection_id`.
4. One-account failure isolation and no fallback to a sibling account.
5. Restart persistence and outbox/event identity conformance.
6. Architecture, Cargo boundaries, SRP, format, Clippy and tests.

### `mail_multi_account_frontend_v1`

1. Generated Settings target client and Mail catalog client.
2. Account list, selected editor and add-account composition.
3. Existing Mail operational surfaces carry selected `connection_id`.
4. Two-account live proof with independent readiness.
5. No generic account facade or frontend canonical storage.

`mail_multi_account_v1` открывается только после всех четырёх gates.

## Отклонённые варианты

### Второй approved Mail registration или process-per-account

Отклонено: дублирует owner admission/grants/routes, противоречит default
topology ADR-0236 и не имеет отдельной security/failure причины.

### Array или JSON map accounts в одном Settings value

Отклонено: уничтожает per-target CAS, Vault scope, lifecycle/readiness и SRP;
generic Settings вынужден интерпретировать provider structure.

### Mail-owned копия non-secret configuration

Отклонено: создаёт вторую authority рядом с Kernel desired/effective Settings
и делает restart/recovery reconciliation неоднозначным.

### Account list из Kernel Module Registry

Отклонено: Kernel получил бы provider operational catalog и начал бы
агрегировать integrations.

### Импортировать одну почту сейчас, вторую позже поверх неё

Отклонено: это overwrite, а не перенос двух accounts, и создаёт ложное
completion evidence.

## Последствия

- Две и более Mail учётные записи обслуживаются одним admitted integration
  runtime без смешения их Settings, Vault bindings и provider state.
- Kernel получает недостающую generic configuration-target semantics, но не
  знает, что target является mailbox.
- Mail frontend может честно показывать account list и readiness.
- Один target apply перезапускает общий Mail process; краткая process-level
  недоступность всех Mail accounts является осознанным V1 trade-off.
- Реализация затрагивает platform, Mail integration и frontend как разные
  build units и должна поставляться отдельными крупными commit slices.
