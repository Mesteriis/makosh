# ADR-0215: Открытая регистрация модулей и capability grants

Статус: Принято
Дата: 2026-07-15
Состояние реализации: private Control Store registry record имеет persisted
`pending/approved/suspended/revoked/blocked_incompatible` state, exact
descriptor SHA-256 и per-registration grant epoch fencing. Имеются private
owner-control, registration и external-runtime IPC evidence surfaces в
production Kernel: bounded `Hello`/`Begin`/single-use `Describe`/own-status
handshake, descriptor admission и owner approval operations.
Owner-control mutation проходит отдельный challenge/session proof enrolled
file-backed ES256 device key: Kernel проверяет присланную signature и не
открывает file signer вместо owner client. Session привязана к instance,
Control Store generation и enrolled owner identity; reset или смена identity
инвалидируют её.
Capability-router authorizer проверяет requested/approved grant,
hard Kernel policy и exact external runtime generation/current grant epoch.
Owner может pin-нуть отдельный ES256 public key external
runtime; новая binding revision повышает registration epoch и invalidates
старую attestation.
Private external-runtime-session socket принимает exact 32-byte distribution
artifact SHA-256, выдаёт одноразовый 32-byte challenge и принимает только raw
ES256 proof привязанного key. Первый успешный proof связывает registration,
runtime identity/generation, current grant epoch и этот digest, после чего
Kernel создаёт external attestation; заранее созданная development attestation
не требуется. Повторный exact proof идемпотентен только для той же current
attestation; stale generation, epoch или другой digest отклоняются. Kernel
выдаёт short-lived session лишь после этой проверки;
каждая capability request повторно сверяет registration/runtime generation и
grant epoch. Это owner-isolated `0600` local production IPC; session является
короткоживущим local bearer credential и не даёт PID, container name или Docker
identity authority.
Owner transition в `suspended` или `revoked` сначала durable повышает
registration/grant epoch, затем одной Control Store transaction переводит все
active Storage bindings этой registration в `revoking`. Kernel запрашивает
существующий Storage Control physical fence и в любом случае пытается
остановить affected managed runtime. Недоступный или отклонивший revoke Storage
оставляет bindings в `revoking` и возвращает incomplete result; active binding
и ложный successful revoke не восстанавливаются.
Эти части проверяются lifecycle и architecture-тестами. Наличие owner-control,
Gateway, Vault, Storage, Event Hub или business-owner кода само по себе не
открывает новый production inventory: каждый owner и data-plane gate требует
своего exact admission evidence по текущей policy.

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0209: Kernel Event Hub и контроль подписок](ADR-0209-kernel-event-hub-and-subscription-control-plane.md);
- [ADR-0210: Telemetry Hub и локальная диагностика](ADR-0210-telemetry-hub-and-local-diagnostics.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Durable Job Platform, Scheduler и горячее изменение заданий](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md).

Уточняется:

- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR закрывает отложенную в ADR-0206 модель локальной module identity и
capability authorization. Он уточняет прежние формулировки про bundled
allowlist: bundled distribution остаётся способом поставки собственных
integration experiences, но наличие подписи binary или предварительное
включение executable в distribution allowlist не является обязательным
условием для регистрации локального module runtime. ADR-0219 отдельно
требует exact-byte verification для любого process, который запускает
Kernel в lifecycle mode `managed`.

Owner/device identity и authorization owner operations определены ADR-0218.
Этот ADR не определяет pairing UX Android-клиента, provider account/agent
identity и remote workload federation. Managed distribution/update/rollback
граница определена ADR-0219.

## Контекст

Макошь работает в контролируемой локальной среде: владелец устанавливает и
запускает Kernel и module runtimes на своём устройстве. Закрытый список
скомпилированных вместе executables сделал бы добавление нового модуля дорогим
и снова связал бы независимые owners с release cycle Kernel.

Одновременно доверять любому процессу, который назвал себя `telegram` или
`tasks`, нельзя. Все процессы одного desktop user обычно имеют одинаковые
UID/GID, PID переиспользуется, а self-declared `module_id` не является
доказательством identity. До явного решения владельца неизвестный процесс не
должен получать PostgreSQL role, NATS permissions, Vault lease, BlobRef,
business RPC или job delivery.

Нужна модель, в которой:

- локальный module может обнаружиться без пересборки Kernel;
- владелец видит запрос прав и явно принимает решение в owner grant approval
  surface Core Gateway;
- модуль не может сам выдать себе capability;
- права меняются во время работы и немедленно отзываются;
- Kernel различает runtime, которым он управляет, и внешний runtime;
- открытая registration без publisher signature не превращается в доверие
  к `module_id` или в право на managed launch;
- отказ одного runtime не расширяет права и не затрагивает соседние modules.

## Trust boundary

Первая реализация принимает следующие ограничения:

- module runtime является локальным OS-процессом на том же host;
- registration listener доступен только через private local IPC endpoint;
- remote TCP/QUIC registration и network-discovered modules запрещены;
- Kernel data/runtime directories доступны только владельцу Макошь;
- полностью скомпрометированный user account, Kernel process или host root
  находится вне threat model первой версии;
- approval означает доверие конкретной локальной регистрации и набору прав,
  но не сертификацию publisher, supply chain или качества кода.

Расширение на недоверенные third-party modules, multi-user host, remote host
или marketplace требует sandbox/attestation/update ADR и не может использовать
этот local trust boundary без изменений.

## Решение

### Открытая, но недоверенная регистрация

Эта surface открывается только gate `module_control_plane_v1` ADR-0225. В
`kernel_recovery_only_v1` registration endpoint, `pending` state, approval и
GrantSet mutations отсутствуют; «открытая» означает policy после gate, а не
текущую runtime availability.

Любой локальный процесс, достигший private registration endpoint, может начать
bounded registration handshake. Это не даёт ему runtime capabilities.

До approval процесс имеет состояние `pending` и может вызвать только:

- `Hello` — согласование версии registration protocol;
- `Describe` — передача exact bounded `ModuleDescriptorV1` и запрошенных
  capabilities ADR-0221;
- registration health/status — только состояние собственной попытки без
  сведений о других modules.

`pending` процесс не получает:

- PostgreSQL/PgBouncer credentials;
- NATS account, publish или subscribe permissions;
- Vault или Blob capabilities;
- business query/request/command routing;
- Job Platform delivery;
- event catalog subscriptions;
- diagnostics других runtime;
- право стать managed child.

Registration protocol имеет ограничения размера descriptor, частоты попыток,
числа pending registrations и времени жизни session. Malformed, incompatible
или resource-abusive session закрывается fail-closed. Повторный descriptor не
создаёт новые права и не обходит существующее состояние регистрации.

### Реестр модулей и owner grant approval surface

Kernel Module Registry является владельцем технической записи регистрации.
Owner grant approval surface через Core Gateway показывает минимум:

- `ModuleRegistrationId` — непрозрачный стабильный ID записи Kernel;
- заявленный `module_id` и display metadata;
- `runtime_instance_id` активного подключения, если оно существует;
- runtime/registration/descriptor protocol versions;
- lifecycle mode `managed` или `external`;
- предоставляемые contracts и JobKind descriptors;
- запрошенные capabilities, subscriptions и resource budgets;
- одобренные права и ограничения scope;
- текущее registration/runtime state;
- `grant_epoch`, last seen и sanitized failure/block reason.

Descriptor является заявлением модуля, а не источником доверия. Private
content, credentials, bearer material, provider sessions и environment values
не сохраняются в registry и не показываются в approval surface.

Этот surface управляет только admission/grants. Module configuration
`operator_managed`/`kernel_managed`, desired/effective revisions и apply state
принадлежат отдельному Kernel Settings Registry ADR-0222. Изменение setting не
является grant approval и не может расширить capability set.

Только authenticated owner device с подходящей capability и fresh privileged
proof ADR-0218 может:

- одобрить только часть запрошенных прав;
- изменить scopes и resource budgets;
- временно приостановить регистрацию;
- отозвать её;
- явно выбрать lifecycle mode;
- просмотреть изменения descriptor до выдачи новых прав.

Новый запрос capability после обновления module остаётся невыданным, пока
владелец не одобрит его отдельно. Уже выданные права не расширяются молча.

### Состояния регистрации

Логический автомат регистрации:

```text
pending → approved ↔ suspended → revoked
    └──────────────→ revoked

pending / approved / suspended
    → blocked_incompatible
```

- `pending` — descriptor виден, прав нет;
- `approved` — registration разрешена, session может получить effective
  grants после полного handshake;
- `suspended` — новые операции и credentials не выдаются, активные права
  отозваны; состояние можно вернуть в `approved` явным действием;
- `revoked` — регистрация окончательно отозвана; повторное подключение создаёт
  новую pending registration и не наследует grants;
- `blocked_incompatible` — protocol, descriptor или hard policy несовместимы;
  права не выдаются до исправления и новой проверки.

Runtime health (`starting`, `ready`, `degraded`, `failed`, `stopped`) является
отдельным измерением и не заменяет authorization state. `approved` не означает
`ready`, а `failed` не означает автоматический `revoked`.

### Managed и external lifecycle

Одна registration имеет ровно один явный lifecycle mode.

#### `managed`

Владелец одобряет launch configuration. Kernel:

- проверяет `ManagedLaunchBinding` ADR-0219 и exact executable bytes
  перед каждым launch/restart;
- запускает process;
- создаёт launch record и private inherited control channel;
- применяет startup, health, crash-loop, drain и shutdown budgets;
- может restart, quiesce, drain, stop и принудительно завершить именно этот
  child process;
- сообщает actual state и restart history.

Kernel гарантирует restart/failure isolation только для `managed` runtime.
Pending или suspended registration никогда не запускается.

#### `external`

Process запускается владельцем, OS service manager или другим явно выбранным
локальным mechanism. Kernel:

- аутентифицирует регистрацию;
- выдаёт и отзывает grants;
- маршрутизирует разрешённые contracts;
- наблюдает health/disconnect.

Kernel не посылает process signals, не меняет его executable и не обещает
restart. Disconnect переводит capability в unavailable/degraded и требует
внешнего запуска либо явного перехода registration в `managed`.

Переключение `managed ↔ external` является отдельной authorized operation:
active runtime сначала quiesce/drain, `grant_epoch` повышается, старая session
закрывается, затем выполняется новый lifecycle handshake. Переход в
`managed` дополнительно требует signed bundled entry либо owner-pinned
digest. Silent topology fallback запрещён.

### Runtime identity и session binding

Строка `module_id`, PID и Unix UID/GID по отдельности не являются logical
module identity.

Для `managed` runtime Kernel связывает identity с:

- одобренным `ModuleRegistrationId`;
- созданным Kernel launch record;
- уникальным `runtime_instance_id` и generation;
- private inherited socket/FD;
- одноразовым challenge nonce;
- текущим `grant_epoch`.

Поля `Hello` используются как consistency checks. Identity берётся из launch
record, а не из самоописания child. Новый restart получает новый channel,
nonce, instance ID и generation.

Для `external` runtime approval привязывает registration к сгенерированному
module-instance public key. При reconnect Kernel отправляет одноразовый nonce,
а module доказывает владение private key подписью challenge. Private key
остаётся в локальном private state модуля; Kernel хранит только public key и
metadata регистрации. Эта подпись доказывает продолжение конкретной
регистрации, но не целостность binary, publisher identity или code signing.

Unix peer credentials и PID используются только как defense-in-depth и audit
correlation. Session короткоживущая, привязана к одному control connection,
runtime instance и grant epoch; её нельзя переносить в другой process или
использовать после reconnect.

### Вычисление effective grants

Единственная формула выдачи прав:

```text
requested by ModuleDescriptorV1
∩ owner-approved grant state
∩ hard Kernel policy
= effective GrantSet
```

- `ModuleDescriptorV1` задаёт только верхнюю границу запроса;
- owner grant approval state задаёт выбранное владельцем подмножество;
- hard Kernel policy запрещает архитектурно невозможные edges независимо от
  UI approval;
- отсутствие любого из трёх условий означает deny.

Hard policy, например, не позволяет integration получить domain persistence,
domain — provider operational contract, Scheduler — provider Vault lease, а
одному owner — чужую PostgreSQL schema. Ни grant approval surface, ни module
settings ADR-0222 не являются обходом ADR или architecture guard.

`GrantSet` является typed server-side state и содержит минимум:

- capability/contract ID и version range;
- разрешённые actions;
- owner/resource/account scopes, когда они применимы;
- audience `ModuleRegistrationId` и `runtime_instance_id`;
- `grant_epoch`, issued/expiry time;
- bounded rate/concurrency/resource budgets.

Broad wildcard grants, общий superuser credential и произвольные строковые
permissions запрещены. Kernel проверяет GrantSet для каждого routed RPC и
передаёт service boundary signed authorization/fencing context. Узкий service
сам выдаёт typed binding либо credential:

- `StorageBindingV1` через Storage Control, а database secret — отдельным
  scoped Vault `CredentialLeaseV1` ADR-0223/ADR-0224;
- NATS publish/subscribe permissions через Event Hub;
- scoped Vault lease;
- expiring BlobRef capability;
- Job Platform command/handler authorization;
- Gateway/control-plane methods.

Module никогда не получает Kernel master credential и не может mint/attenuate
grants самостоятельно.

Vault grant не является secret reference или готовым credential. Descriptor
объявляет bounded `VaultPurposeRequestV1`, а Vault после повторной проверки
effective `GrantSet` выдаёт `CredentialLeaseV1` ADR-0223. Lease привязан к
`vault_instance_id`, `vault_runtime_generation`, logical owner/configuration и
purpose, audience `ModuleRegistrationId + runtime_instance_id`, текущему
`grant_epoch` и secret revision. Default TTL равен 10 минутам, hard maximum —
1 часу; initial lease разрешает не более одного `Resolve`, а renewal создаёт
новый lease после полной authorization check.

Vault restart/lock/restore, смена generation, module restart, suspend/revoke
или смена `grant_epoch` инвалидируют lease. Revoke закрывает transport session,
запрещает новые resolve/renewal и требует quiesce/stop затронутого runtime, но
не может отозвать уже скопированные plaintext bytes или отменить выполненный
external side effect.

### Горячее изменение и отзыв

Изменение grant approval не требует restart всего Макошь и не смешивается с
settings apply lifecycle ADR-0222:

1. Registry сохраняет новую revision и повышает `grant_epoch`;
2. Kernel перестаёт маршрутизировать новые операции по старому epoch;
3. старые runtime sessions закрываются либо переводятся в re-authentication;
4. Event Hub, storage, Vault, Blob и Job Platform получают revoke/fencing;
5. затронутый runtime выполняет bounded quiesce/drain, если это безопасно;
6. новая session получает только новый effective GrantSet.

Stale request проверяется по session-bound grant epoch. Durable event,
observation, result, Ack или job completion использует typed
`SourceFenceV1(GRANT_EPOCH)` ADR-0220, когда catalog требует grant fencing.
Consumer сверяет scope/epoch с current control-plane state до owner payload
decode и canonical mutation. Job execution/provider account lease остаётся в
owner payload как отдельный typed fence и не смешивается с grant epoch.
Отзыв не считается завершённым, пока downstream credentials/leases не отозваны
либо не истекли и actual state не отражает это явно.

Для storage смена `grant_epoch` блокирует новые bindings, но сама по себе не
отзывает уже аутентифицированную SQL session. Storage revoke остаётся в
`revoking`, пока generation-scoped role не переведена в `NOLOGIN`, старый
PgBouncer alias не disable/drain/kill-нут, matching PostgreSQL backends не
завершены и Storage Control не подтвердил zero sessions ADR-0224. Только после
этого разрешён new role/alias/binding; Vault lease expiry не заменяет этот
fencing.

Для операции с уже начатым non-idempotent external side effect применяется
`unknown_outcome`; Kernel не повторяет её автоматически и не делает вид, что
revocation отменил внешний результат.

### Binary signing и distribution provenance

Publisher signature и digest allowlist не требуются для создания
`pending` registration или approved lifecycle mode `external`.

Для `managed` launch provenance обязательна: bundled executable связан с
signed distribution manifest, а local external executable — с explicit
owner-pinned SHA-256 digest. Exact bytes проверяются перед каждым
launch. Download/install не принадлежат Kernel, rollback не выполняется
автоматически. Полный contract определён ADR-0219.

### Persisted control state

Этот ADR определяет логическую модель Registry, registrations, approval,
grant revision и lifecycle mode. Физическое хранение выбрано
[ADR-0216](ADR-0216-private-kernel-control-store-with-sqlite.md): private
kernel-owned SQLite adapter.

Эти данные обязаны переживать restart Kernel и быть доступны в
`recovery_only`, когда PostgreSQL, PgBouncer, NATS и Vault не запущены. Поэтому
единственная authoritative копия registry/grants находится в Kernel Control
Store, а не в canonical PostgreSQL и не за Vault boundary.

Settings schema/values/revisions физически используют тот же private Control
Store по ADR-0216, но остаются отдельной typed model ADR-0222 и не участвуют в
формуле effective GrantSet.

## Выбор библиотек первой реализации

Модель не требует отдельного policy server или distributed workload identity:

- local IPC и managed launch channel строятся на Unix sockets/owned FDs;
- `nix` может использоваться для OS-specific peer credential checks;
- `getrandom` — для challenge nonce;
- `zeroize` — для чувствительных transient buffers;
- `ed25519-dalek` — для proof-of-possession external registration key, если
  implementation review подтверждает актуальный API и dependency boundary.

[SPIRE](https://github.com/spiffe/spire) не вводится: это полноценная
multi-workload identity infrastructure с server/agent и datastore, избыточная
для одного контролируемого local host.

[Cedar](https://github.com/cedar-policy/cedar) не вводится: первая hard policy
является маленьким закрытым typed rule set, а Cedar всё равно требует уже
аутентифицированного principal и отдельного policy lifecycle.

[Biscuit](https://github.com/eclipse-biscuit/biscuit-rust) не вводится в
первую версию: переносимые attenuated bearer capabilities не нужны, пока все
проверки выполняет Kernel и service-native credential issuers. Biscuit может
быть рассмотрен отдельно, если grants потребуется проверять нескольким
независимым процессам или безопасно делегировать через transport boundaries.

## Отклонённые варианты

### Доверять любому локальному подключению

Отклонено: один same-user process получил бы данные и полномочия всех modules.

### Считать `module_id`, PID или UID identity

Отклонено: `module_id` self-declared, PID переиспользуется, а UID/GID обычно
общие для всех processes Макошь.

### Требовать signed bundled executable для любой registration

Отклонено: открытая external registration не даёт Kernel launch authority
и не требует release cycle. Но любые bytes, которые Kernel запускает
как `managed`, обязаны пройти integrity binding ADR-0219.

### Один общий PostgreSQL/NATS/Vault credential

Отклонено: отзыв одного module невозможен без остановки всех, а compromise
пересекает owner boundaries.

### Только статический grant config без approval revision

Отклонено: новое право нельзя безопасно согласовать или отозвать во время
работы, а actual state расходится с config без явного fencing.

### Автоматически перезапускать external runtime

Отклонено: Kernel не владеет process handle и launch contract. Такая попытка
создала бы duplicate runtime и split-brain.

### SPIRE, Cedar или Biscuit в первой версии

Отклонено как преждевременная operational и compile complexity. Решение можно
пересмотреть только при появлении конкретного требования, которое typed local
GrantSet не удовлетворяет.

## Проверка решения

До изменения `Состояние реализации` обязательны tests:

- неизвестный local process становится `pending` и не получает ни одной
  data-plane capability;
- approval subset выдаёт только пересечение requested/approved/hard policy;
- owner approval и module settings не могут разрешить architecture-forbidden
  grant;
- новый requested grant после reconnect остаётся pending approval;
- module process с тем же UID не может approve/revoke собственную registration;
- owner device revoke инвалидирует owner session до следующей mutation;
- duplicate `module_id` не объединяет registrations и не наследует права;
- managed child identity связана с launch record/channel, а не полями Hello;
- managed launch без signed bundled entry либо owner-pinned digest
  отклоняется;
- managed restart повторно проверяет exact executable bytes;
- external reconnect требует proof-of-possession и не принимает replay nonce;
- peer UID/PID не используются как единственное доказательство identity;
- suspend/revoke повышает epoch и блокирует новые RPC/NATS/storage/job
  operations;
- stale epoch result/observation отклоняется до canonical mutation;
- managed crash следует bounded restart policy;
- external disconnect не запускает process и делает capability unavailable;
- lifecycle mode не переключается без drain и нового epoch;
- private content, descriptor secrets и credentials отсутствуют в registry,
  approval surface, logs, errors, health и telemetry;
- Kernel restart восстанавливает registry/grants в `recovery_only` без
  PostgreSQL/PgBouncer/NATS/Vault через ADR-0216 Control Store.

## Последствия

Положительные:

- новый локальный module можно подключить без пересборки Kernel;
- authority остаётся у владельца и Kernel hard policy;
- managed и external runtime имеют честные разные lifecycle guarantees;
- capability revocation единообразно fences RPC, PostgreSQL, NATS, Vault,
  Blob и jobs;
- обязательная supply-chain infrastructure не блокирует clean-room start.

Стоимость:

- нужен durable private Kernel Registry, owner grant approval UI/API и audit
  изменений;
- каждый platform service должен поддерживать scoped issue/revoke/fencing;
- external module хранит собственный private registration key;
- controlled-host trust model не защищает от полного compromise user account;
- managed lifecycle дополнительно требует launch-integrity verifier ADR-0219.
