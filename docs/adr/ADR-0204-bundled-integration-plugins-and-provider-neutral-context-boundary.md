# ADR-0204: Встроенные integration-плагины и нейтральная граница контекста

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md).

Связано с:

- [ADR-0203: Управление локальной инфраструктурой и восстановление](ADR-0203-managed-infrastructure-supervision-and-recovery.md);
- [ADR-0205: Core Gateway и транспорт клиентских приложений](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0215: Открытая регистрация модулей и capability grants](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219: Целостность managed modules и explicit updates](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0220: Канонический durable envelope и эволюция контрактов](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0221: ModuleDescriptorV1 и capability-level lifecycle](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0222: Kernel Settings Registry и supervised reconfiguration](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223: Encrypted SQLite Vault и scoped credential leases](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0236: Integration owners, protocol adapters и configuration instances](ADR-0236-integration-owners-protocol-adapters-and-configuration-instances.md).

## Контекст

Макошь одновременно решает две разные задачи:

1. даёт полноценный интерфейс для работы с конкретным каналом: Mail,
   Telegram, WhatsApp, Zulip и будущими интеграциями;
2. строит над каналами provider-neutral слой evidence, памяти, review и
   контекста.

Для пользовательского интерфейса различия каналов существенны. Mail имеет
папки, треды и черновики; Telegram — чаты, темы, реакции и membership
operations; Zulip — streams и topics. Попытка преждевременно свести их к одному
универсальному operational API либо теряет возможности provider, либо создаёт
неограниченный `metadata`-контейнер и скрытый provider switch.

Для контекстных доменов источник, наоборот, не должен определять business
semantics. Новое письмо и новое сообщение Telegram являются разными внешними
наблюдениями, но ни Tasks, ни Personas, ни Knowledge не должны импортировать
Mail/Telegram SDK, разбирать provider payload или выбирать поведение по имени
provider.

Reference frontend подтверждает необходимость отдельных channel surfaces и
общих presentation shells. Он не является архитектурным шаблоном и не
доказывает работоспособность provider runtime. На момент решения работающими
считаются только Mail, Telegram и Zulip; остальные integration-плагины должны
получить новое executable evidence.

Проверенное reference evidence на дату решения:

- `appLayoutAccountNavigation.ts` создаёт отдельные account routes для Mail,
  Telegram и WhatsApp;
- `CommunicationsWorkspaceView.vue` выбирает Mail или messenger experience;
- `useTelegramConversationRuntimeActions.ts` уже отделяет Telegram lifecycle и
  sync operations от neutral Communications queries;
- `useZulipCommunicationsSurface.ts` описывает streams, topics и direct
  messages, которые нельзя честно свести к Telegram DTO;
- общий `MessengerWorkspace.vue` импортирует Telegram types, что фиксируется
  как boundary debt и не переносится в clean-room архитектуру.

Макошь не планирует marketplace, store или установку произвольных сторонних
плагинов. Термин «плагин» нужен для явного ownership,
`ModuleDescriptorV1` и lifecycle, а не для remote distribution.

## Решение

### Значение integration-плагина

**Встроенный integration-плагин** — поставляемый вместе с Макошь модуль,
который владеет:

- внешним протоколом и provider SDK;
- authentication/session runtime и provider credentials lease;
- sync cursor, rate limits и provider command execution;
- provider-specific operational state и projection;
- operational-контрактом своего канала;
- преобразованием provider observation в нейтральный evidence-контракт;
- capability declarations `ModuleDescriptorV1` и frontend surface contract
  reference.

Credential material integration получает только через scoped
`CredentialLeaseV1` ADR-0223. Integration продолжает владеть cursors,
checkpoints и provider operational/session state: небольшой credential-bearing
session blob может храниться в Vault, а большая или high-churn session database
остаётся в private integration store и получает только
`SessionStoreKeyLease`. Hidden WhatsApp WebView хранит cookies/storage в своём
OS-managed per-account profile и не экспортирует их в generic Vault.

Integration-плагин не является business domain. Он не создаёт Tasks,
Personas, Projects, Documents, Decisions, Obligations или другую durable
business truth.

Backend runtime встроенного плагина является отдельным managed process по
ADR-0200. Перед каждым launch Kernel проверяет signed distribution entry и
exact executable bytes по ADR-0219. Frontend experience поставляется в
подписанном application bundle и не загружается как произвольный remote code.

### Два семейства публичных контрактов

Каждый communication integration-плагин предоставляет два независимых
семейства контрактов.

#### Provider-specific operational contract

Operational-контракт принадлежит конкретному плагину, например:

```text
mail.operational.v1
telegram.operational.v1
whatsapp.operational.v1
zulip.operational.v1
```

Он может честно выражать provider semantics: folders, streams, topics,
reactions, participants, delivery state, join/leave, drafts, threading и
другие реальные capabilities. Его потребителями могут быть только:

- frontend experience этого provider;
- внешний API adapter, обслуживающий этот experience;
- workflow, явно относящийся к техническому lifecycle integration и не
  создающий business truth;
- сам integration runtime.

Другой provider и business/context domain не могут зависеть от этого
контракта.

Operational state является локальным представлением внешней системы и может
быть durable, но не является canonical business truth Макошь. Его schema,
tables, cursors и projections принадлежат integration-плагину и защищены
storage grants владельца.

#### Provider-neutral evidence contract

При пересечении границы context/memory provider data преобразуется самим
integration-плагином в нейтральный versioned contract, например:

```text
communications.evidence.v1
```

Контракт передаёт семантику наблюдаемого communication evidence, а не SDK DTO
или универсализированную копию всех provider fields. Общие message/source,
observation identity, recorded/observed time и causation/correlation находятся
в `DurableEnvelopeV1.observation` ADR-0220. Evidence payload содержит
owner-specific provenance, external occurrence time и opaque references на
private bodies или media без дублирования outer delivery metadata.

Raw provider cursor не пересекает outer boundary: integration хранит его в
собственном protected state, а envelope при доказанной необходимости переносит
только bounded SHA-256 cursor digest. Provider/session secrets не используются
для partition key, subject, trace или diagnostics.

Provider identity сохраняется в provenance для трассировки и доказательства
происхождения. Она не используется context domain для выбора business
поведения. Domain может хранить `EvidenceRef`, `SourceRef` и `BlobRef`, но не
может разбирать provider payload или branch-иться по `module_id`/provider name.

Антикоррупционный mapper `provider protocol → neutral evidence` принадлежит
integration-плагину. Core только проверяет identity, capability, contract
version и envelope и не переводит provider payload в business semantics.

### Ownership

| Ответственность | Владелец |
|---|---|
| Внешний protocol, auth/session, cursor, rate limit | Integration-плагин |
| Provider-specific commands и operational projection | Integration-плагин |
| Provider-specific экран и его application controller | Frontend experience плагина |
| Переиспользуемые визуальные компоненты | Shared presentation layer |
| Neutral evidence, provenance и canonical persistence | Evidence/Communications owner |
| Promotion evidence в business truth | Review/workflow и target domain |
| Routing, auth, capabilities и lifecycle | Core runtime |

Shared presentation component может отображать список сообщений, composer или
attachment preview, но не импортирует provider SDK/generated types и не
определяет provider behavior. Provider frontend adapter преобразует свой
operational DTO в узкие presentation props и обрабатывает provider-specific
actions.

Переиспользование `MessengerWorkspace`, `MailWorkspace` или похожей оболочки не
требует одного backend operational contract для всех каналов.

### Frontend surfaces и registry

Отдельный экран Mail, Telegram, WhatsApp, Zulip или другого bundled provider
разрешён и является **provider experience surface**, а не product domain.
Экран может показывать provider brand, capabilities и provider-specific
operations.

Frontend код доступных experiences включается в desktop или Android application
bundle на этапе сборки. Core registry активирует surface только когда
одновременно существуют:

- valid signed bundled distribution entry с совпадающими `module_id`, platform,
  protocol range и exact executable, `ModuleDescriptorV1` и optional settings
  schema artifact digests;
- compatible exact validated `ModuleDescriptorV1`;
- bundled frontend surface handler с совпадающими contract versions;
- выданные пользователем capabilities для конкретного configuration instance;
- current resolved settings revision, если capability зависит от settings.

Android client может реализовать не все desktop surfaces, но использует те же
versioned operational/context contracts. Наличие или отсутствие Android UI не
меняет ownership integration/domain и не создаёт mobile-specific business API.

Произвольный URL, JavaScript bundle, dynamic library или package не может
добавить исполняемый frontend/backend код во время работы Макошь.

`ModuleDescriptorV1` integration-плагина объявляет минимум:

- `module_id` и runtime protocol version;
- operational contract names/versions;
- neutral evidence contract names/versions;
- frontend `surface_id` и требуемые capabilities;
- account model и поддерживаемые lifecycle operations;
- typed storage, durable contract, `VaultPurposeRequestV1` ADR-0223 и blob
  capability requests;
- `SettingsSchemaRefV1`, если plugin имеет configuration ADR-0222.

Навигация и доступность screen должны строиться из проверенного registry, а не
из несогласованных provider whitelist в нескольких слоях приложения.

Query-cache ownership разделяется так:

```text
['integrations', '<provider>', 'operational', ...]
['integrations', '<provider>', 'runtime', ...]
['communications', 'evidence', ...]
['personas' | 'tasks' | 'knowledge' | ..., ...]
```

Provider-specific operational state не маскируется под neutral business cache.
Canonical evidence и context state не получают provider-root keys.

### Inbound flow

Один provider observation порождает два согласованных, но разных представления:

```text
External provider
        ↓
Integration runtime
        ├─→ provider operational projection
        │       ↓
        │   provider experience screen
        │
        └─→ neutral evidence observation в outbox
                ↓
            NATS JetStream
                ↓
            Evidence/Communications owner
                ↓
            Review / context workflows / domains
```

Operational projection не заменяет canonical evidence. Evidence persistence не
даёт context domain права читать operational tables плагина.

Если фиксация provider state и evidence observation должна быть атомарной, они
записываются в одной transaction владельца плагина вместе с outbox. Canonical
persistence у evidence owner остаётся отдельной idempotent transaction с inbox
deduplication.

### Outbound flow

Provider experience отправляет typed provider-specific request или command:

```text
Provider experience
        ↓
generated operational client
        ↓
Core capability router / durable command route
        ↓
Integration runtime
        ↓
External provider
        ↓
operational result + neutral evidence when applicable
```

Business/context domain не отправляет provider-specific command напрямую. Если
business workflow должен инициировать внешнее действие, он формирует
provider-neutral intent через отдельный application contract. Явный routing
workflow выбирает разрешённый channel/account и создаёт command конкретного
integration-плагина с сохранением causation, correlation и evidence. Выбор не
скрывается внутри domain.

### Failure isolation

- Ошибка integration runtime делает degraded только его operational surface и
  capabilities.
- Другие provider screens и domain runtimes продолжают работу.
- Уже сохранённые canonical evidence и context остаются доступны при отказе
  provider runtime.
- Временный отказ evidence/context owner может скрыть context overlay, но не
  обязан останавливать локальную operational projection плагина.
- Core не переключает provider, implementation или topology автоматически.
- Очереди, cursors, provider sessions и evidence не очищаются при restart.

## Запрещено

- marketplace, plugin store или runtime download/install произвольного кода;
- remote frontend bundles и provider code из непроверенного
  `DistributionManifestV1`;
- один универсальный operational message DTO для всех providers;
- provider enum/switch, SDK types или provider payload в business domain;
- `metadata: map/json` как escape hatch neutral domain contract;
- domain subscription на provider operational subjects;
- shared UI component, импортирующий provider generated types;
- monolithic Communications implementation, владеющий provider protocol,
  session, cursors и provider-specific tables всех каналов;
- преобразование provider payload в business semantics внутри Core;
- direct read/write operational tables другого integration-плагина.

## Отклонённые варианты

### Один provider-neutral API до самого frontend

Отклонено: он либо теряет уникальные capabilities Mail/Telegram/Zulip, либо
становится неограниченным union с provider switches и opaque metadata.

### Каждый provider как business domain

Отклонено: канал доставки не владеет Personas, Tasks, Knowledge или памятью и
не должен становиться отдельной предметной моделью Макошь.

### Один Communications-модуль со всеми provider implementations

Отклонено: объединяет независимые failure domains, storage ownership и причины
изменения и мешает независимому restart.

### Plugin marketplace и динамическая установка

Отклонено: продукт не требует store, а такая модель добавляет supply-chain,
sandboxing, signing, compatibility и remote-code risks без текущей ценности.

## Последствия

Положительные:

- frontend сохраняет полноценные отдельные channel experiences;
- provider особенности не протекают в память и business domains;
- новый bundled provider добавляется через signed distribution entry, runtime
  descriptor, contracts и surface, а не через изменение каждого domain;
- shared UI остаётся переиспользуемым без provider coupling;
- сбой одного integration-плагина не останавливает остальные каналы;
- provenance сохраняет источник без превращения provider в business semantic.

Отрицательные:

- у communication integration появляются два публичных contract families;
- требуется явный mapper operational observation → neutral evidence;
- frontend registry и generated clients должны проверять совместимость версий;
- одинаковые presentation patterns не устраняют provider-specific DTO и tests;
- необходимо отдельно тестировать operational projection и evidence delivery.

## Проверка решения

До признания реализации завершённой должны существовать executable checks:

- package guard запрещает domain dependency на integration implementation,
  provider operational contract и provider SDK;
- integration может зависеть только от собственного operational contract,
  neutral evidence contracts и разрешённых platform contracts;
- frontend provider experience не импортирует другой provider experience;
- shared presentation packages не импортируют provider generated types;
- negative source/dependency check не находит provider enum, SDK DTO или
  provider payload в domain contracts;
- contract tests проверяют deterministic mapping в neutral evidence, stable
  observation IDs, provenance и duplicate delivery;
- registry отказывает при неизвестном module/surface/contract version;
- invalid signature, size или digest bundled integration entry блокирует runtime
  launch и activation surface без выбора другого executable;
- crash integration runtime сохраняет работоспособность других screens и
  доступность уже сохранённого evidence;
- private bodies, media и secrets отсутствуют в NATS, logs, errors и health;
- Kernel и production runtime не имеют пути загрузки или установки
  remote/plugin-store code и не выполняют automatic rollback.
