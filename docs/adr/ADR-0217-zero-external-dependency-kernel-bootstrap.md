# ADR-0217: Нулевой внешний bootstrap Kernel

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано; production Kernel, path resolver,
command-line contract и recovery endpoint ещё не созданы

Зависит от:

- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0206: Конституция Kernel и автомат запуска и восстановления](ADR-0206-kernel-constitution-boot-and-recovery-state-machine.md);
- [ADR-0212: Топология Cargo packages и изоляция пересборки модулей](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Конституция кода, ownership и автономность модулей](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0216: Private Kernel Control Store на SQLite](ADR-0216-private-kernel-control-store-with-sqlite.md).

Уточняется:

- [ADR-0218: Owner/device identity, enrollment и offline recovery](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0219: Целостность managed modules, distribution manifest и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0225: Первый recovery-only Kernel slice и фазовые ворота](ADR-0225-first-production-recovery-only-kernel-slice-and-phase-gates.md).

Этот ADR закрывает оставленную ADR-0216 boot-границу. Он определяет только
данные, необходимые до открытия Kernel Control Store. Общая модель settings,
их API, revisions, применение и hot reload определена ADR-0222 и намеренно не
становится частью pre-store bootstrap.

## Контекст

Kernel обязан запускать локальный recovery control plane до PostgreSQL,
PgBouncer, NATS, Vault и module runtimes. Исправная SQLite также не может быть
условием существования минимального recovery surface: именно через него
владелец должен диагностировать и явно восстановить Control Store.

Если до открытия Control Store Kernel требует layered configuration, secrets,
environment overlay или поиск прежней директории, boot перестаёт быть
детерминированным:

- разные источники могут выбрать разные Макошь instances;
- ошибочная конфигурация может молча открыть пустой store;
- Vault снова становится косвенной boot dependency;
- recovery path начинает зависеть от того же mutable state, который должен
  восстанавливать.

Для первой clean-room реализации нужен минимальный, проверяемый и одинаковый
на desktop и будущих headless installations bootstrap.

## Решение

### У Kernel нет обязательного конфигурационного файла

Нормальный Kernel запускается без `bootstrap.toml`, `.env` или другого
пользовательского configuration file. Отсутствие такого файла является штатным
состоянием, а не fallback.

До открытия Control Store допустимы только:

1. compiled immutable defaults и стабильная application identity;
2. OS-standard private per-user application-data location;
3. один явный non-secret command-line override `--data-dir <absolute-path>`;
4. immutable signed bundled distribution inventory, поставляемый вместе
   с Kernel.

Никакие PostgreSQL, PgBouncer, NATS, Vault, network endpoint или module runtime
не являются bootstrap source или prerequisite.

### Выбор data directory

В первой версии действует строгий порядок:

```text
explicit --data-dir
        ↓ если отсутствует
OS-standard per-user local application-data directory
```

Default path вычисляется через platform API, представленный Rust crate
[`directories`](https://codeberg.org/dirs/directories-rs), и его
`ProjectDirs::data_local_dir()`. Application identity является compiled
distribution constant и не задаётся пользователем.

`--data-dir`:

- является единственным Макошь-specific override первой версии;
- принимает только абсолютный local filesystem path;
- выбирает ровно один отдельный Макошь instance;
- не объединяется с default directory и другими stores;
- не является secret и может передаваться через argv;
- не вызывает migration или копирование данных автоматически.

Kernel не выполняет directory scanning, autodiscovery старых installations,
поиск store относительно current working directory или silent fallback на
default path. Если явно выбранная директория недоступна, небезопасна или
несовместима, Kernel сообщает sanitized bootstrap error и не открывает другой
store.

OS-standard variables, которые platform API использует для системного выбора
директорий, не являются Макошь settings. Собственные `MAKOSH_*` environment
overrides, dotenv-файлы и generic environment configuration providers в
bootstrap запрещены.

### Private filesystem boundary

Data directory и runtime directory являются разными понятиями:

- data directory хранит private durable technical data Kernel, включая SQLite
  Control Store и его recovery artifacts;
- runtime directory хранит lock, local IPC endpoints и ephemeral process
  metadata.

Platform adapter выбирает стандартную private runtime location, а если
операционная система её не предоставляет — создаёт короткую owner-private
runtime location с тем же instance key. Runtime directory не настраивается
отдельным bootstrap field.

Перед использованием Kernel обязан проверить:

- canonical ownership и отсутствие подмены symbolic link;
- local filesystem requirement;
- owner-only permissions: `0700` для directories и `0600` для files на
  POSIX либо эквивалентный private ACL на Windows;
- возможность безопасно удерживать single-instance lock;
- возможность защитить local recovery endpoint.

Cloud-synchronized и network filesystem не используются для Control Store.
Невозможность доказать безопасную private boundary не вызывает fallback в
temporary, current-working или соседнюю directory.

### Instance identity до и после Control Store

До открытия SQLite Kernel выводит ephemeral boot instance key из stable
application identity и canonical data-directory identity. Он используется
только для lock и безопасного именования local endpoint.

Durable random `instance_id` создаётся и хранится в Control Store после его
успешного открытия. Маленький non-secret installation anchor отличает pristine
instance от удалённой SQLite по ADR-0218. Ни identity, ни anchor не находятся в
bootstrap configuration, argv или environment. Смена `--data-dir` означает
выбор другого instance, а не переименование существующего.

### Recovery не зависит от исправной SQLite

Boot gates разделены:

```text
Kernel process + private path/lock + minimal local recovery
                            ↓
             trustworthy Kernel Control Store
                            ↓
         managed infrastructure и data plane
```

Если Control Store отсутствует, повреждён, имеет неподдерживаемую schema
version или не проходит integrity check:

- Kernel остаётся в restricted `recovery_only`;
- публичный Gateway, managed infrastructure и modules не запускаются;
- online доступны только sanitized status и Control Store validate/export;
- registration/grant mutations и topology actions запрещены;
- default state не создаётся поверх повреждённого store;
- другой store не выбирается автоматически.

`restore/reset` выполняются только offline при остановленном Kernel, explicit
`--data-dir`, exclusive lock и local owner confirmation по ADR-0218.

### Immutable distribution input не является settings

Bundled distribution inventory описывает поставленные executable, exact
digests и protocol compatibility. Он immutable для конкретной сборки,
проверяется по ADR-0219 и не предоставляет runtime approval. Enablement,
grants и desired lifecycle требуют trustworthy Control Store по
ADR-0215/ADR-0216.

Отсутствующий или недоверенный distribution inventory блокирует bundled
managed launches и `ready`, но не restricted local recovery и не саму
возможность создать недоверенную external `pending` registration. Kernel
ничего не скачивает и не ищет replacement inventory.

### Граница будущих settings

Этот ADR не принимает решений о:

- schema и публичном API Kernel settings;
- module/product settings;
- revisions, defaults reconciliation и hot reload;
- ownership modes, endpoints, listener policy и runtime budgets;
- UI настроек и синхронизации между desktop и Android.

Такие параметры не становятся pre-store bootstrap inputs. Если последующее
решение признает их Kernel-owned desired state, их durable truth должна
проходить через Kernel Control Store. До отдельного ADR запрещено добавлять
generic settings service или layered configuration framework.

### Rust dependencies

Для первого implementation slice:

- `directories` используется только для OS-standard path discovery;
- `clap` используется с минимальным feature set для typed parsing явного
  `--data-dir` и recovery/validation command surface;
- `config-rs`, Figment, dotenv/environment loaders и file watchers не
  используются в Kernel bootstrap;
- `serde + toml` не добавляются, пока обязательного bootstrap file нет.

Bootstrap parsing остаётся cohesive internal responsibility
`makosh-kernel`; отдельный Cargo package без второго потребителя не
создаётся. Exact versions, disabled/default features и полный direct dependency
profile первого slice зафиксированы ADR-0225 и executable policy.

## Failure semantics

| Failure | Результат |
|---|---|
| Default directory можно безопасно создать | продолжить bootstrap |
| Explicit `--data-dir` invalid/unavailable | fail closed без fallback |
| Single-instance lock уже удерживается | новый process не становится вторым Kernel |
| Local recovery endpoint нельзя защитить | `fatal`; recovery control plane недостоверен |
| SQLite отсутствует или недоверена | online status/validate/export; restore/reset только offline |
| PostgreSQL/NATS/Vault недоступны | Kernel остаётся жив; capability blocked |
| Distribution inventory недоверен | data plane blocked, restricted recovery доступен |

Sanitized error может содержать категорию и безопасный path identifier, но не
secrets, private content или raw environment dump.

## Отклонённые варианты

### Обязательный `bootstrap.toml`

Отклонено: для единственного допустимого override создаёт ещё один источник
truth, parser/migration lifecycle и отдельный failure mode.

### Layered configuration через config-rs или Figment

Отклонено для boot path: file/environment/profile merging делает выбор instance
неявным. Эти crates не запрещены во всём продукте, но не входят в Kernel
bootstrap.

### Environment variables как Макошь configuration

Отклонено: inherited environment трудно диагностировать, оно может отличаться
между Tauri, shell и OS watchdog и допускает скрытое переключение instance.

### Поиск существующего store

Отклонено: directory scanning и выбор «подходящей» базы способны открыть не тот
instance или создать split-brain.

### Хранить data directory внутри Vault

Отклонено: возвращает circular dependency, при которой Kernel не знает, где
находится recovery state, пока Vault не открыт.

## Проверка решения

Первый production slice обязан доказать:

- запуск без configuration file выбирает OS-standard data-local directory;
- `--data-dir` выбирает ровно один отдельный instance;
- Макошь-specific environment не меняет data directory;
- invalid explicit path не вызывает fallback;
- два process не получают один single-instance lock;
- missing/corrupt/incompatible SQLite оставляет restricted local recovery
  доступным;
- online untrusted-store recovery допускает только status/validate/export;
- restore/reset требуют stopped Kernel, explicit data directory, exclusive
  lock и confirmation;
- при недоверенном store не запускаются managed services, modules и data plane;
- recovery endpoint local-only и owner-private;
- bootstrap logs и errors не содержат environment dump, credentials или private
  content;
- path-resolution и recovery tests не требуют PostgreSQL, NATS или Vault.

До появления production code executable policy фиксирует обязательные
bootstrap invariants и negative self-tests самой policy.

## Последствия

Плюсы:

- Kernel действительно запускается без внешней инфраструктуры и Vault;
- отсутствует обязательный config-file lifecycle;
- выбор Макошь instance детерминирован и диагностируем;
- повреждение SQLite не уничтожает recovery surface;
- mutable settings не смешиваются с boot identity.

Стоимость:

- platform path/permission adapter требует отдельных macOS, Linux и Windows
  tests;
- custom data directory доступна только как явный process option;
- смена data directory требует restart и считается выбором другого instance;
- Settings Registry требует trustworthy Control Store и не может исправить
  ошибку выбора data directory до его открытия.
