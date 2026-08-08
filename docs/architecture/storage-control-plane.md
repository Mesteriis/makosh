# Storage Control Plane

Статус: foundation package/Protobuf/AST-admission реализованы; runtime gate закрыт
Дата: 2026-07-16

Полный нормативный contract находится в
[ADR-0224](../adr/ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md).
Это summary не расширяет ADR.

## Topology

```text
Kernel Supervisor
├─ PostgreSQL
├─ PgBouncer
└─ Storage Control
       ├─ cluster/bootstrap
       ├─ roles/grants/budgets
       ├─ migration admission/coordinator
       └─ readiness/reconciliation

module runtime ── StorageBindingV1 + CredentialLease ──> PgBouncer ──> PostgreSQL
```

Kernel владеет process lifecycle, но не SQL implementation. Storage Control
является отдельным managed control-plane process и никогда не проксирует
business queries. Module runtimes ходят только через PgBouncer. Kernel может
работать в `recovery_only` без всего storage stack.

## Packages

```text
backend/src/platform/storage/protocol/     makosh-storage-protocol
backend/src/platform/storage/control/      makosh-storage-control
backend/src/platform/storage/vault/        makosh-storage-vault
backend/src/platform/storage/runtime/      makosh-storage-runtime
backend/src/platform/storage/postgres/     makosh-storage-postgres
backend/src/platform/storage/pgbouncer/    makosh-storage-pgbouncer
backend/src/platform/storage/migrations/   makosh-storage-migrations
```

Только protocol является public dependency Kernel/modules. SQL-free control
package владеет use cases/internal ports, runtime только собирает adapters.
Только PostgreSQL adapter и owner persistence packages получают SQL client.
Owner migration bundle передаётся как admitted artifact, поэтому Storage
Control не зависит от всех owner persistence packages.

## Data boundary

Используются один managed PostgreSQL cluster, одна Макошь database и три fixed
schemas:

- `makosh_data` — owner-prefixed business/provider objects;
- `makosh_platform` — technical state, ledgers и shared outbox/inbox;
- `makosh_extensions` — extension-owned objects.

`PUBLIC CREATE` отозван. Runtime `search_path` содержит только `pg_catalog`, SQL
всегда schema-qualified. Каждый durable owner имеет `NOLOGIN` DDL role и
generation-scoped `LOGIN NOINHERIT` runtime principal. Cross-owner SQL/FK
запрещены. Shared technical tables доступны modules только через exact
versioned functions с caller/generation checks; V1 exact allowlist содержит
только append-outbox и accept-inbox functions. Прямой DML запрещён. RLS может
быть defense in depth, но не заменяет grants/function contract.

## Runtime data path

```text
owner mutation + outbox append
              ↓ one local transaction
       shared technical outbox
              ↓ exact bytes
          Event relay → NATS
```

PgBouncer работает в `transaction` mode. Connection budgets действуют в module
client pool, PgBouncer, PostgreSQL role и global PostgreSQL reserve. Pooler не
является единственной security/budget boundary.

`StorageBindingV1` связывает endpoint/principal/budget с exact storage,
runtime, grant и role generations, а также applied migration bundle digest.
Credential приходит только process-bound Vault lease. Revoke повышает epoch,
останавливает новую выдачу, drain/kill-ит PgBouncer pool, завершает PostgreSQL
backends и ротирует role credential до выдачи нового binding.

Пока same-UID module process не ограничен OS socket/network sandbox, policy не
обещает физическую невозможность открыть скрытый direct endpoint. Modules не
получают его address/credential; PostgreSQL grants и role limits остаются
authority, а endpoint isolation требует отдельного conformance evidence.

## Migration data path

```text
owner persistence package
        ↓ package time
canonical StorageBundleV1
        ↓ signed distribution entry / owner-pinned digest
Storage Control admission
        ↓ digest + PostgreSQL AST + owner/grant checks
owner DDL role + direct admin transaction
        ↓
immutable storage ledger + privilege audit
        ↓ exact digest match
owner runtime start
```

Migration V1 является transactional, additive, owner-local и forward-only.
Module runtime не передаёт SQL и не выполняет DDL. Regex architecture guard —
только ранний heuristic; authoritative enforcement требует AST parser,
roles/grants и Testcontainers PostgreSQL/PgBouncer tests.

## Failure ownership

- Storage Control crash перезапускает только Storage Control.
- PgBouncer crash перезапускает только PgBouncer.
- Pool exhaustion не перезапускает PostgreSQL.
- Planned PostgreSQL restart quiesce-ит modules, drain-ит transactions,
  проверяет ledger/roles/grants, повышает storage generation и выдаёт новые
  bindings.
- Automatic reset, restore, down migration и fallback запрещены.

## Implementation status

Сейчас реализованы six-package foundation, `StorageBundleV1` Protobuf,
structural bundle validation и fail-closed AST admission для owner-local
`CREATE TABLE` / `ALTER TABLE … ADD COLUMN`. PostgreSQL adapter bootstrap-ит
fixed schemas, проводит sanitized readiness probe, фиксирует unique
owner/DDL/runtime mapping вместе с exact binding fences в platform ledger,
создаёт owner-bound роли, применяет step atomically через exact DDL role и
сверяет catalog перед выдачей DML grants только own tables/sequences. SQL-free
lifecycle запрещает replacement binding до завершённого revoke. Binding
детерминированно связывает pool alias с registration/runtime generation; SQL-free
revoker сохраняет lifecycle в `Revoking`, пока не подтвердятся Vault invalidation,
PgBouncer `PAUSE`/`DISABLE`/`KILL` и PostgreSQL `NOLOGIN`/revoke/terminate.
Storage Vault adapter формирует fenced ciphertext-only `RevokeAudience`, а
также однократно issue-ит `Create` или `Resolve` lease для одного
`PlatformCredential`; он store/resolve-ит только AAD-bound encrypted response;
принимает только AAD-bound encrypted response. Concrete trusted
managed Storage→Kernel dispatcher уже существует: он сверяет registration,
caller runtime generation, grant epoch и active Vault generation, затем
подписывает и relay-ит opaque route. `makosh-storage-runtime serve-inherited`
уже принимает только descriptor-bound inherited FD и обслуживает typed status,
привязанный к exact managed runtime generation. Private owner control сохраняет
non-secret desired topology с monotonic revision и завершает active runtime
перед её заменой. Kernel stage-ит отдельную private runtime configuration:
profile, storage generation, identities, exact PostgreSQL/PgBouncer digests и
validated host:port endpoints, а также текущие non-secret `vault_instance_id`,
Vault runtime generation и ephemeral HPKE public key. Это short-lived `0400`
child contract; Vault context не попадает в desired topology или Control Store,
и в нём нет credential либо Vault private key. Child возвращает staged Vault
runtime generation в typed status и отвечает `reconciling` только с этими
exact fences; Kernel сверяет их с текущим Vault status до
успешного return из launch. Credential и runtime attestation не входят в
topology. После Vault credential bootstrap и platform checks child посылает
exact `ManagedRuntimeReadyRequestV1`; Kernel принимает normal status/control
relay только после проверки registration, runtime generation и grant epoch.
Для schema до endpoint contract старая topology сбрасывается
fail-closed и должна быть явно настроена owner. Runtime contract допускает
только topology-matching `desired_bindings` вместе с absolute private path
PgBouncer include; пустой set не имеет path. Production `serve-inherited`
после Vault-admin bootstrap атомарно заменяет include, делает exact `RELOAD`,
проверяет catalog и лишь затем отражает bindings как `ready`; bypass test seam
остаётся `reconciling`. Kernel persist-ит exact descriptor-declared storage request
по registration/capability. Для managed module runtime owner-private control
может выпустить durable non-secret binding только после проверки exact current
managed launch, capability grant, topology и всех fence. Запись имеет строгую
revision sequence; повторная выдача одновременно повышает role и credential
lease epochs, а rollback bundle revision отклоняется. Перед launch Kernel
повторно сверяет каждый staged binding с current managed launch и grant, затем
передаёт только topology-matching set в private child contract. External
attestation пока создаёт лишь authorization fact: его dedicated issuance path
ещё отсутствует. Его client использует only inherited FD,
descriptor handshake и typed route frame.
Перед публикацией PgBouncer alias child теперь materializes typed binding в
opaque deterministic DDL owner/runtime roles и reconciliation-ит их через
PostgreSQL admin adapter. Затем он находит только exact staged
`StorageBundleV1` по owner/revision/SHA-256 binding-а и применяет его под тем же
DDL role до публикации pool. Ошибка role reconciliation, digest match или
migration не публикует alias. Authenticated Docker contour подтверждает этот
порядок с временными runner secrets. Отдельный сценарий на том же contour
проверяет encrypted `RevokeAudience` route Storage child через test Vault
recipient и реальные PgBouncer/PostgreSQL fences для exact staged binding; он
не запускает реальный Kernel dispatcher или Vault process.
Owner command `begin_managed_storage_binding_revocation` сначала атомарно
сохраняет `active → revoking` с exact binding revision. Затем Kernel relay-ит
тот же exact staged binding в live Storage child. Child выполняет
`RevokeAudience`, PgBouncer `PAUSE`/`DISABLE`/`KILL` и PostgreSQL
`NOLOGIN`/revoke/terminate; только complete typed response удаляет binding из
его active set. Ошибка сохраняет durable reservation и останавливает child,
поэтому restart не может снова stage-ить старый binding. Composition test
пропускает Storage credential bootstrap через real Kernel route handler и live
Vault service по inherited Unix channels; он не подменяет ciphertext response.
При managed successor Kernel после durable `revoking`, но до physical Storage
fence, ставит exact predecessor worker `stop_requested`. Этот supervisor-local
marker запрещает autorestart child при ожидаемой потере старого PgBouncer alias,
но не заменяет authority fence. Только после Vault/PgBouncer/PostgreSQL fence и
join predecessor разрешена reservation generation `N + 1`; ошибка любого шага
оставляет binding `revoking` и не выдаёт successor identity.
Opt-in disposable Docker runner создаёт временный signed macOS release bundle
для собранных Vault и Storage binaries, принимает оба через production
release-binding path и запускает через production Kernel launch path. Он
импортирует service-scoped file credentials into Vault, checks fenced
`reconciling` status, затем independently stops и restarts Storage. Новый child
проходит Vault-backed startup с новой generation и повторной signed-artifact
verification; это conformance signed-release execution в disposable bundle, а
не Developer ID signing, notarization или production release attestation.
Storage release admission уже фиксирует один exact signed platform artifact,
descriptor digest и binding revision. Перед каждым status child делает bounded
credential-free TCP preflight staged PostgreSQL и PgBouncer endpoint'ов:
неуспех даёт только `failed/storage_endpoint_unavailable`; при пустом binding
set успех оставляет `reconciling`. Это reachability check, не endpoint
attestation и не credential delivery; binding issuance и полноценная
infrastructure composition всё ещё не подключены.
PgBouncer adapter формирует только exact commands из fenced binding и имеет
отдельный simple-query admin transport: endpoint и short-lived credential
находятся только в Storage process. Он не доступен module processes и не
передаёт credential Kernel. Его private `0600` database include заменяется
через fsync/rename/directory-fsync только в Storage-owned `0700` directory;
runtime alias сортируются, дубликаты и symlink paths отклоняются, затем adapter
выполняет exact `RELOAD`. PostgreSQL adapter отклоняет несовпадающий binding.
Disposable development Compose smoke реально выполняет PostgreSQL и PgBouncer
`SELECT 1` и проверяет NATS health; отдельный opt-in test создаёт и сверяет
PostgreSQL roles/migrations и runtime endpoint preflight на живом instance. Это не является runtime
attestation или production evidence. Конфигурация переносит валидированный
Vault public route context. Before the first Storage launch, the Vault
initializer may import exactly two owner-private file-backed service secrets:
`pgbouncer-admin-password` and `postgres-admin-password`. Their exact scopes
are `storage_main` generation `1`; files are regular, non-symlink,
owner-private inputs and their content never enters Kernel, Control Store or
logs. При startup managed Storage child по descriptor-bound inherited FD
сначала resolve-ит existing platform credential, а при отсутствии записи Vault
сам генерирует opaque token; Kernel получает только ciphertext и generation
response не содержит plaintext. A generated token cannot authenticate an
already provisioned database deployment, so it does not by itself establish
runtime readiness. Runtime wiring
Для PgBouncer и PostgreSQL runtime запрашивает отдельные Vault purposes:
plaintext приходит только через fresh `Resolve` response. Для каждого staged
binding он использует отдельный runtime-principal Vault scope и exact
`credential_lease_revision`: сначала пытается `Resolve`, а для отсутствующей
record выполняет `Create`/opaque token generation и новый `Resolve`. Token
остаётся только в zeroizing runtime memory и передаётся PostgreSQL через
server-side quoted `ALTER ROLE ... PASSWORD`; он не появляется в Control Store,
environment, PgBouncer include или Kernel. До normal control loop runtime
выполняет bounded `SHOW VERSION` authentication в PgBouncer и idempotent
bootstrap fixed PostgreSQL schemas под отдельным admin user. Exact
owner-role/migration lifecycle теперь запускается до pool publication. Для
одного exact binding live contour уже покрывает encrypted revoke route и
PgBouncer/PostgreSQL session fencing и real Vault service delivery для Storage
credential bootstrap. External binding issuance требует owner session, exact
attestation, runtime generation и grant epoch; она сохраняет только non-secret
binding. После proof-backed session external runtime может получить лишь exact
canonical binding, PgBouncer endpoint и текущий Vault public context; пароль
остаётся доступен только через отдельный fenced HPKE Vault route. Команда
`make -C backend test-storage-external-process` подтверждает live external
credential delivery и rotation через owner-control IPC: временный signed Kernel
bundle запускает real Vault child, а отдельный proof-backed external process
получает binding и выполняет HPKE lease route. После owner-authorized external
revocation прежний binding получает `runtime_session_stale`, а successor
получает другой credential; process сохраняет лишь SHA-256 assertions вместо
plaintext. Authenticated Docker contour проверяет owner-local migrations, роли
и Vault-delivered credentials against real PostgreSQL/PgBouncer. Immutable
migration ledger остаётся admin-only. Ограничение same-UID direct endpoint
документировано как ограничение evidence, а не как ложное заявление о sandbox
изоляции. Эта совокупность conformance открывает `storage_control_v1`.

Отдельный disposable authenticated Compose contour проверяет, что PgBouncer
отвергает неверный admin password и принимает credential из private temporary
file, смонтированного именно как Docker secret. Второй независимый secret
даёт Storage runtime выполнить PostgreSQL fixed-schema bootstrap. Runner
удаляет project, volumes и files после test. Эти secret создаются runner-ом, а
не Vault, поэтому test не считается evidence production Vault deployment,
binding issuance или credential rotation.

Этот contour также монтирует Storage-owned database include как private
directory. Он проверяет, что runtime применяет validated topology-matching
binding: атомарно добавляет generation-scoped alias, делает PgBouncer `RELOAD`
и сверяет alias через catalog. Отдельный runtime revoke scenario проверяет
fencing exact binding, но это всё ещё не доказывает owner-authorized binding
issuance через owner control IPC или полный external Kernel→Vault process path.

Для будущего managed initial cluster bootstrap PostgreSQL adapter уже создаёт
one-shot `initdb --pwfile`: только в Storage-owned `0700` runtime directory,
exclusive `0600` file, `fsync`, explicit removal и best-effort drop cleanup.
Он не запускает `initdb` сам и пока не связывает этот password file с
managed PostgreSQL lifecycle; это отдельная оставшаяся runtime composition.
