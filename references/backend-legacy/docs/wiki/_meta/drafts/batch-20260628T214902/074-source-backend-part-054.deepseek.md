### Summary / Резюме

В русскую Obsidian‑wiki `components/backend.md` добавляется документация трёх рабочих процессов бэкенда, обнаруженных в исходниках этого чанка: **конвейер синхронизации почты** (`email_sync_pipeline`), **графовая проекция** (`graph_projection`) и **эвристики анализа писем** (`email_intelligence`). Существующее содержимое страницы не встроено в контекст, поэтому предлагается полный вариант страницы, описывающий перечисленные компоненты на основе предоставленного кода.

### Proposed pages / Предлагаемые страницы

- `components/backend.md`

```markdown
# Бэкенд

## Конвейер синхронизации email (Email Sync Pipeline)

### Обзор

Конвейер обрабатывает пачку email‑сообщений, поступающих от поддерживаемых провайдеров (Gmail, iCloud, IMAP), и выполняет сквозную проекцию в доменную модель МакошьHub. Публичная точка входа – функция `project_email_sync_batch_with_mail_blobs` из модуля `email_sync_pipeline::service`.

Шаги конвейера:
1. **Запись сырых данных** – сохранение бинарного тела письма и метаданных в хранилище.
2. **Проекция сырых записей** – диспетчеризация сигнала, парсинг RFC‑822, извлечение вложений, запуск `analyze_ingested_message`.
3. **Извлечение знаний** – определение участников, персон, организаций и связей.
4. **Обновление кандидатов** – пополнение инбокса проверки (decisions, knowledge, tasks).
5. **Формирование отчёта** – `EmailSyncPipelineReport`.

### Запись сырых писем

Функция `record_email_sync_batch_with_mail_blobs` модуля `recording`:

- Извлекает бинарные данные письма в зависимости от провайдера:
  - Gmail – base64url (поле `raw_base64url`),
  - iCloud / IMAP – стандартный base64 (поле `raw_rfc822_base64`).
- Неподдерживаемые провайдеры (TelegramUser, TelegramBot, WhatsAppWeb, WhatsAppBusinessCloud, ZoomUser, ZoomServerToServer, YandexTelemostUser) возвращают ошибку `UnsupportedProviderKind`.
- Сохраняет блоб через `LocalCommunicationBlobPort` и регистрирует метаданные в `CommunicationBlobMetadataPort` с типом `message/rfc822`.
- Компонует payload, заменяя поля `raw_base64url` / `raw_rfc822_base64` на ссылки на сохранённый блоб (`raw_blob_id`, `raw_blob_sha256`, `raw_blob_storage_kind`, `raw_blob_storage_path`, `raw_blob_size_bytes`).
- Формирует `NewRawCommunicationRecord` с `raw_record_id` формата:
  `raw:v1:<len аккаунта>:<account_id>:<len вида записи>:<record_kind>:<len провайдер-идентификатора>:<provider_record_id>` (вид записи всегда `email_message`).
- Сохраняет запись через `CommunicationIngestionPort` и, при наличии, записывает контрольную точку.

### Проекция сырых записей

`project_raw_records` (модуль `raw_records`) для каждой сырой записи:
1. Диспетчеризует почтовый сигнал через `dispatch_mail_raw_signal` – если сигнал не принят, запись пропускается.
2. Пытается спроецировать принятый сигнал в сообщение через `project_accepted_signal_if_runtime_allows` – может вернуть `None`, тогда запись пропускается.
3. Парсит RFC‑822 из блоба вызовом `parse_raw_email_message_from_blob`.
4. Запускает анализ входящего сообщения `analyze_ingested_message`.
5. Вызывает `project_attachments` для извлечения вложений.

### Извлечение вложений

Модуль `attachments`, функция `project_attachments`:

- Для каждого `ParsedEmailAttachment` сохраняет тело через `blob_store.put_blob`.
- Регистрирует блоб с указанием `content_type` из письма.
- Выполняет проверку безопасности через `AttachmentSafetyScanner` (по умолчанию используется `HeuristicAttachmentSafetyScanner`).
- Создаёт `NewCommunicationAttachment`, связывая его с сообщением, сырой записью и блобом.
- Ведёт учёт количества обработанных, извлечённых и непросканированных вложений (`AttachmentProjectionReport`).

### Извлечение знаний

Модуль `knowledge`, функция `project_message_knowledge`:

- Парсит отправителя и получателей с помощью `parse_email_participant` (модуль `participants`), извлекая email, отображаемое имя и роль (`sender` / `recipient`).
- Для каждого участника:
  - Выполняет `upsert_email_person_with_observation` – создаёт или находит персону в домене `persons`.
  - Регистрирует участника сообщения через `upsert_message_participant`.
  - Создаёт событие связи (`email_sent` / `email_received`) через `insert_relationship_event` (модуль `relationships`).
  - Проецирует организацию по email-домену через `project_email_participant_organization` (модуль `organizations`).

**Организации** (модуль `organizations`):
- Из адреса извлекается домен; публичные почтовые домены исключаются:
  `gmail.com`, `googlemail.com`, `icloud.com`, `me.com`, `mac.com`, `outlook.com`, `hotmail.com`, `live.com`, `yahoo.com`, `proton.me`, `protonmail.com`, `mail.ru`, `yandex.ru`.
- Организация upsert’ится с `organization_id` вида `org:v1:email-domain:<len домена>:<домен>`.
- Связь персоны с организацией регистрируется через `OrganizationContactLinkPort`.
- На основе этой связи **материализуется** relationship типа `member_of` с помощью `RelationshipReviewPort`; создаётся запись `NewRelationship` и доказательство `NewRelationshipEvidence` с метаданными об организации.

### Кандидаты в инбокс проверки

Модуль `candidates`, функция `refresh_message_context_candidates`:

- После проекции всех сообщений пакета вызывает три операции в `review_inbox`:
  - `refresh_message_decisions_into_review`,
  - `refresh_message_knowledge_candidates_into_review`,
  - `refresh_message_task_candidates_into_review`.
- Каждая из них пополняет соответствующие списки кандидатов для последующего подтверждения/отклонения пользователем.

### Отчёт

Структура `EmailSyncPipelineReport` (модуль `report`) агрегирует все счётчики операций:

- `imported_records`
- `raw_blobs_upserted`
- `projected_messages`
- `attachment_blobs_upserted`, `attachments_extracted`, `attachments_not_scanned`
- `upserted_persons`, `upserted_person_identities`
- `upserted_message_participants`, `upserted_relationship_events`
- `upserted_organizations`, `upserted_organization_contact_links`
- `refreshed_decision_candidates`, `refreshed_knowledge_candidates`, `refreshed_task_candidates`
- `checkpoint_saved`

### Ошибки

- `EmailSyncRecordError` – ошибки низкого уровня: пустые поля, проблемы декодирования base64, неподдерживаемые провайдеры.
- `EmailSyncPipelineError` – агрегирует ошибки записи, проекции сообщений, сигналов, персон, организаций, связей, задач, вложений, а также ошибки БД и невалидные адреса участников.

## Графовая проекция (Graph Projection)

### Обзор

Сервис `GraphProjectionService` (модуль `graph_projection`) преобразует доменные сущности (сообщения, решения, обязательства, документы) в узлы и рёбра графа, используя `GraphProjectionPort`. Поддерживается повторная проекция с удалением старых рёбер перед созданием новых.

### Проекция сообщений

Модуль `messages`, основной метод `project_message`:

- Создаёт узел `Message` с идентификатором `message_id` и свойствами (account_id, provider_record_id, observation_id, occurred_at и т.д.).
- Для отправителя и каждого получателя ищет соответствующую персону по email в таблице `persons`. Если персона не найдена, создаёт узел `EmailAddress`.
- Создаёт направленные рёбра:
  - `PersonSentMessage` / `PersonReceivedMessage` – если найден узел персоны,
  - `EmailAddressSentMessage` / `EmailAddressReceivedMessage` – если только адрес.
- К каждому ребру прикрепляется доказательство `GraphEvidence` с источником `Message` (тема, observation_id, провайдер-идентификатор и т.д.).

### Проекция решений (decisions)

Модуль `decisions`, метод `project_decision`:

- Читает записи из таблицы `decisions` (decision_id, title, status, review_state, confidence).
- Из таблицы `decision_impacted_entities` извлекает сущности, на которые влияет решение, с указанием `entity_kind` и `impact_type`.
- Создаёт узел `Decision` и рёбра до затронутых сущностей; тип сущности маппится на `GraphNodeKind`:
  `persona` → `Person`, `organization` → `Organization`, `project` → `Project`, `communication` → `Message`, `document` → `Document`, `task` → `Task`, `event` → `Event`, `decision` → `Decision`, `obligation` → `Obligation`, `knowledge` → `Knowledge`.
- Для каждого ребра используется доказательство из `decision_evidence` (source_kind, source_id, observation_id, quote).
- Состояние проверки (`review_state`) маппится: `suggested` → `GraphReviewState::Suggested`, `user_confirmed` → `UserConfirmed`, `user_rejected` → `UserRejected`.

### Проекция обязательств (obligations)

Модуль `obligations`, метод `project_obligation`:

- Аналогичен проекции решений: создаётся узел `Obligation`, рёбра к `obligated_entity` и, если задана, `beneficiary_entity`. Типы сущностей преобразуются той же функцией `entity_graph_node_kind`.
- Доказательства из `obligation_evidence`.

### Проекция документов

Модуль `documents`, метод `project_document`:

- Создаёт узел `Document` с свойствами `document_kind`, `source_fingerprint`, `imported_at`.

### Доказательства

Модуль `evidence` предоставляет вспомогательные функции:

- `message_evidence` – формирует `NewGraphEvidence` для сообщения с excerpt (тема), observation_id и метаданными.
- `project_message_evidence` – аналогично, но с указанием `match_rule: "project_keyword"` и дополнительными полями (account_id, occurred_at, projected_at).
- `project_document_evidence` – для документов (document_kind, source_fingerprint, imported_at).

### Модели и состояния

- `GraphProjectionReport` – агрегирует количество upsert'нутых узлов, рёбер и доказательств.
- `MessageEndpoint` – перечисление (`Person` / `EmailAddress`), определяет тип relationship для направления отправки/получения.
- `RelationshipDirection` – `Sent` / `Received`.
- Константа `PROJECT_KEYWORD_CONFIDENCE = 0.75` используется для связей проектов, добавленных автоматически по ключевым словам; для подтверждённых пользователем связей confidence = 1.0, для отклонённых – 0.0 (модуль `helpers`).

## Интеллект писем (Email Intelligence)

> Контекст этого чанка включает только **тесты** модуля `email_intelligence`; реализация сервиса `EmailIntelligenceService` и типов‑результатов не встроена. Приведённое ниже опирается исключительно на поведение, проверяемое в тестах.

### Эвристическая оценка важности

`EmailIntelligenceService::heuristic_score(&ProjectedMessage) -> u32` (точный тип возврата не подтверждён, тесты сравнивают с i32-литералами)

Поведение, демонстрируемое тестами:

- Письмо с темой "URGENT: Action Required" и телом "Please respond ASAP" → счёт **≥ 35**.
- Письмо со словом "invoice", "payment", "Amount due" в теле → счёт **≥ 50**.
- Маркетинговое письмо с фразами "Click here", "unsubscribe" → счёт **≤ 30**.

### Эвристическая категоризация

`EmailIntelligenceService::heuristic_category(&ProjectedMessage) -> Option<String>`

- "Invoice #123" с телом "Here is your invoice for services" → категория `"finance"`.
- "Contract" с телом "Please review the NDA and agreement" → категория `"legal"`.
- "Hello" / "Just checking in" → `None`.

### Извлечение структурированной сводки

`EmailIntelligenceService::heuristic_structured_summary(&ProjectedMessage)` возвращает структуру, содержащую:

- `key_points: Vec<String>` – ключевые пункты.
- `action_items: Vec<String>` – требуемые действия.
- `risks: Vec<String>` – идентифицированные риски.
- `deadlines: Vec<String>` – дедлайны.
- `event_candidates` – кандидаты на события (проверяется наличие поля `title`).
- `persona_candidates` – кандидаты на персон (поле `title`).
- `organization_candidates` – кандидаты на организации (поле `title`).
- `document_candidates` – кандидаты на документы (поле `title`).
- `agreement_candidates` – кандидаты на соглашения (поле `title`).

Тесты подтверждают:
- Извлечение конкретных фраз из тела, распознавание действий ("Please review the NDA"), рисков ("payment risk") и дедлайнов ("Friday").
- Извлечение кандидатов на события ("Meeting on Monday"), персон ("Ada Lovelace"), организаций ("acme.example"), документов ("MSA") и соглашений ("NDA").
- **Дедупликацию и ограничение размера** результатов (повторяющиеся значения удаляются, результат не разрастается безгранично).

### Категории писем

`EmailCategory` – перечисление с подтверждёнными вариантами:

- `Critical` (из строки `"critical"`)
- `Spam` (из `"spam"`)
- `Finance` (из `"finance"`)

Метод `EmailCategory::parse(&str) -> Option<EmailCategory>` выполняет парсинг из строки.
```

### Source coverage / Покрытие источников

- **`backend/src/workflows/email_intelligence/tests.rs`**
  - Тесты `heuristic_score` с пороговыми значениями для urgent, finance, marketing.
  - Тесты `heuristic_category` (finance, legal, none).
  - Тест `heuristic_structured_summary_extracts_key_points_actions_risks_and_deadlines` и `_extracts_mail_knowledge_candidates` – поля сводки.
  - Тест `heuristic_structured_summary_is_bounded_and_deduplicated` – дедупликация.
  - Тест `email_category_from_str_all_valid` – парсинг вариантов EmailCategory.

- **`backend/src/workflows/email_sync_pipeline.rs`**
  - Публичный API модуля: `record_email_sync_batch`, `record_email_sync_batch_with_mail_blobs`, `EmailSyncPipelineReport`, `project_email_sync_batch_with_mail_blobs`.

- **`backend/src/workflows/email_sync_pipeline/attachments.rs`**
  - Функция `project_attachments`, `AttachmentProjectionReport`, сохранение блобов, проверка безопасности, upsert вложений.

- **`backend/src/workflows/email_sync_pipeline/candidates.rs`**
  - `refresh_message_context_candidates` и вызовы функций `review_inbox`.

- **`backend/src/workflows/email_sync_pipeline/errors.rs`**
  - Структура ошибок `EmailSyncRecordError` и `EmailSyncPipelineError` с их вариантами.

- **`backend/src/workflows/email_sync_pipeline/ids.rs`**
  - Формат `raw_record_id`, константа `EMAIL_MESSAGE_RECORD_KIND`.

- **`backend/src/workflows/email_sync_pipeline/knowledge.rs`**
  - `project_message_knowledge`, `MessageKnowledgeReport`, обработка участников, персон, связей и организаций.

- **`backend/src/workflows/email_sync_pipeline/organizations.rs`**
  - `project_email_participant_organization`, фильтрация публичных доменов, создание/поиск организации, связывание персон с организациями, материализация relationship `member_of`.

- **`backend/src/workflows/email_sync_pipeline/participants.rs`**
  - Парсинг участника `parse_email_participant`, сохранение `upsert_message_participant`.

- **`backend/src/workflows/email_sync_pipeline/raw_payload.rs`**
  - Декодирование сырых данных `raw_message_bytes` для разных провайдеров, `payload_with_raw_blob_reference`.

- **`backend/src/workflows/email_sync_pipeline/raw_records.rs`**
  - `project_raw_records` – цикл проекции сырых записей, диспетчеризация сигнала, проекция сообщения, парсинг RFC‑822, анализ, вложения.

- **`backend/src/workflows/email_sync_pipeline/recording.rs`**
  - `record_email_sync_batch` и `record_email_sync_batch_with_mail_blobs`, сохранение контрольных точек.

- **`backend/src/workflows/email_sync_pipeline/relationships.rs`**
  - `insert_relationship_event` – создание событий `email_sent` / `email_received`.

- **`backend/src/workflows/email_sync_pipeline/report.rs`**
  - Структура `EmailSyncPipelineReport` с полным перечнем полей.

- **`backend/src/workflows/email_sync_pipeline/service.rs`**
  - `project_email_sync_batch_with_mail_blobs` – оркестрация всего конвейера и формирование итогового отчёта.

- **`backend/src/workflows/graph_projection.rs`**
  - Публичный API модуля: `GraphProjectionError`, `GraphProjectionReport`, `GraphProjectionService`.

- **`backend/src/workflows/graph_projection/constants.rs`**
  - Константа `PROJECT_KEYWORD_CONFIDENCE = 0.75`.

- **`backend/src/workflows/graph_projection/decisions.rs`**
  - Методы `list_decisions`, `project_decision`, `list_decision_impacted_entities`, `decision_evidence`, `delete_decision_edges`.
  - Вспомогательные функции `decision_review_state`, `entity_graph_node_kind`.
  - Маппинг review_state и entity_kind.

- **`backend/src/workflows/graph_projection/documents.rs`**
  - `list_documents`, `project_document`.

- **`backend/src/workflows/graph_projection/errors.rs`**
  - Варианты `GraphProjectionError`.

- **`backend/src/workflows/graph_projection/evidence.rs`**
  - Функции `message_evidence`, `project_message_evidence`, `project_document_evidence`.

- **`backend/src/workflows/graph_projection/helpers.rs`**
  - `normalize_email_address`, `project_review_graph_state`, `project_review_confidence`.

- **`backend/src/workflows/graph_projection/messages.rs`**
  - Методы `list_messages`, `project_message`, `delete_message_edges`, `resolve_message_endpoint`, `person_by_normalized_email`, `project_message_endpoint`.

- **`backend/src/workflows/graph_projection/models.rs`**
  - `GraphProjectionReport`, `PersonRow`, `MessageRow`, `DocumentRow`, `MessageEndpoint`, `RelationshipDirection`, маппинг relationship‑типов.

- **`backend/src/workflows/graph_projection/obligations.rs`**
  - Методы `list_obligations`, `project_obligation`, `project_obligation_entity_edge`, `obligation_evidence`, `delete_obligation_edges`, `obligation_review_state`.

### Drift candidates / Кандидаты на drift

Контекст не содержит текущей версии wiki‑страницы `components/backend.md`, поэтому невозможно зафиксировать расхождения между документацией и кодом. Если страница уже содержала описания этих компонентов, они могли бы расходиться с приведёнными исходниками (например, изменились сигнатуры, отчёты или маппинги состояний). В предоставленном коде внутренних противоречий не обнаружено. Отдельно отмечу, что модули `graph_projection::persons`, `graph_projection::projects`, `graph_projection::rows` и `graph_projection::service` упоминаются в объявлении модуля, но их исходники **не встроены** в контекст – документация по ним не может быть проверена или составлена.
