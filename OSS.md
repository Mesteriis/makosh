# Rust-ландшафт вокруг Макошь

Собрал **50 публичных проектов**, которые пересекаются с Макошь на уровне продукта, доменов, движков или интеграций. Срез актуален на **16 июля 2026 года**.

Это не буквальный дамп всех репозиториев GitHub. По запросам вроде `Rust contact manager` выдача быстро превращается в учебные проекты размером от нуля до нескольких десятков килобайт. То есть адресная книга с надеждами, а не `RelaSystem`.

Я оценивал совпадение не по наличию кнопки «создать задачу», а по твоей архитектуре:

- интеграции являются внешними адаптерами, а не продуктовыми доменами;
- Email, Telegram и WhatsApp сходятся в `Communications`;
- `Radar` ловит сигнал до того, как он станет задачей, контактом, документом или проектом;
- память, временная линия, поиск, доверие и внимание оформляются как отдельные движки;
- междоменные изменения проходят через события и workflows.

## Главный вывод

**Полного Rust-аналога Макошь я не нашёл.**

Ближайшее распределение выглядит так:

| Часть Макошь                | Самые близкие проекты                                   |
| --------------------------- | ------------------------------------------------------- |
| Продукт целиком             | `Macro`                                                 |
| Персональный AI runtime     | `IronClaw`, `Moltis`                                    |
| Memory Engine               | `TinyCortex`                                            |
| Radar / Attention           | `QCue`, `Atomic`, `Minne`                               |
| Timeline / наблюдение       | `screenpipe`, `ActivityWatch`                           |
| Communications / Mail       | `Stalwart`, `Bichon`, `Himalaya`, `Chatmail Core`       |
| Knowledge Graph             | `CozoDB`, `Raphtory`, `HelixDB`, `TerminusDB`           |
| Search                      | `Tantivy`, `Qdrant`, `LanceDB`, `Meilisearch`, `Trieve` |
| AI orchestration            | `Swiftide`, `Rig`                                       |
| AI evidence и наблюдаемость | `TensorZero`                                            |
| Durable workflows           | `Restate`                                               |
| Local-first sync            | `Loro`, `Automerge`, `Iroh`                             |

---

# 1. Самые близкие к Макошь продукты

## 1. `macro-inc/macro`

**Самый близкий продуктовый конкурент.**

Macro объединяет email, сообщения, документы, файлы, задачи, звонки, агентов, GitHub и CRM в одном workspace. Объекты связаны между собой, участвуют в едином поиске и используют общую память. Это почти буквальное отражение верхнего слоя Макошь, только Macro ориентирован прежде всего на команды, а Макошь у тебя personal-first.

Особенно полезно изучить:

- unified inbox;
- связь писем, каналов, файлов, задач и контактов;
- представление одного объекта в разных контекстах;
- общий поиск;
- CRM-проекции поверх коммуникаций;
- объединённую навигацию без отдельных «почтового клиента», «таск-трекера» и «файлового менеджера».

Проект очень активен: 15 июля 2026 года в него одновременно входили изменения email, поиска, звонков и CRM.

**Вердикт:** главный конкурентный и UX-референс. Не стоит использовать как основу Макошь: командная модель и AGPL потянут продукт в другую сторону.

---

## 2. `nearai/ironclaw`

**Ближайший аналог защищённого персонального AI-ассистента.**

IronClaw предлагает локальную память, несколько каналов, web gateway, фоновые routines, cron и event triggers, heartbeat, MCP, динамические инструменты, параллельные задания и sandbox. Особое внимание уделяется WASM-изоляции, защите секретов, prompt injection и сетевым allowlist.

Совпадает с Макошь по направлениям:

```text
AI Agents
Memory
Communications integrations
Background workflows
Security
Secrets isolation
Persistent context
Local-first execution
```

Но IronClaw остаётся **ассистентом с инструментами**, а не системой доменов `Contacts`, `Organizations`, `Documents`, `Tasks`, `Calendar` и `Obligations`.

**Вердикт:** обязательный референс для `ai/`, tool security, каналов и agent runtime.

---

## 3. `moltis-org/moltis`

**Персональный AI gateway в одном Rust-бинарнике.**

Moltis поддерживает несколько LLM-провайдеров, Telegram, web gateway, SQLite-сессии, долговременную память, knowledge base, subagents, skills, lifecycle hooks, cron-задачи, web browsing, голос, OAuth, MCP, OpenTelemetry и sandboxed execution.

Особенно интересны:

- trait-based provider architecture;
- сериализация запусков внутри сессии;
- очереди сообщений во время работы агента;
- hooks с приоритетами и circuit breaker;
- разделение gateway, agent runner, tools и providers;
- локальная поставка одним бинарником.

**Вердикт:** хороший донор архитектурных решений для `ai/runtime`, `integrations`, `skills`, `hooks` и локальной поставки Макошь.

---

## 4. `tinyhumansai/tinycortex`

**Самый близкий кандидат к Макошь Memory Engine.**

TinyCortex фильтрует шум при ingestion, хранит источник истины локально, строит производные SQLite, vector, graph и summary-tree индексы, учитывает provenance, recency и влияние взаимодействий пользователя. Поиск объединяет ключевые слова, векторы и граф.

Совпадает с твоей философией:

```text
Raw evidence
↓
Normalized memory
↓
Derived indexes
↓
Contextual retrieval
↓
Agent context
```

Это заметно ближе к Макошь, чем типичная схема «запихнуть весь текст в vector DB и надеяться, что cosine similarity разберётся с жизнью человека».

**Вердикт:** первый проект для глубокого архитектурного teardown `engines/memory`.

---

## 5. `kenforthewin/atomic`

**Knowledge Intelligence + Radar + Watchtower.**

Atomic хранит Markdown-атомы, строит семантический граф, создаёт wiki с inline citations, выполняет периодические исследования, отчёты, contradiction scans и отслеживает открытые вопросы. Есть RSS, browser extension, MCP, RAG и desktop/server-клиенты.

Очень близко к:

```text
Radar
Knowledge Notes
Evidence
Research Agents
Organization Watchtower
Open Questions
Contradiction Detection
```

**Вердикт:** один из лучших референсов для связки `Radar → Knowledge → Watchtower`.

---

## 6. `SparkyWen/qcue`

**Capture-first second brain с approval gate.**

QCue ведёт append-only поток текста и голоса, поддерживает raw sources как отдельный источник истины, а LLM использует для поддержания производной Markdown-wiki. Есть agentic recall, ночная консолидация и подтверждение изменений пользователем.

Особенно ценно совпадение с правилами Макошь AI:

```text
Source
Confidence
Evidence
Human approval
```

AI не переписывает память незаметно, а предлагает производные изменения. Удивительно разумная концепция для индустрии, которая обычно даёт модели доступ к базе и затем изображает удивление.

**Вердикт:** лучший референс для Review/Promotion и безопасной AI-консолидации памяти.

---

## 7. `perstarkse/minne`

**Графовая PKM-система с контролируемым извлечением сущностей.**

Minne принимает текст, URL, PDF, аудио и изображения, строит hybrid search, graph explorer и scratchpad. AI может предлагать сущности и отношения, но пользователь подтверждает их. Backend написан на Rust/Axum, данные хранятся в SurrealDB.

Близко к:

```text
Radar ingestion
Document Intelligence
Entity candidates
Relationship candidates
Knowledge Graph
Human Review
```

**Вердикт:** полезный референс для `documents → candidates → review → graph`.

---

## 8. `appaquet/exomind-v2`

**Исторически самый близкий Personal OS.**

Exomind объединял письма, заметки, задачи и закладки в одном персональном knowledge/inbox-пространстве, имел Gmail-интеграцию, browser extension, desktop и mobile clients.

Проект больше не подходит как техническая основа, но его объектная модель и продуктовая концепция по-прежнему полезны.

**Вердикт:** архитектурная археология. Не зависимость и не fork base.

---

## 9. `screenpipe/screenpipe`

**Внешний сенсор для Timeline и Radar.**

Screenpipe локально фиксирует accessibility tree, OCR, аудио, распознанную речь, говорящих, переключения приложений, клавиатурный ввод и браузерный контекст. Поверх этого строятся поиск, timeline и scheduled agents: meeting summary, day recap, standup, blockers и time breakdown.

Для Макошь это потенциальный источник событий:

```text
screen.activity.observed
meeting.conversation.observed
application.context.changed
work.session.detected
unfinished_work.detected
```

**Вердикт:** сильнейший источник для `Timeline`, meeting outcomes, focus intelligence и Radar. Но лицензия теперь source-available и требует коммерческого соглашения для коммерческого использования.

---

## 10. `AppFlowy-IO/AppFlowy`

**Документы, базы, проекты и AI workspace.**

AppFlowy является open-source альтернативой Notion: документы, Kanban, базы, проекты, AI и self-hosting. UI написан на Flutter, существенная часть ядра и синхронизации использует Rust.

**Вердикт:** референс для Documents/Notes/Projects и редакторского UX, но не для Memory-first архитектуры.

---

# 2. Memory, Knowledge, Documents и Timeline

| Проект                           | Что полезно для Макошь                                                                                                                                                                                 |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `iwe-org/iwe`                    | Markdown как источник истины, knowledge graph, backlinks, LSP, CLI и MCP. Особенно подходит для `Notes`, `Knowledge` и agent context без отдельного облачного хранилища.                               |
| `kuku-mom/kuku`                  | Локальный Markdown workspace, AI diffs с подтверждением, шифрованная синхронизация, decision documents и история изменений. Rust используется в AI/indexing-ядре.                                      |
| `sysid/bkmr`                     | Сбор URL, snippets, Markdown и файлов, full-text/semantic/hybrid search, локальные embeddings, LSP и agent memory. Хороший референс для browser-to-Radar ingestion.                                    |
| `illegal-instruction-co/rememex` | Локальный semantic file search, более 120 форматов, OCR, EXIF, геоданные, hybrid retrieval, reranking, annotations и MCP. Близко к `Document Intelligence`.                                            |
| `hang-in/seCall`                 | Собирает сессии Claude Code, Codex, Gemini и ChatGPT, превращает их в Markdown, поддерживает BM25+vector search, MCP, graph и git sync. Полезно для AI-session timeline.                               |
| `ActivityWatch/activitywatch`    | Локально фиксирует активное приложение, окно, браузерную вкладку, URL и AFK-состояние. Архитектура разделяет watchers, server, web UI и sync. Более лёгкий и открытый сенсор Timeline, чем screenpipe. |
| `exomind-team/exomind`           | Новый pre-1.0 personal assistant с local-first/event-driven подходом, event logging, time blocks и sync. Rust присутствует в runtime, но продукт в целом смешанный.                                    |

---

# 3. Communications и Mail

## Почтовая инфраструктура

| Проект                  | Совпадение                                                                                                                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `stalwartlabs/stalwart` | Полноценный Rust mail/collaboration server: JMAP, IMAP, SMTP, CalDAV, CardDAV, WebDAV, spam/phishing analysis, DKIM, DMARC, SPF, ARC, S/MIME/OpenPGP, FTS, webhooks и автоматизации. Главный инфраструктурный референс Mail. |
| `rustmailer/bichon`     | Архиватор писем: несколько IMAP-аккаунтов, incremental sync, общий FTS, threads, attachment search, tags, извлечённые контакты, deduplication и dashboard analytics. Почти готовая модель `Archive Intelligence`.            |
| `pimalaya/himalaya`     | Rust CLI/API для IMAP, SMTP, JMAP, Gmail REST, Microsoft Graph, Maildir и нескольких аккаунтов. Хорошая модель provider-neutral mail adapter.                                                                                |
| `meli/meli`             | Почтовый клиент с IMAP, Maildir, notmuch, mbox, JMAP, threading, SQLite search, vCard и GPG sign/encrypt/verify. Полезен для crypto UX и работы с threads.                                                                   |
| `chatmail/core`         | Rust-ядро Delta Chat: превращает email в encrypted instant messaging, поддерживает IMAP/SMTP, MIME, Autocrypt, SecureJoin, контакты и multi-device networking. Хороший референс универсальной модели Communication.          |

## Каналы сообщений

| Проект                       | Совпадение                                                                                                                                                                                                     |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `matrix-org/matrix-rust-sdk` | Production-ready SDK для Matrix-клиентов, encryption, synchronization и room state. Подходит как изолированный Matrix integration adapter.                                                                     |
| `oxidezap/whatsapp-rust`     | Неофициальный WhatsApp Web client: QR/pairing, persistent sessions, Signal Protocol, media, groups, reactions, receipts, contacts, archive, pin и mute. Архитектурно очень близок к твоей WhatsApp-интеграции. |
| `AmanoTeam/ferogram`         | Эргономичный Rust-слой над активным продолжением `grammers` для Telegram-клиентов. Подходит для provider adapter, но не для продуктового домена Telegram.                                                      |
| `whisperfish/presage`        | Rust-основа для клиента Signal. Полезна как изолированный integration foundation.                                                                                                                              |

`whatsapp-rust` активно обновляется, включая изменения от 15 июля 2026 года, но README прямо предупреждает о неофициальности клиента и возможном конфликте с условиями Meta.

Правильная роль всех этих проектов в Макошь:

```text
Provider SDK
↓
integrations/<provider>
↓
integration.<provider>.*.observed
↓
Communications
```

Не:

```text
Provider SDK
↓
отдельный продуктовый домен WhatsApp
```

Иначе через год приложение будет состоять из пяти почтовых ящиков в плаще.

---

# 4. Calendar и Tasks

| Проект                              | Что можно взять                                                                                                                                                        |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `lennart-k/rustical`                | CalDAV/CardDAV server на Rust, SQLite, recoverable calendars, OIDC, sharing и WebDAV Push. Подходит для локального calendar/contact provider.                          |
| `pimalaya/calendula`                | Provider-neutral Rust API/CLI для CalDAV и vdir, несколько аккаунтов, discovery и JSON output. Хорошая основа `integrations/calendar`.                                 |
| `GothenburgBitFactory/taskchampion` | Rust task storage/synchronization engine, используемый Taskwarrior. Полезен для task state, replica model и offline synchronization, но не содержит Task Intelligence. |

Зрелого Rust-проекта, который реализует одновременно:

```text
Waiting Intelligence
Blocking Intelligence
Readiness Score
Task Context Pack
Obligations
Focus Planning
```

я не нашёл. Здесь Макошь заметно выходит за границы обычных task managers.

---

# 5. Knowledge Graph, история и поиск

## Граф и система памяти

| Проект                  | Наиболее подходящая роль                                                                                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cozodb/cozo`           | Embedded graph/relational database с Datalog, рекурсивными запросами, time travel, HNSW, FTS и graph algorithms. Лучший кандидат для локального reasoning-oriented Knowledge Graph.                        |
| `Pometry/Raphtory`      | Temporal graph database: time travel, multilayer graph, analytics, scoring, risk detection и GraphQL. Очень близко к Relationship Brain, Trust/Health history и Watchtower.                                |
| `HelixDB/helix-db`      | Молодая graph-vector database на Rust для knowledge graphs и AI memory; объединяет graph, vector, KV, documents и relational data. Перспективный watchlist.                                                |
| `surrealdb/surrealdb`   | Multi-model Rust database: document, graph, relational, vector, time-series и realtime events. Очень широкая платформа, но может стать вторым продуктом внутри первого.                                    |
| `terminusdb/terminusdb` | Version-controlled document and knowledge graph, commits для каждого изменения, diff, clone, push/pull и time-travel. Особенно интересен для Memory First и истории сущностей. Смешанный Rust/Prolog стек. |
| `oxigraph/oxigraph`     | RDF/SPARQL graph database и toolkit на Rust. Лучший вариант, если Макошь потребуется JSON-LD/RDF, онтологии и внешняя semantic-web совместимость.                                                          |

### Моя архитектурная оценка

- **CozoDB**: лучший кандидат для embedded reasoning и графовых запросов.
- **Raphtory**: лучший кандидат для временной аналитики отношений.
- **HelixDB**: следить и экспериментировать.
- **TerminusDB**: изучить модель versioned truth.
- **Oxigraph**: брать только при реальной необходимости RDF/SPARQL.
- **SurrealDB**: не делать хранилищем всего лишь потому, что оно умеет всё. Так обычно рождаются базы, которые однажды начинают владеть продуктовой архитектурой.

## Полнотекстовый и семантический поиск

| Проект                    | Роль                                                                                                                                                                                            |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `quickwit-oss/tantivy`    | Embedded Lucene-подобный Rust FTS: BM25, phrase/range queries, facets, tokenizers и incremental indexing. Лучший базовый кандидат для локального Search Engine.                                 |
| `qdrant/qdrant`           | Dense, sparse и multivector search, metadata filters, hybrid fusion, RRF, quantization и embedded Qdrant Edge. Подходит для отдельного semantic index.                                          |
| `lancedb/lancedb`         | Локальный multimodal vector store с FTS, SQL, versioning и поддержкой текста, изображений и видео. Интересен для attachment/document intelligence.                                              |
| `meilisearch/meilisearch` | Search-as-you-type, typo tolerance, facets, hybrid search, multilingual support, personalization, conversational search и document relations. Хорош для пользовательского global search.        |
| `devflowinc/trieve`       | Готовая платформа search, recommendations и RAG: dense+sparse hybrid, reranking, recency bias, grouping, highlighting и self-hosting. Полезна как reference implementation поискового продукта. |

Для personal-first Макошь разумная стартовая конфигурация выглядит так:

```text
Tantivy
+
FastEmbed
+
domain-owned PostgreSQL/SQLite data
```

А отдельные Qdrant, LanceDB или Trieve стоит добавлять только после появления измеримой нагрузки или требований, которые embedded-поиск действительно не закрывает. Человек способен добавить пять баз данных за день. Удаляет он их потом несколько лет.

---

# 6. Workflows, события и local-first sync

| Проект                   | Совпадение                                                                                                                                                                                                      |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `restatedev/restate`     | Durable execution, workflows-as-code, event processing, stateful actors, exactly-once communication, retries, durable timers и observability. Самый близкий инфраструктурный аналог для Макошь workflows/sagas. |
| `windmill-labs/windmill` | Rust backend + Svelte 5; APIs, jobs, workflows, schedules, webhooks, Kafka, WebSockets, email triggers и workflow UI. Полезен для пользовательских автоматизаций, но слишком platform-oriented для ядра Макошь. |
| `automerge/automerge`    | Rust CRDT и sync protocol для local-first приложений. Подходит для синхронизации документов и заметок между устройствами.                                                                                       |
| `loro-dev/loro`          | CRDT для текста, rich text, деревьев, списков и maps; P2P sync, version control и быстрый time travel. Вероятно, лучше Automerge для сложных редактируемых документов.                                          |
| `n0-computer/iroh`       | Public-key addressed P2P networking через QUIC, hole punching, relay fallback, blobs, gossip и eventually consistent docs. Полезен для encrypted device sync и вложений.                                        |

Для Макошь я бы не ставил Restate в центр сразу. Сначала:

```text
Event Store
Outbox
Idempotent Consumers
Correlation / Causation
DLQ
```

И только когда появятся реально долгие, восстанавливаемые cross-domain процессы, подключал бы durable runtime. Твоя модель событий уже правильно отделяет ownership доменов от orchestration.

---

# 7. AI Agents, inference и доказательность

| Проект                    | Роль в Макошь                                                                                                                                                                          |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0xPlaygrounds/rig`       | Rust abstraction для LLM providers, agents, tools, streaming, embeddings, vector stores и memory policies. Подходит как лёгкий provider/agent layer.                                   |
| `bosun-ai/swiftide`       | Agent harness, typed task graphs, pause/resume, human approval, MCP, lifecycle hooks, RAG pipelines и tracing. Очень близко к контролируемым AI workflows Макошь.                      |
| `tensorzero/tensorzero`   | LLM gateway, observability, feedback, datasets, replay, evaluation, optimization, routing, retries и A/B testing. Лучший вариант для проверки и аудита AI-результатов.                 |
| `Anush008/fastembed-rs`   | Локальные text, sparse и image embeddings, multilingual models и rerankers через Rust/ONNX. Хорошо подходит для local-first Search и Memory.                                           |
| `EricLBuehler/mistral.rs` | Локальный Rust inference server: OpenAI/Anthropic-compatible API, multimodal models, embeddings, quantization и agentic runtime. Альтернатива Ollama для более тесной Rust-интеграции. |

### Предпочтительная комбинация

```text
Rig или Swiftide
    ↓
Макошь AI contracts
    ↓
TensorZero observability/evals
    ↓
Cloud models / Ollama / mistral.rs
    ↓
FastEmbed local embeddings
```

`Swiftide` ближе к твоей модели typed workflows и human approval. `Rig` проще как provider abstraction. TensorZero закрывает именно тот пробел, который обычно игнорируют: **почему AI выдал этот результат, на каких данных, насколько хорошо он работает и что изменилось после смены модели**.

---

# Где Макошь действительно уникален

## 1. Contact Brain и Relationship Intelligence

Я не нашёл зрелой Rust-системы, которая объединяет:

```text
Identity Resolution
Communication DNA
Relationship Timeline
Trust
Health
Skills
Memory Cards
Dossier
Contact Enrichment
```

Macro имеет CRM и contact views, Bichon извлекает адреса из почты, Rustical поддерживает CardDAV, но это всё ещё не Relationship Intelligence.

Это один из главных незанятых участков Макошь.

## 2. Organization Intelligence

Не найден зрелый Rust-аналог для связки:

```text
Organizations
Contracts
Invoices
Domains
Portals
Procedures
Playbooks
VAT/VIES
Passive OSINT
Watchtower
Dossier
```

Atomic частично закрывает research/watchtower, Macro частично CRM, графовые базы дают инфраструктуру. Но самого домена нет.

## 3. Radar как универсальный инкубатор сущностей

QCue, Atomic, TinyCortex и Minne близки по духу, но ни один из них явно не реализует общий путь:

```text
Signal
↓
Review
↓
Promotion
↓
Task / Contact / Organization / Project / Document / Note
```

Это сильная продуктовая дифференциация Макошь.

## 4. Междоменный Context Pack

Почти никто не собирает единый пакет:

```text
Person
Organization
Messages
Documents
Meetings
Tasks
Obligations
Decisions
Timeline
Risks
Next actions
```

для подготовки к встрече, ответа на письмо или решения задачи. Macro подходит ближе остальных, но его модель менее явно memory/domain-driven.

## 5. Obligations, Waiting и Readiness

Task managers хранят статус. Макошь пытается понять:

```text
Кто кому что обещал?
Чего мы ждём?
Что блокирует действие?
Каких данных не хватает?
Каков риск?
Готова ли задача к выполнению?
```

Это отдельный слой intelligence, и прямых аналогов я не нашёл.

---

# Репозитории, которые стоит изучить первыми

## Уровень A: обязательный teardown

1. `macro-inc/macro` — продуктовая композиция.
2. `tinyhumansai/tinycortex` — Memory Engine.
3. `nearai/ironclaw` — agent security и channels.
4. `moltis-org/moltis` — local-first AI gateway.
5. `kenforthewin/atomic` — Radar, research и evidence.
6. `SparkyWen/qcue` — approval и consolidation.
7. `screenpipe/screenpipe` — Timeline sensor.
8. `stalwartlabs/stalwart` — mail protocol intelligence.
9. `rustmailer/bichon` — archive и attachment intelligence.
10. `cozodb/cozo` — embedded knowledge reasoning.
11. `Pometry/Raphtory` — temporal relationship analytics.
12. `bosun-ai/swiftide` — typed agent workflows.
13. `tensorzero/tensorzero` — AI observability и evaluation.

## Уровень B: выбирать как технические компоненты

```text
Himalaya
Calendula
Matrix Rust SDK
whatsapp-rust
Tantivy
FastEmbed
Loro
Automerge
Iroh
Qdrant
LanceDB
mistral.rs
```

## Уровень C: продуктовые и исторические референсы

```text
Exomind v2
AppFlowy
Kuku
IWE
Minne
ActivityWatch
Rememex
seCall
TerminusDB
Windmill
```

---

# Лицензионные и архитектурные ловушки

- **Macro, AppFlowy, Windmill и Bichon** используют AGPL. Изучать можно, переносить код в закрытый или иначе лицензированный Макошь нужно крайне осторожно.
- **screenpipe** теперь source-available: личное некоммерческое использование разрешено, коммерческое требует лицензии.
- **SurrealDB** использует BSL для части распространения. Его лицензию надо проверять под конкретную модель поставки.
- **whatsapp-rust** является неофициальным клиентом и несёт риск блокировки аккаунта или нарушения условий Meta.
- **Exomind v2** годится как исторический референс, а не как живая основа.
- **Rig** прямо предупреждает о будущих breaking changes. Интерфейс вокруг него надо закрывать собственным Макошь port.

---

# Практический вывод для архитектуры Макошь

Макошь не стоит строить как fork одного из этих проектов. Ближайший разумный синтез выглядит так:

```text
Product composition
    Macro + AppFlowy references

Communications
    Stalwart
    Himalaya
    Matrix SDK
    whatsapp-rust
    Ferogram

Mail archive
    Bichon

Attention / Radar
    TinyCortex
    QCue
    Atomic
    Minne

Timeline sensors
    screenpipe
    ActivityWatch

Knowledge and history
    CozoDB
    Raphtory projection
    TerminusDB concepts

Search
    Tantivy
    FastEmbed
    optional Qdrant/LanceDB

AI
    Swiftide or Rig
    TensorZero
    mistral.rs/Ollama

Workflows
    Макошь Event Store + Outbox
    optional Restate

Local-first sync
    Loro or Automerge
    optional Iroh transport
```

При этом **доменные источники истины должны остаться собственностью Макошь**. Ни vector database, ни graph database, ни AI framework, ни workflow engine не должны внезапно стать местом, где живёт бизнес-модель. Движки считают, интеграции наблюдают, workflows координируют, AI предлагает, а домены решают и сохраняют историю. Иначе Personal Operating System быстро превратится в Personal Integration Accident.
