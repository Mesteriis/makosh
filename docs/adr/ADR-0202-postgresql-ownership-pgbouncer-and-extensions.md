# ADR-0202: PostgreSQL, изоляция данных и PgBouncer

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md).

Уточняется:

- [ADR-0224: Storage Control Plane, owner-scoped PostgreSQL и lifecycle migrations](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

## Контекст

Макошь использует одну физическую relational database для canonical state.
Отдельная database или schema на каждый домен создаёт ненужную локальную
эксплуатационную сложность. Одновременно отдельные module runtimes не должны
получать неограниченный доступ ко всем таблицам или исчерпывать PostgreSQL
connections.

Количество отдельных процессов делает прямые connection pools каждого модуля
опасными: сумма локальных pools способна превысить `max_connections` даже при
малой фактической нагрузке. Поэтому connection pooling и per-module quotas
являются частью первого `storage_control_v1`/durable-owner slice, а не
recovery-only Kernel и не последующей оптимизацией.

## Решение

### Основное хранилище

PostgreSQL является первым и обязательным storage driver clean-room backend.
Domain и application contracts не содержат `PgPool`, SQLx types, PostgreSQL
error codes или dialect-specific DTO.

PostgreSQL-специфичный код разрешён только в `makosh-storage-postgres` и
module-owned persistence adapters. Kernel, public storage protocol и SQL-free
Storage Control orchestration не содержат SQL client или schema introspection.

Используется одна physical database с фиксированными platform-created schemas
ADR-0224. Отдельная schema на модуль запрещена. Ownership обеспечивается
owner-prefixed objects, PostgreSQL roles/grants и executable storage registry;
RLS остаётся defense in depth, а не authority для cross-owner доступа.

### Владение таблицами

- Каждая business table имеет ровно один owner module.
- Owner фиксируется в admitted `StorageBundleV1`, immutable storage ledger и
  проверяется guard.
- Названия business tables имеют стабильный owner prefix.
- Domain runtime не читает и не изменяет таблицы другого модуля.
- Foreign keys разрешены внутри одного owner module.
- Cross-module foreign keys запрещены; ссылки между модулями представлены
  typed IDs, events и workflow state.
- Shared technical tables допустимы только для exact platform owners: events,
  storage, scheduler и audit. Generic `platform` owner запрещён.

Raw SQL вне owner persistence adapter запрещён.

### Role model

Минимальная role topology:

- privileged bootstrap/admin role — создаёт database, extensions и base roles;
- migration coordinator role — выполняет versioned migrations через owner
  roles;
- owner DDL role — stable `NOLOGIN`, владеет tables, sequences и functions
  одного durable owner;
- runtime principal — generation-scoped `LOGIN`, `NOINHERIT` для exact
  `registration_id + runtime_generation`, имеет только admitted DML grants;
- event relay role — работает только с event log, outbox и delivery metadata;
- read-only observability role — получает только безопасные statistics и
  health queries.

Runtime role не получает:

- `SUPERUSER`, `CREATEDB`, `CREATEROLE` или `BYPASSRLS`;
- DDL privileges;
- membership в чужом owner/runtime role;
- доступ к vault/session tables;
- право `SET ROLE` в privileged role;
- direct access к PostgreSQL catalog secrets.

`PUBLIC` не получает `CREATE` в production schemas и не получает доступ к
module tables.

### Shared technical tables

Shared outbox/inbox/event tables принадлежат exact owner `events`. Module
runtime не получает raw `SELECT`/`INSERT`/`UPDATE`/`DELETE` grants на них.
Вместо этого он получает `EXECUTE` только на admitted versioned
transaction-local functions, например
`makosh_platform.events_append_outbox_v1`. Function выводит caller из
аутентифицированного generation-scoped runtime principal, сверяет current
registration/runtime/storage/grant/role generations, проверяет owner, bounded
metadata, payload size и hash и не декодирует business payload.

Для таких tables применяется `FORCE ROW LEVEL SECURITY` как defense in depth,
но authority задают exact function contract, grants и caller mapping. Policy
не основывается на произвольном session variable, чтобы transaction pooling не
переносил module identity между клиентами.

Event relay имеет отдельную явно ограниченную role. Его расширенный доступ не
передаётся domain или integration runtime.

### Транзакции

- Модуль управляет только локальной транзакцией своих tables.
- Business mutation и outbox append выполняются атомарно.
- Cross-module database transaction запрещена.
- Cross-module consistency обеспечивается workflow, idempotent commands и
  compensating actions.
- Транзакции должны быть короткими и не включать remote provider calls,
  ожидание NATS или user input.
- Connection возвращается в pool сразу после commit/rollback.

Supervisor subsystem Kernel управляет только OS lifecycle PostgreSQL,
PgBouncer и `makosh-storage-runtime`. Storage Control проверяет cluster,
выдаёт typed attestation/readiness и управляет role provisioning, migration
ordering, connection budgets и observability. Kernel не выполняет SQL
introspection; ни Kernel, ни Storage Control не являются универсальным SQL
proxy и не содержат domain queries.

### Migrations

Каждый owner module поставляет immutable binary `StorageBundleV1` для
собственных objects. Exact digest pin-ится admitted distribution/managed
binding; running module не передаёт migration SQL. Storage Control:

1. получает global migration lock через direct administrative connection;
2. проверяет exact bundle digest, PostgreSQL AST, owner namespace и ownership
   conflicts;
3. применяет migration под соответствующей owner role;
4. фиксирует immutable step checksum, bundle digest и revision в canonical
   storage ledger;
5. выполняет privilege audit;
6. только после этого разрешает запуск runtime.

Module runtime и Kernel не выполняют migrations и не получают direct
PostgreSQL admin connection. Изменение extension set выполняет Storage Control,
а не module migration. V1 разрешает только transactional additive owner-local
changes и forward-only recovery по ADR-0224.

### PgBouncer

PgBouncer обязателен с первого `storage_control_v1` walking skeleton. В
`kernel_recovery_only_v1` он запрещён ADR-0225. После открытия storage gate все
normal runtime connections идут через PgBouncer в `transaction` pooling mode.

Direct PostgreSQL connections разрешены только для:

- bootstrap и migrations;
- backup/restore;
- controlled administration;
- операций, которым доказанно нужна session affinity и для которых нет
  runtime-path альтернативы.

Direct connection credential не выдаётся module runtime.

### Connection budgets

Connection limits задаются согласованно на трёх уровнях:

1. bounded client pool внутри module runtime;
2. per-user/per-database limits PgBouncer;
3. PostgreSQL `max_connections` с отдельным administrative reserve.

Обязательный инвариант конфигурации:

```text
sum(max module server connections)
+ event relay reserve
+ migration/admin reserve
<= PostgreSQL max_connections
```

Для каждого module role задаются конечные:

- client pool maximum;
- `max_user_connections`;
- `max_user_client_connections`;
- queue/wait timeout;
- statement timeout;
- lock timeout;
- idle-in-transaction timeout.

Отдельная PostgreSQL runtime role создаёт отдельный PgBouncer user/database
pool. Forced shared server user для module traffic запрещён, потому что он
смешивает connection budget и database identity разных модулей.

`max_db_connections`, `max_client_conn`, OS file-descriptor limit и
administrative reserve проверяются при bootstrap. Значения не копируются из
PgBouncer defaults без capacity calculation.

Исчерпание pool одного module role должно ставить в очередь или отклонять
только его requests. Unlimited connection retry запрещён.

### Ограничения transaction pooling

Runtime code не полагается на состояние PostgreSQL session между
транзакциями. В runtime path запрещены:

- session-level `SET`/`RESET`;
- `LISTEN`;
- session-level advisory locks;
- SQL-level `PREPARE`/`DEALLOCATE`;
- session-persistent temporary tables;
- cursor с `WITH HOLD`;
- смена role или `search_path` как mechanism изоляции.

Protocol-level prepared statements разрешаются только после contract test с
настроенным PgBouncer `max_prepared_statements`. При несовместимости driver
переключается на безопасный режим без statement cache; обход PgBouncer не
разрешён.

Migration coordinator и другие session-affine administrative operations
используют отдельный direct connection и не смешиваются с module runtime pool.

### Аутентификация и сеть

- PostgreSQL и PgBouncer слушают только разрешённый local interface или Unix
  socket.
- Runtime identities аутентифицируются отдельными credentials; одна shared
  application role запрещена.
- Предпочтительна SCRAM-SHA-256 authentication.
- Credentials не хранятся в repository, argv или logs.
- Storage Control выдаёт runtime typed `StorageBindingV1` с generation-scoped
  PgBouncer alias. Secret доставляется отдельно только через scoped Vault
  `CredentialLeaseV1` ADR-0223/ADR-0224; Kernel не получает plaintext.
- Health различает отказ PgBouncer, отказ PostgreSQL и исчерпание конкретного
  module pool.

### PostgreSQL extensions

В initial storage distribution включаются и проверяются:

- встроенный Full Text Search: `tsvector`, `tsquery`, GIN;
- `pg_trgm` для typo/fuzzy matching;
- `unaccent` для accent-insensitive normalization;
- `pg_stat_statements` для измерения query cost и pool pressure.

`pgvector` не является обязательным initial extension. Он может быть добавлен
отдельным решением после появления проверенного semantic-search scenario,
embedding ownership, privacy policy и benchmark.

Extension устанавливает только bootstrap role. Module persistence adapter
может использовать extension только если capability объявлена в storage
storage ownership declaration.

### Search ownership

Реализация этого derived module полностью заблокирована
[ADR-0208](ADR-0208-domain-development-allowlist-and-projection-freeze.md).
Следующая модель сохраняется только как отложенная ownership boundary и не
разрешает создавать package, schema, consumer или index до отдельной
разблокировки.

Global search является отдельным derived module:

```text
module events
    ↓
neutral SearchDocument projection
    ↓
PostgreSQL FTS / pg_trgm / optional vector index
```

Search не читает business tables напрямую и не является canonical truth.
Проекция должна быть rebuildable. Vault secrets, provider sessions, cookies,
tokens и raw blob bytes не индексируются.

### Backup и failure domain

Одна physical database остаётся общей failure domain. Process isolation не
маскирует отказ storage. Core переводит storage-dependent capabilities в
degraded state и не запускает бесконечные reconnect loops.

Backup включает PostgreSQL state и отдельно проверяет восстановление
role/grant model и storage ledger в disposable cluster. Restore повышает
storage generation и выполняет полный session fencing ADR-0224. NATS
JetStream не заменяет backup canonical database.

## Отклонённые варианты

### Отдельная database или schema на каждый модуль

Отклонено как обязательная topology: повышает локальную эксплуатационную
стоимость без необходимой пользы для текущего single-owner продукта.

### Одна shared runtime role

Отклонено, потому что любое SQL injection или defect получает доступ ко всем
module tables и один pool может исчерпать connections за всех.

### Прямое подключение каждого module pool к PostgreSQL

Отклонено: сумма независимых pools не ограничивает реальное количество server
connections и воспроизводит connection exhaustion.

### Ядро как generic SQL proxy

Отклонено, потому что ядро стало бы bottleneck и владельцем domain query
semantics.

## Последствия

Положительные:

- остаётся одна управляемая database;
- SQL ownership проверяется PostgreSQL, а не только code review;
- PgBouncer ограничивает server connection count;
- module defect не получает автоматический доступ к чужим tables;
- outbox сохраняет атомарность между state и NATS delivery;
- search расширяется внутри PostgreSQL без отдельного search cluster.

Отрицательные:

- roles, grants, RLS и PgBouncer configuration становятся production code;
- transaction pooling запрещает ряд session-based PostgreSQL features;
- migration coordinator должен управлять owner roles и privilege audit;
- одна database остаётся общей инфраструктурной failure domain.

## Проверка решения

Статический precondition уже закреплён в
[`backend/architecture/policy.json`](../../backend/architecture/policy.json): SQL client
dependency и standalone SQL разрешены только package surface `persistence`,
SQL identifiers обязаны иметь owner prefix, а cross-owner reads, writes и
foreign keys отклоняются architecture guard. Это не заменяет проверку реальных
PostgreSQL roles/grants и PgBouncer isolation ниже.

Integration environment с первого среза запускает через testcontainers:

- PostgreSQL;
- PgBouncer;
- NATS JetStream.

Обязательные tests:

- module role может CRUD только собственные tables;
- попытка читать или изменять чужую table отклоняется;
- runtime role не может DDL, `SET ROLE` или обойти RLS;
- shared outbox/inbox недоступен raw DML, а versioned technical function
  проверяет generation-scoped caller и сохраняет exact bytes;
- state mutation и outbox append атомарны;
- NATS outage не теряет committed outbox;
- PgBouncer transaction pooling переживает смену backend connection;
- session-dependent operation fails in test и запрещена guard;
- исчерпание pool одного module role не блокирует другой role;
- суммарный connection budget не превышает PostgreSQL limit;
- migration использует direct admin path и admitted `StorageBundleV1`, а
  runtime — только generation-scoped PgBouncer binding;
- revoke/role epoch/storage generation завершают old PgBouncer aliases и
  PostgreSQL sessions до выдачи нового binding;
- extensions присутствуют и доступны только разрешённым adapters;
- fresh database bootstrap воспроизводит roles, grants, RLS, extensions и
  schema без legacy migrations.

## Ссылки

- [PgBouncer pooling modes and feature compatibility](https://www.pgbouncer.org/features.html)
- [PgBouncer configuration](https://www.pgbouncer.org/config.html)
- [PostgreSQL privileges](https://www.postgresql.org/docs/current/ddl-priv.html)
- [PostgreSQL row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [PostgreSQL Full Text Search](https://www.postgresql.org/docs/current/functions-textsearch.html)
- [PostgreSQL pg_trgm](https://www.postgresql.org/docs/current/pgtrgm.html)
- [PostgreSQL unaccent](https://www.postgresql.org/docs/current/unaccent.html)
- [PostgreSQL pg_stat_statements](https://www.postgresql.org/docs/current/pgstatstatements.html)
