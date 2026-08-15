# ADR-0278: Mail Gmail OAuth setup and refresh gate

Статус: Принято
Дата: 2026-07-25
Состояние реализации: Реализовано 2026-07-26. Generated
start/complete/refresh/query routes,
Mail-owned PKCE attempts и durable operations, descriptor revision 3, settings
schema revision 4, Storage bundle revision 4, action-specific Vault admission,
typed Gmail HTTPS exchange и runtime orchestration реализованы. Gmail
sync/delivery теперь резолвят access credential on demand по Mail-owned opaque
binding, а не по settings revision. Live provider/Vault conformance доказывает
exact PKCE/form exchange, отзывчивость control route во время provider I/O,
one-use completion, CAS rotation, stale/revoke fences, sanitized output и
отсутствие hidden retry после ambiguous provider/Vault outcome. Focused,
architecture, SRP, Cargo, Clippy и full backend gates зелёные;
`mail_gmail_oauth_v1` открыт.

Client update 2026-08-14 реализует loopback browser callback для web/dev
surface: setup, permanent-delete authority, portability и legacy recovery
открывают exact Google authorization URL и автоматически передают matching
`state` и one-use code в generated Mail completion contract. Callback принимает
только exact Google endpoint, exact текущий loopback redirect, `response_type`
`code` и PKCE `S256`, удаляет query из browser history до дальнейшей обработки
и использует отдельный same-origin channel, scoped by exact `state`. UI больше
не предлагает ручной copy/paste `state` или authorization code; эти значения не
попадают в логи или rendered callback copy. Native Tauri/system-browser
loopback host этим client slice не реализован и остаётся отдельным gate.

Уточняет:

- ADR-0204: integration/provider-neutral context boundary;
- ADR-0205: Core Gateway and OAuth callback transport;
- ADR-0215: capability grants;
- ADR-0220: durable command and terminal result;
- ADR-0223: Vault leases and secret classes;
- ADR-0243: provider OAuth credential rotation;
- ADR-0270: Mail Kernel admission;
- ADR-0277: Gmail outbound mutation.

## Контекст

Legacy Gmail setup смешивал Mail integration, Communications account facade,
host-global pending state, Calendar/Contacts side effects и один JSON token
bundle. Это historical product evidence, а не допустимый clean-room contract.

Gmail production delivery уже получает exact access-token lease, но token
пока seed-ится conformance harness. Наличие HTTPS exchange helper и PostgreSQL
таблицы binding не создаёт admitted setup/refresh workflow. Реальная
реализация обязана одновременно решить:

- PKCE state и one-use authorization code;
- owner-specific client route без Mail schema в Gateway;
- action-specific Vault authority;
- отдельные access/refresh secret classes;
- owner-local durable operation state;
- restart/revoke fencing;
- отсутствие token/code/verifier в Communications, events и diagnostics.

## Решение

Gmail OAuth является operational workflow Mail integration:

```text
Desktop / Android client
        ↓ generated Mail OAuth contract
Core Gateway
        ↓ opaque owner-declared ClientRpc
Kernel capability router
        ↓
Mail managed runtime
        ├─→ Mail-owned PKCE/operation state
        ├─→ makosh-mail-gmail HTTPS token exchange
        └─→ Kernel-issued action lease → encrypted Vault
```

Mail OAuth не является:

- Communications command или account facade;
- отдельным business domain;
- Kernel/Gateway OAuth implementation;
- поводом создать общий provider account domain;
- прямым side effect в Calendar, Contacts или другом owner.

Другие owners могут реагировать только на отдельные typed business events или
workflow commands после собственных ADR/gates. Этот gate не публикует
credential lifecycle в Communications.

### Exact Core/Kernel agreement

Generated Mail API получает четыре независимых operational contracts:

| Capability | Kind | Responsibility |
|---|---|---|
| `mail.oauth.start.v1` | request RPC | создать bounded one-use PKCE attempt и вернуть authorization URL |
| `mail.oauth.complete.v1` | durable command | принять exact setup/state/code и durably начать token exchange |
| `mail.oauth.refresh.v1` | durable command | начать rotation существующего Mail-owned binding |
| `mail.oauth.query.v1` | query RPC | вернуть только sanitized terminal operation state |

Gateway аутентифицирует client/device session и переносит opaque generated
bytes. Kernel проверяет exact descriptor route, approved capability, runtime
generation и grant epoch. Ни Gateway, ни Kernel не декодируют Mail payload,
не знают Google endpoint/scopes/client ID, не хранят PKCE state и не получают
token plaintext.

Обычный HTTP callback допускается ADR-0205 только как transport exception.
Он обязан проверить bounded `state` и передать callback в exact owner-declared
Mail OAuth route через owner-neutral routing. Он не является REST business API
и не компилирует Mail implementation. До отдельного callback transport
evidence generated ClientRpc `complete` остаётся каноническим backend
completion seam.

### PKCE и provider boundary

Mail генерирует cryptographically random `setup_id`, `state` и
`code_verifier`, сохраняет pending attempt в своей PostgreSQL schema и
возвращает только:

- opaque `setup_id`;
- authorization URL;
- expiry.

`code_verifier` не возвращается клиенту и не покидает Mail owner. Pending
attempt имеет bounded TTL, exact redirect URI, client ID/settings revision,
scope set, one-use completion marker и конфликтующий replay denial.

Production endpoints и scopes являются signed Mail configuration:

```text
https://accounts.google.com/o/oauth2/v2/auth
https://oauth2.googleapis.com/token
openid
email
https://www.googleapis.com/auth/gmail.modify
https://www.googleapis.com/auth/gmail.send
```

Client не передаёт произвольный authorization/token endpoint или scope.
Первоначальный read-only scope заменён ADR-0307 при открытии typed message flag
mutations. Existing grant с `gmail.readonly` не повышается автоматически и
требует явной повторной authorization.
Loopback TLS provider endpoint/custom CA допускаются только compile-time
`conformance-test-support`. HTTPS exchange имеет whole-operation deadline и
bounded response.

### Vault capability units

Access и refresh material не объединяются в JSON bundle:

```text
mail_gmail_access_token
  class = ProviderCredential

mail_gmail_refresh_credential
  class = OAuthRefreshCredential
```

Approval units:

```text
mail.gmail.oauth-setup.credentials.v1
  Create access-token revision 1
  Create refresh-credential revision 1
  ReplaceCas both credentials for explicit re-authorization

mail.gmail.oauth-refresh.credentials.v1
  Resolve current refresh credential
  ReplaceCas access token at current + 1
  ReplaceCas refresh credential only when provider rotates it
```

Existing `mail.gmail.credentials.v1` сохраняет только `Resolve` access token
для sync/delivery. Setup, refresh и provider execution являются разными
functional responsibilities и могут approved/revoked независимо.

Kernel выдаёт lease только как пересечение signed descriptor, owner-approved
GrantSet и current runtime fences. Credential plaintext проходит только в HPKE
Vault session между exact Mail runtime и Vault; Kernel видит только ciphertext
и sanitized route metadata.

### Durability и failure semantics

Start, complete и refresh имеют owner-local idempotency state. `accepted`
означает только durable Mail persistence, а не Google/Vault completion.
Terminal states:

```text
pending
completed
rejected
outcome_unknown
```

Authorization-code exchange и refresh HTTP mutation могут иметь ambiguous
outcome. После потери ответа runtime не выполняет silent automatic retry.
Новый attempt требует нового explicit operation согласно сохранённому state.

Mail хранит только opaque Vault record IDs, revisions, expiry и sanitized
provider scope metadata. Authorization code, PKCE verifier, access token,
refresh credential и client secret не попадают в durable command result,
events, subjects, logs, errors, health или settings.

Каждый успешный Vault step фиксируется в owner-local operation state до
следующего step. Crash между Vault response и PostgreSQL checkpoint может
оставить недостижимый encrypted Vault record, но не может дать ложное
`completed`, опубликовать secret или привязать частичный credential set.
Повторный setup создаёт новые revisions; lifecycle/cleanup такого orphan
остаётся Vault responsibility и не реализуется cross-owner SQL.

### Build units

Responsibilities остаются раздельными:

```text
makosh-mail-api          generated operational contracts
makosh-mail-core         PKCE/state/operation decisions without I/O
makosh-mail-gmail        Google HTTPS/form/JSON adapter
makosh-mail-persistence  owner-local attempts, bindings and operation state
makosh-mail-runtime      composition and Vault/provider orchestration
makosh-managed-vault-client
                         provider-neutral correlated encrypted Vault port
```

`makosh-mail-gmail` не зависит от persistence/runtime/Kernel/Gateway/Vault или
Communications. Platform Vault client не знает Gmail/Mail purposes.

## Phase gate `mail_gmail_oauth_v1`

Gate открывается только при одновременном evidence:

1. exact generated start/complete/refresh/query contracts;
2. descriptor capabilities разделены по route и Vault action;
3. signed descriptor/settings/storage artifacts;
4. durable one-use PKCE attempt и replay/expiry/conflict tests;
5. loopback TLS authorization-code exchange с exact form evidence;
6. correlated V2 Vault `Create` для двух разных secret classes;
7. owner-local binding содержит только opaque record IDs/revisions;
8. refresh разрешает current refresh `Resolve` и access-token `ReplaceCas`;
9. stale revision, runtime generation, grant epoch и revoke fail closed;
10. ambiguous HTTP/Vault failure не становится completed и не retry-ится
    скрыто;
11. token/code/verifier/secret отсутствуют в client result, durable events,
    diagnostics и health;
12. Kernel/Gateway/Communications не импортируют Mail implementation, Mail не
    импортирует другой domain/runtime;
13. focused live conformance и architecture/SRP/Cargo/full backend gates
    зелёные.

### Implementation evidence

Live managed conformance запускает signed Mail, Vault и Storage runtimes с
loopback TLS provider fixture:

- `managed_mail_gmail_oauth_rotates_credentials_once_and_fails_closed`
  проверяет exact authorization-code/refresh form, pending status во время
  delayed provider response, one-use setup, access/refresh CAS revision 1 → 2,
  stale revision rejection, `outcome_unknown` без повторного provider request,
  отсутствие Communications event и private values в diagnostics;
- `managed_mail_gmail_oauth_route_is_fenced_by_owner_revoke` проверяет, что
  owner revoke закрывает exact OAuth route до provider mutation;
- существующий
  `managed_mail_gmail_runtime_mutates_once_and_replays_event_without_private_payload`
  подтверждает, что delivery резолвит access credential только через
  Mail-owned opaque binding и не возвращает seeded-settings fallback.

Owner unit tests отдельно фиксируют production endpoint policy и
compile-time conformance endpoints. Full `make ci` проверяет workspace,
integration profile, architecture/SRP/Cargo boundaries, dependency policy и
supply-chain evidence.

Loopback web/dev popup/callback UX реализован в frontend и покрывает обычный
setup, permanent-delete authority, portability и legacy recovery. Он использует
этот owner contract, но не расширяет и не доказывает backend admission gate.
Native Tauri/system-browser callback host и Scheduler-driven proactive refresh
остаются следующими client/job slices.

## Отклонённые варианты

### Хранить legacy token bundle одним секретом

Отклонено: access execution и refresh rotation получают лишний plaintext и
не могут иметь независимые secret classes/actions/revisions.

### Передать OAuth endpoint, scopes или client secret в command

Отклонено: command превратился бы в SSRF/configuration/secret transport и
обошёл signed settings/Vault.

### Реализовать Gmail callback как Communications REST route

Отклонено: provider setup принадлежит Mail integration, а Communications не
является provider account facade.

### Обновлять Calendar/Contacts после Gmail setup напрямую

Отклонено: это direct cross-domain side effect. Отдельный workflow может
реагировать на typed owner event только после собственного admission.

## Последствия

Production Gmail больше не зависит от seeded access token как конечной модели.
Kernel/Core остаётся neutral control/router, Mail владеет provider workflow и
durable binding, Vault владеет credential plaintext, а Communications не
участвует в OAuth.
