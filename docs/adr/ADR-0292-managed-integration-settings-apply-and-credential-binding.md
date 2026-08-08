# ADR-0292: Managed integration settings apply и credential binding

Статус: Принято
Дата: 2026-07-26
Состояние реализации: `managed_integration_settings_apply_v1` и
`zulip_account_lifecycle_v1` реализованы. Public
`owner_vault_provisioning_desktop_v1`, generated frontend client и Mail-owned
UI реализованы по ADR-0295; multi-client `owner_vault_provisioning_v1`
остаётся planned до Android adapter.

Уточняет:

- [ADR-0215: module registration and grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: managed distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0222: Kernel Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Vault and credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0288: managed successor quiesce](ADR-0288-managed-successor-quiesce-and-storage-fence-order.md);
- [ADR-0291: Zulip account lifecycle](ADR-0291-zulip-account-history-query-and-replay-boundary.md).

## Контекст

Settings Registry уже хранит exact schema, desired/effective revisions и
apply state. Generic managed integration launch уже читает только current
snapshot. Однако production control plane не выполняет переход
`pending_validation → pending_apply → applying → current` и не заменяет
managed integration process после owner mutation.

Provider fixtures обходили этот разрыв двумя способами:

- передавали settings snapshot напрямую в launch helper;
- помещали provider credential revision внутрь settings.

Оба способа неприемлемы как production lifecycle:

- direct launch bytes обходят Kernel authority и desired/effective fencing;
- credential revision является Vault binding metadata, а ADR-0222 прямо
  запрещает secret reference/binding в settings;
- успешный spawn ещё не доказывает readiness новой configuration revision;
- старый runtime может продолжить provider intake после credential rotation;
- Kernel не должен знать realm, account, purpose или provider command.

Этот defect общий для managed integrations. Его нельзя исправлять отдельным
Zulip restart alias или импортом Zulip package в Kernel.

## Решение

### Три независимые authority

Один configured provider account образуется только композицией трёх
независимых состояний:

```text
Kernel Settings Registry
  typed non-secret desired/effective configuration

Vault
  secret bytes and monotonic secret revision

Integration owner storage
  opaque configuration instance + purpose + selected secret revision
```

Ни одно состояние не заменяет другое:

- Settings не хранит credential revision, record ID, purpose binding или
  ciphertext;
- Vault не хранит realm, email, phone, provider account ID или UI labels;
- integration storage не хранит credential plaintext и не управляет Kernel
  desired/effective revision.

### Provider-neutral managed settings apply

Kernel вводит exact owner-control operation
`ApplyManagedIntegrationSettingsV1`. Запрос содержит только:

```text
registration_id
storage_capability_id
configuration_instance_id
request_host_bridge
expected_desired_revision
owner_session_id
```

Он не содержит settings values, provider identity, credential metadata,
storage password или executable path. Values уже durably committed в Settings
Registry отдельной `UpdateOperatorSettings` mutation.

Операция разрешена только когда:

- registration approved и owner session current;
- desired revision совпадает с request;
- desired revision больше effective revision;
- apply state равен `pending_validation`;
- exact schema и snapshot повторно проходят structural validation;
- все изменяемые definitions требуют `restart_module`;
- current managed release, grants, Event Hub, Vault и Storage binding доступны.

В первом slice semantic validation выполняется новым managed generation во
время bounded bootstrap/readiness. Это строже, чем принятие process spawn:
runtime обязан декодировать свой snapshot, открыть необходимые platform leases
и послать exact ready acknowledgement. Отдельный restricted
`ValidateSettings` phase остаётся последующим общим gate; до него ошибка
bootstrap переводит revision в `blocked_config`, не в `current`.

### Порядок replacement

Kernel выполняет один fail-closed pipeline:

1. переводит revision в `pending_apply`;
2. переводит revision в `applying`;
3. через ADR-0288 quiesce-ит predecessor до physical Storage fence;
4. резервирует generation `N + 1`;
5. выдаёт successor Storage binding с новыми role/credential fences;
6. повторно проверяет signed executable, descriptor и settings artifact;
7. запускает successor с exact desired snapshot;
8. ждёт bounded exact ready acknowledgement;
9. только после ready подтверждает effective revision и `current`.

Failure после durable desired commit:

- не откатывает desired values или executable;
- не запускает predecessor;
- не уменьшает revision;
- сохраняет sanitized `blocked_config` reason;
- требует explicit correction/retry;
- оставляет выданные successor fences authoritative.

### Integration-owned credential binding

Каждая integration определяет отдельный typed account-lifecycle capability.
Для Zulip:

```text
capability = zulip.account.lifecycle.v1
route      = /makosh.zulip.account.v1.ZulipAccountLifecycleService/Apply
```

Контракт переносит только sanitized revision metadata:

- `BindCredential(account_id, expected_binding_revision,
  credential_revision)`;
- `RetireAccount(account_id, expected_binding_revision)`.

Он не принимает API key, password, OAuth token, Vault record ID, arbitrary
purpose или generic map. `configuration_instance_id` берётся из current
runtime admission, а не из client payload.

Zulip persistence хранит:

```text
account_id
configuration_instance_id
credential_revision
binding_revision
state = pending_restart | active | retired
applied_runtime_generation?
```

CAS не позволяет потерять concurrent rotation. Bind/retire немедленно
quiesce-ит provider intake текущего runtime. Новый credential применяется
только successor generation. После успешного resolve exact Vault revision
runtime атомарно отмечает binding `active`.

### Configuration-only runtime

Managed Zulip runtime может быть ready без provider credential только в
ограниченном configuration state:

- Storage и owner-local account binding доступны;
- account lifecycle, sanitized status и Settings apply control доступны;
- provider HTTP/history/event queue/commands отключены;
- Communications observations не создаются;
- credential resolve не выполняется для отсутствующей/retired binding.

Это не provider readiness. Operational status обязан явно вернуть
`unconfigured`, `pending_restart`, `active` или `retired`.

После bind/rotation runtime не начинает provider I/O с новым revision
самостоятельно. Kernel supervised replacement остаётся единственным apply
path.

### Owner credential provisioning

Credential plaintext создаётся/заменяется только через owner-authorized
write-only Vault provisioning с HPKE boundary ADR-0223. Клиент получает
sanitized revision, затем integration account-lifecycle command связывает её
со своим opaque configuration instance.

Этот ADR не разрешает временный plaintext REST/client method. До реализации
public `owner_vault_provisioning_v1` automated conformance может seed-ить
disposable Vault напрямую, но:

- seed является test support, не production API;
- backend lifecycle gate может доказать binding/resolve/replacement;
- frontend/full operational gate остаётся закрыт без sealed provisioning.

### Kernel/Core agreement

Kernel/Core согласуют только:

- opaque registration/configuration/storage capability identities;
- desired/effective revision and apply state;
- managed runtime/grant/storage/vault generations;
- exact owner-control operation and bounded readiness result.

Kernel/Gateway не:

- импортируют integration packages;
- декодируют provider settings semantics;
- выбирают Vault purpose/secret revision;
- читают integration tables;
- получают credential plaintext;
- создают account/business truth.

Integration не:

- пишет Control Store;
- подтверждает effective revision напрямую;
- резервирует runtime generation;
- выдаёт себе grants;
- вызывает другой domain/runtime/store.

### Units of assembly

Functional responsibilities остаются отдельными:

```text
Kernel settings application unit
  generic state machine and supervised replacement

Kernel owner-control adapter
  owner authorization and protocol mapping

Integration account contract
  typed account lifecycle messages

Integration persistence
  credential binding CAS and state

Integration runtime
  provider quiesce, binding resolve and applied acknowledgement

Release assembly
  immutable artifact composition only
```

Ни Kernel unit, ни integration runtime не становятся assembly.

## Phase gates

### `managed_integration_settings_apply_v1`

Требует:

1. exact owner-control request/response;
2. structural revalidation of desired snapshot;
3. restart-module-only admission;
4. ADR-0288 predecessor quiesce/fence;
5. successor Storage binding/generation;
6. bounded ready wait;
7. effective revision only after ready;
8. blocked state without automatic rollback;
9. stale revision/session/generation negatives;
10. architecture evidence без integration imports в Kernel.

### `zulip_account_lifecycle_v1`

Дополнительно требует:

1. settings schema без credential revision/reference;
2. typed account lifecycle route;
3. owner-local CAS credential binding;
4. configuration-only runtime without provider I/O;
5. bind and retire quiesce current provider intake;
6. fresh Vault resolve only in successor generation;
7. sanitized account status;
8. live rotation from revision `N` to `N + 1`;
9. old runtime/grant/storage/credential fences denied;
10. no secret marker in settings, logs, events or health.

### `owner_vault_provisioning_v1`

Остаётся отдельным multi-client platform/frontend umbrella:

1. authenticated owner device session;
2. HPKE sealed write-only put/replace/retire;
3. no generic read-back;
4. sanitized revision receipt;
5. replay/CAS/wrong-owner negatives;
6. desktop and Android host adapters;
7. browser API never receives root/platform key material.

Desktop subset закрыт отдельным implemented
`owner_vault_provisioning_desktop_v1`; он не утверждает наличие Android
adapter.

## Фактическая реализация

- Core Gateway owner-control protocol публикует отдельную
  `ApplyManagedIntegrationSettingsV1` operation. Kernel adapter повторно
  проверяет current owner session, exact pending revision, schema digest,
  complete snapshot и `restart_module` apply mode.
- Provider-neutral Kernel unit
  `modules/settings/managed_integration.rs` использует ADR-0288 successor
  pipeline: quiesce predecessor, physical Storage fence, next runtime
  generation, fresh Storage/Vault binding, bounded ready и только затем
  effective/current acknowledgement. Ни один integration package туда не
  импортируется.
- Zulip Settings schema major 3 содержит только `account_id`,
  `account_email` и `realm_url`; bot-only schema major 2 заменён ADR-0304.
  Credential revision, record ID, secret reference и plaintext отсутствуют.
- `zulip.account.lifecycle.v1` и Zulip Storage bundle revision 3 реализуют
  typed CAS binding `pending_restart | active | retired`; provider HTTP
  отключается сразу после bind/retire.
- Configuration-only runtime открывает owner-local Storage и client routes, но
  не получает provider credential, не регистрирует event queue и не выполняет
  provider I/O.
- Live contour доказал exact Vault replacement revision 1 → 2, Settings
  revisions 1 → 2 → 3, managed generations N → N+1 → N+2, stale predecessor
  fence, sanitized active/retired status и отсутствие provider connections у
  retired successor.
- Negative live branch связал отсутствующую Vault revision 3, попытался
  применить Settings revision 4 и получил `blocked_config`: effective revision
  осталась 3, predecessor не был автоматически восстановлен.

Live evidence:

```bash
MAKOSH_STORAGE_MANAGED_TEST_FILTER=managed_zulip_account_rotation_and_retirement_use_settings_successors node scripts/test-authenticated-storage.mjs 1.97.0
```

Disposable direct Vault seed/replace в этом contour является test support.
Production plaintext provisioning этим gate не открыто.

## Отклонённые варианты

### Credential revision в Settings

Отклонено: это secret binding metadata и нарушение ADR-0222/0223.

### Provider-specific restart command в Kernel

Отклонено: Kernel начал бы интерпретировать integration semantics.

### Runtime сам подтверждает effective revision

Отклонено: compromised process мог бы объявить stale/partial configuration
current. Только Kernel подтверждает revision после current-generation ready.

### Hot swap API key внутри старого process

Отклонено для первого slice: уже открытые provider requests и in-memory secret
не получают однозначного generation fence.

### Автоматический rollback к старой revision

Отклонено ADR-0222/0219: desired intent остаётся видимым, correction/revert
создаёт новую monotonic revision.

## Проверка

Обязательное evidence:

- unit tests state machine и invalid transitions;
- architecture test на provider-neutral Kernel imports;
- integration persistence CAS/restart/retire tests;
- managed live contour с Settings Registry desired/effective state;
- live successor `N + 1` использует новую credential revision;
- predecessor route, Storage alias и Vault lease больше не current;
- failed credential revision остаётся `blocked_config`;
- no provider connections в configuration-only state;
- public query возвращает только sanitized lifecycle status;
- full architecture, SRP, Cargo boundary, Clippy и workspace test gates.

## Последствия

- Settings, Vault и integration account state больше не смешиваются;
- один managed replacement protocol применим к Mail, Telegram, WhatsApp и
  Zulip без provider imports;
- первый backend gate можно доказать до frontend, но full migration остаётся
  закрыт без generated client и sealed owner provisioning;
- provider runtime получает однозначный restart/generation fence для rotation;
- реализация требует additive owner storage migration и новый generic
  owner-control operation.
