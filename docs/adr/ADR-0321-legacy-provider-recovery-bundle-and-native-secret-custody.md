# ADR-0321: Legacy provider recovery bundle and native secret custody

Статус: Принято
Дата: 2026-07-28
Состояние реализации: implemented. `legacy_provider_recovery_bundle_v1`,
bounded native source sessions, direct HPKE sealing в current Owner Vault host,
provider-specific Mail/Telegram apply flows и owner-private resumable receipt
ledger реализованы. Live evidence подтверждает два Mail targets, fresh Gmail
OAuth boundary, real Telegram QR и restart reconciliation трёх terminal
candidates. Существующий provider state не мутируется; потерянный binding или
account восстанавливается stable operation и persisted public credential
revisions без повторной Vault mutation. Atomic `0600` receipt, changed-source,
corrupt/unknown schema и ambiguous-outcome/explicit-retry negatives проверены.

Уточняет:

- [ADR-0309: loopback browser Owner Vault provisioning host](ADR-0309-loopback-browser-owner-vault-provisioning-host.md);
- [ADR-0310: Telegram user-only TDLib QR account identity](ADR-0310-telegram-user-only-tdlib-qr-account-identity.md);
- [ADR-0319: owner-authorized legacy provider account recovery](ADR-0319-owner-authorized-legacy-provider-account-recovery.md);
- [ADR-0320: Mail multi-account configuration instances](ADR-0320-mail-multi-account-configuration-instances-and-runtime-multiplexing.md).

## Контекст

Read-only inspection owner-selected legacy source подтвердил exact V1 inventory:

```text
gmail active = 1
icloud active = 1
telegram_user active = 1
gmail deleted = 2
```

Legacy credential carriers при этом неоднородны:

- iCloud `imap_password` имеет account-scoped Host Vault reference;
- проверенный legacy Vault был создан в dev mode: его отдельный owner-private
  `master.key` доступен, а production Keychain entry для этого Vault
  отсутствует;
- Gmail имеет legacy OAuth reference, но ADR-0319 запрещает считать старый
  token совместимым current credential binding;
- Telegram user имеет legacy session-store key, который ADR-0319 запрещает
  переносить вместе с TDLib state;
- Telegram API ID и API hash исторически были process configuration, а не
  account-scoped Host Vault references;
- Gmail public OAuth client metadata ссылается на отдельный installed-app
  configuration file.

Копирование `.env`, OAuth client JSON, Vault database или TDLib files в current
runtime вернуло бы compatibility configuration и обошло current owner
contracts. Передача расшифрованного iCloud password или Telegram API hash через
browser JavaScript также нарушила бы native secret custody ADR-0309.

Нужен exact одноразовый source carrier, который:

- фиксирует неизменный recovery input;
- отделяет structural account discovery от secret decryption;
- допускает только проверенные legacy keys;
- не превращает generic environment import в production feature;
- передаёт secret bytes из native source reader прямо в existing HPKE
  provisioning ceremony.

## Решение

### Recovery bundle является immutable source snapshot

Owner-authorized preparation создаёт private recovery bundle вне репозитория и
вне legacy source:

```text
manifest.v1.json
catalog.v1.json
vault.db
legacy-vault-master-key.v1
legacy-provider-config.v1
google-oauth-client.v1.json
```

Bundle не является backup format, target store или новым application config.
Он существует только как bounded source adapter для
`legacy_provider_account_recovery_v1`.

`manifest.v1.json` содержит:

```text
schema_revision = 1
created_at
source_generation
exact relative file names
size and SHA-256 for every file
catalog row count
provider counts
```

Manifest не содержит email, username, account ID, secret reference, filesystem
path или secret bytes. Source generation является random public identifier,
не производным от private account identity.

Bundle root задаётся host workflow explicit absolute CLI argument. Native
reader принимает только exact file inventory, regular files без symlinks,
owner-only permissions и immutable digests. Additional files, missing files,
changed bytes, writable group/world permissions и changed manifest fail
closed. Reader не принимает arbitrary file path на каждом operation.

Preparation:

- выполняет PostgreSQL export только в read-only transaction из остановленного
  legacy runtime или owner-approved isolated source clone;
- копирует consistent SQLite Host Vault snapshot без изменения source;
- принимает exact explicit legacy dev master-key file только для проверенного
  dev-mode source, валидирует 32-byte key и доказывает им decrypt exact iCloud
  Vault binding до commit bundle;
- извлекает только exact legacy configuration keys;
- записывает private files atomically с mode `0600`, directory с mode `0700`;
- не печатает private field values или source paths;
- не пишет в legacy PostgreSQL, Vault или TDLib state.

После подготовки apply использует только bundle. Source PostgreSQL и legacy
configuration больше не находятся на target mutation path.

ADR-0319 по умолчанию требует Keychain-backed legacy decrypt. Для этого exact
owner-approved dev-mode source данное решение заменяет только carrier
master-key: native custody и decrypt invariants остаются теми же, но key bytes
берутся из digest-bound private bundle file. Автоматический поиск key files,
fallback между несколькими keys и запись legacy key в current Keychain
запрещены.

### Exact catalog schema

`catalog.v1.json` содержит только три active candidates и две rejected
tombstones в separate count:

```text
candidate kind
opaque source account digest
display label
external account identity
provider-specific non-secret configuration
exact secret purpose plus source Vault reference
deleted candidate count
```

Raw identifiers нужны только native/app composition для current typed owner
contracts и никогда не попадают в stdout, logs, receipts или rendered UI.
Reader проверяет:

- ровно один active `gmail`;
- ровно один active `icloud`;
- ровно один active `telegram_user`;
- ровно две deleted Gmail records;
- отсутствие active `telegram_bot`;
- уникальные source account digests и secret purposes;
- exact provider-specific configuration schema;
- bounded UTF-8 strings и numeric ranges.

Generic JSON pass-through, arbitrary provider config, unknown keys и raw SQL
inside the recovery host запрещены.

### Exact legacy configuration carrier

`legacy-provider-config.v1` не является копией `.env`. Preparation читает
owner-selected legacy configuration и записывает только:

```text
telegram_api_id
telegram_api_hash
```

Parser принимает exact key/value grammar без shell evaluation, substitutions,
includes или executable fragments. Unknown keys не переносятся в bundle.
Telegram API ID является bounded positive integer. Telegram API hash считается
secret and stays native.

### Ограниченный development Telegram credential source

Root `make dev` может передать loopback Owner Vault development host explicit
owner-only `.env` path. Этот путь не является recovery bundle или runtime
configuration: native host принимает только одну полную exact пару
`HERMES_TELEGRAM_API_ID/HASH` либо `MAKOSH_TELEGRAM_API_ID/HASH` и допускает
соответствующий Google OAuth path key только как совместимый соседний literal.
Unknown, mixed, duplicated, non-literal, symlink и group/world-readable input
fail closed.

Browser получает только bounded positive Telegram API ID. API hash остаётся в
native host и передаётся в existing Owner Vault HPKE ceremony через отдельный
purpose-bound custodied sealer; plaintext hash не возвращается в JavaScript,
не попадает в Vite environment и не передаётся managed integration runtime.
Account/Settings mutation начинается только после owner action `Connect with
QR`; сам `make dev` account не создаёт. Production/Tauri contract этим adapter
не расширяется.

`google-oauth-client.v1.json` содержит только installed-app:

```text
client_id
redirect_uris
```

Legacy `client_secret`, token URI payload, certificate metadata и OAuth token
не копируются. Recovery выбирает только exact loopback redirect URI,
поддерживаемый current Gmail OAuth contract. Если такого URI нет, Gmail target
остаётся `blocked_config`.

### Native secret handles

Dry-run создаёт bounded in-memory recovery session, привязанную к:

```text
bundle fingerprint
source generation
candidate opaque digest
secret purpose
expiry
```

Browser получает только:

- session ID;
- bundle fingerprint;
- exact provider counts;
- candidate opaque handles;
- sanitized candidate state.

Email, username, API hash, Vault reference и source path не рендерятся и не
попадают в receipt. Provider-specific non-secret mutation payload может
существовать только как non-logged first-party app IPC value между source
adapter и exact current Settings/lifecycle client; UI не отображает и не
сохраняет его.

Для iCloud и Telegram source secret никогда не возвращается в JavaScript.
Existing Owner Vault provisioning client сначала получает current owner
challenge and authorization. Затем native host принимает opaque secret handle,
повторно проверяет bundle fingerprint, расшифровывает exact legacy record в
`Zeroizing` buffer и передаёт bytes напрямую в existing
`OwnerVaultProvisioningHostV1::seal`.

`legacy-vault-master-key.v1` читается только native reader, zeroize-ится после
use и никогда не используется как current Vault key, wrapping key или
provider credential.

Разрешены только:

```text
icloud / imap_password -> provider_credential
telegram_user / telegram_api_hash -> provider_credential
telegram_user / generated_session_store_key -> session_store_key
```

Generated Telegram session-store key создаётся current native host CSPRNG.
Legacy `telegram_session_key` только структурно подтверждается и игнорируется;
его bytes не расшифровываются и не переносятся.

Gmail legacy OAuth secret не получает native handle. Recovery создаёт только
configuration target и возвращает `reauthorization_required`.

### Provider-specific app compositions

Host recovery UI является maintenance composition и показывает только
fingerprint, counts и sanitized terminal states. Она не создаёт generic
provider account model.

Mail composition:

```text
Gmail opaque candidate
  -> exact target-scoped Settings create/apply
  -> current Mail account catalog
  -> reauthorization_required

iCloud opaque candidate
  -> exact target-scoped Settings create/apply
  -> native iCloud secret seal and current Vault commit
  -> current Mail credential bind
  -> successor activation
  -> provider-path readiness query
```

Telegram composition:

```text
opaque Telegram candidate
  -> native API hash seal
  -> native fresh session-store key seal
  -> exact Telegram Settings target
  -> current user account provision
  -> TDLib RequestQrCodeAuthentication
  -> qr_authorization_required until AuthorizationStateReady
```

Provider-specific units share only source-session and sanitized receipt
contracts. Mail code не импортирует Telegram workflow; Telegram code не
импортирует Mail workflow.

### Idempotency и receipt custody

Native host использует отдельный owner-private recovery receipt file вне bundle
и current integration stores. Receipt содержит только:

```text
bundle fingerprint
source generation
candidate opaque digest
target configuration instance
completed step identifiers
sanitized terminal state
operation IDs and current public revisions
```

Secret bytes, raw account identity, Vault record/reference, OAuth code и
provider payload запрещены.

Каждая mutation получает stable operation ID из persisted recovery step.
Повторный apply:

- использует persisted target configuration instance;
- reconciles current public query state;
- не использует terminal receipt как substitute current provider readiness;
- повторяет только отсутствующий idempotent provider lifecycle/binding step с
  тем же stable operation ID и persisted public credential revisions;
- не создаёт duplicate target;
- не повторяет ambiguous Vault commit;
- требует explicit retry для `outcome_unknown`;
- отвергает changed bundle fingerprint;
- не считает accepted receipt terminal readiness.

Receipt writes atomic, owner-only and versioned. Повреждённый или unknown
receipt fail closed без target mutation.

## Phase gate

### `legacy_provider_recovery_bundle_v1`

1. Exact immutable bundle inventory, manifest digests и private file modes.
2. Read-only PostgreSQL export с точным inventory `1 Gmail + 1 iCloud +
   1 Telegram user`, две deleted Gmail только counted.
3. Exact no-eval legacy configuration parser и public-only Gmail client export.
4. Exact explicit dev master-key carrier, decrypt proof и wrong-key negative;
   Keychain mutation/fallback запрещены.
5. Changed/missing/extra/symlink/permission/duplicate/unknown-provider
   negatives.
6. Source preparation и dry-run не меняют legacy stores.

### `legacy_provider_native_secret_custody_v1`

1. Native source session с expiry и bundle fingerprint binding.
2. iCloud app password и Telegram API hash никогда не возвращаются browser JS.
3. Existing HPKE owner provisioning ceremony используется без нового Vault
   write path.
4. Fresh Telegram session-store key; legacy TDLib/session key не копируются.
5. Gmail OAuth token/client secret не импортируются.
6. Provider-specific app compositions используют только current public
   Settings/Vault/Mail/Telegram contracts.
7. Idempotent private sanitized receipts и ambiguous-outcome negative.
8. UI показывает только fingerprint, exact counts и sanitized states.
9. Live evidence: два Mail targets, Gmail reauthorization, iCloud honest
   readiness, Telegram real QR до owner scan.
10. Architecture, SRP, formatting, lint, unit/integration и secret-negative
    tests.

## Последствия

- Историческая неоднородность source configuration не размывает current
  Settings или Vault contracts.
- Recovery bundle можно удалить после successful migration; он не становится
  supported import/export format.
- Browser orchestration сохраняет current owner proof и generated clients, но
  не получает legacy secret bytes.
- Telegram account обязательно проходит новый real QR flow.
- Gmail account обязательно проходит current OAuth authorization.
- Communications получает данные только после обычного provider sync и
  durable events.

## Отклонённые варианты

### Передать `.env` и `vault.db` в current integration runtime

Отклонено: это compatibility configuration, arbitrary source access и direct
legacy secret custody внутри managed module.

### Вернуть расшифрованные secrets в browser wizard

Отклонено: plaintext оказался бы в JavaScript memory, DevTools и generic form
state.

### Перенести Telegram session key и TDLib database

Отклонено: это скрытая session migration, отдельно запрещённая ADR-0319.

### Импортировать Gmail legacy OAuth token или client secret

Отклонено: legacy credential shape не доказывает current binding/scopes; owner
проходит real OAuth заново.

### Хранить recovery receipt в source bundle

Отклонено: source должен оставаться immutable, а receipt принадлежит current
host maintenance workflow.
