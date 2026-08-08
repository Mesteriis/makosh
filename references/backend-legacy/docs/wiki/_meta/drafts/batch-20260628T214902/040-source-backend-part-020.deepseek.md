## Summary / Резюме

Страницу `components/backend.md` необходимо создать или обновить, чтобы задокументировать основные компоненты бэкенда, исходя из предоставленных исходных файлов домена коммуникаций (`communications`). Документ будет описывать конвейер приёма и анализа сообщений, проекцию сообщений, хранилище и поиск, состояния workflow, события, юридические документы и импорт фикстур. Все утверждения основаны исключительно на вложенных исходных текстах.

## Proposed pages / Предлагаемые страницы

### `components/backend.md`

```markdown
# Бэкенд Макошь

Бэкенд реализован на Rust и построен по модульной архитектуре. Данная страница описывает подсистемы, видимые в предоставленных исходных файлах домена `communications`.

## Коммуникационный домен

Домен отвечает за приём, обработку, хранение и поиск коммуникационных сообщений (email, Telegram, WhatsApp), а также за управление юридическими документами и папками.

### Приём и анализ сообщений

Каждое входящее сообщение проходит обязательный эвристический анализ. Логика реализована в `backend/src/domains/communications/ingestion.rs`.

- **Эвристическая оценка важности** (`heuristic_score`) вычисляет целочисленный балл от 0 до 100.
  Базовый балл: 30.
  Баллы добавляются за наличие ключевых слов в теме или теле письма:
  +15  `urgent`, `asap`, `deadline`, `immediately`, `critical`, `action required`
  +20  `invoice`, `payment`, `factura`, `bill`, `amount due`, `receipt`, `tax`
  +25  `contract`, `agreement`, `nda`, `legal`, `liability`, `confidential`, `attorney`
  +10  наличие знака вопроса `?`
  +10  ключевые слова вложений (`attached`, `attachment`, и т.д.)
  -20  признаки рассылки/спама (`unsubscribe`, `opt out`, …)
  -10  тело короче 50 символов
  Результат зажимается в `[0, 100]`.

- **Категоризация** (`heuristic_category`) возвращает `Option<String>` со значениями:
  `finance` – тело содержит `invoice` / `factura` / `payment`
  `legal` – тело содержит `contract` / `nda` / `agreement`
  `marketing` – тело содержит `unsubscribe` / `newsletter`
  `notification` – тема или тело содержит `notification`
  `suspicious` – тело содержит `click here` и (`account` или `verify`)
  `None` – категория не определена.

- **Детектор спама**: сообщение считается спамом, если тело содержит `unsubscribe` **и** (`buy now` / `limited offer` / `click here`).

- **Детектор фишинга**: тело содержит (`verify your account` / `confirm your password` / `urgent action required`) **и** отправитель не содержит email-адрес аккаунта (`account_id`).

- **Автоматический переход workflow** (`auto_workflow_state`):
  - `Spam` – фишинг или (спам и оценка < 20)
  - `NeedsAction` – оценка >= 75
  - `New` – во всех остальных случаях.

- Результаты анализа (категория, оценка) сохраняются через `set_ai_analysis`. Если состояние изменилось не на `New`, вызывается `transition_workflow_state`.

### Импорт фикстурных email

Файл `backend/src/domains/communications/import.rs` содержит функцию `import_fixture_email_messages`, которая:

- Принимает `FixtureEmailImportRequest` (account_id, import_batch_id, JSON-строку с фикстурой).
- Парсит фикстуру через `parse_fixture_email_messages`.
- Для каждого сообщения создаёт `NewRawCommunicationRecord` с идентификатором формата `raw:v1:<len-account_id>:<account_id>:<len-record_kind>:email_message:<len-provider_record_id>:<provider_record_id>`.
- Сохраняет записи через `CommunicationIngestionStore::record_raw_source`.
- Возвращает `FixtureEmailImportReport` с количеством вставленных или уже существовавших записей.

### Проекция сообщений

Модуль `backend/src/domains/communications/messages/` отвечает за преобразование сырых записей (`StoredRawCommunicationRecord`) в спроецированные сообщения (`ProjectedMessage`).

#### Идентификаторы сообщений

Формат `message_id` (файл `messages/ids.rs`):
`msg:v1:<len-account_id>:<account_id>:<len-provider_record_id>:<provider_record_id>`.

#### Модели данных

- **`NewProjectedMessage`** – структура для создания сообщения. Содержит обязательные поля: `message_id`, `raw_record_id`, `account_id`, `provider_record_id`, `subject`, `sender`, `recipients` (Vec<String>), `body_text`, `channel_kind`, `delivery_state`, `message_metadata` (JSON-объект). Валидация запрещает пустые поля (кроме случая `allow_empty_body_text`). `message_metadata` должно быть объектом JSON.

- **`ProjectedMessage`** – полная модель сообщения, включает все поля `NewProjectedMessage` плюс:
  - `observation_id`
  - `projected_at` (DateTime<Utc>)
  - `conversation_id`, `sender_display_name`
  - `workflow_state` (перечисление `WorkflowState`)
  - `importance_score` (Option<i16>)
  - `ai_category`, `ai_summary`, `ai_summary_generated_at`
  - `local_state` (`LocalMessageState`)
  - `local_state_changed_at`, `local_state_reason`

- **`ProjectedMessageSummary`** = `ProjectedMessage` + `attachment_count`.

- **`ProjectedMessagePage`** = список `ProjectedMessageSummary`, курсор пагинации `next_cursor`, флаг `has_more`.

- **`ProjectedMessagePageQuery`** – параметры запроса страницы: фильтры по аккаунту, состоянию, каналу, беседе, текстовому запросу, `MessageSearchQuery`, локальному состоянию, курсору и лимиту.

- **`WorkflowStateCount`** – пара (состояние, количество).

#### Проекция из сырых записей

Файл `messages/projection.rs` предоставляет функции:

- `project_raw_email_message` – извлекает `subject`, `from`, `to`, `body_text` из payload сырой записи и вызывает `store.upsert_message`.
- `project_raw_email_message_from_blob` – читает сырой email из блоб-хранилища (поддерживается только `local_fs`), парсит RFC822 и проецирует.
- `project_parsed_raw_email_message` – проекция уже распарсенного сообщения.

#### Хранилище сообщений

`MessageProjectionStore` (файл `messages/store.rs`) – обёртка над `PgPool`. Предоставляет операции:

- **upsert** (в подмодуле `upsert`, не встроен) – вставка или обновление `NewProjectedMessage`.
- **Запросы** (`store/queries.rs`):
  - `recent_messages` – последние активные сообщения с JOIN к вложениям, сортировка по `occurred_at`.
  - `message` – одно сообщение по `message_id`.
  - `list_messages` / `list_messages_page` – постраничный список с фильтрами и пагинацией на основе курсора.
  - `count_messages_by_state` – подсчёт по состояниям workflow.
  - Курсор кодируется в base64 (URL safe, без padding) и содержит `sort_at`, `projected_at`, `message_id`.
- **AI-анализ** (`store/metadata.rs`):
  - `set_ai_analysis` – обновляет `ai_category`, `ai_summary` (и время генерации), `importance_score`. Оценка валидируется в диапазоне `0..=100`.
- **Метаданные** (`store/metadata.rs`):
  - `set_message_metadata` / `set_message_metadata_with_observation` – обновляет `message_metadata` (должен быть объектом) и опционально связывает с наблюдением (`observation`).
- **Локальное состояние** (`store/local_state.rs`):
  - `move_to_local_trash` – перемещает в корзину, устанавливает `local_state = 'trash'`, `local_state_reason`.
  - `restore_from_local_trash` – восстанавливает в `active`.
  - Поддерживают привязку к наблюдению через транзакцию.
- **Участники** (`store/participants.rs`):
  - `upsert_email_participant` – вставляет или обновляет запись в `communication_message_participants` и связывает её с наблюдением.

#### Поиск сообщений

Поисковый запрос представляется структурой `MessageSearchQuery` (`messages/models.rs`), которая поддерживает:

- `plain_terms` – простые поисковые фразы
- `subject_contains` / `subject_equals`
- `body_contains` / `body_equals`
- `sender_contains` / `sender_equals`
- `all_contains` / `all_equals`
- `match_mode` – `All` (по умолчанию) или `Any`
- `expression` – явное поисковое выражение с булевой логикой.

Парсинг запроса (файл `messages/query_parser.rs`, частично обрезан) поддерживает:

- Правила вида `subject:текст`, `body=текст`, `from:текст`, `all:текст`.
- Модификатор `mode:any` или `mode:all`.
- Явные выражения со скобками, `AND`, `OR`, кавычками для точных фраз.

Построение SQL WHERE-условий из поискового запроса выполняет модуль `messages/search.rs`. Основные функции:

- `append_message_search_filter` – добавляет `AND (...)` к SQL-запросу. Использует `coalesce` для полей с учётом возможных NULL.
- `push_expression` рекурсивно обходит дерево выражений, формируя предикаты `ILIKE` для `Contains` и `=` для `Equals` (оба с `lower()`).
- Для `all`-поля строится конкатенация subject, sender, body_text, provider_record_id, sender_display_name через `concat_ws`.

#### Состояния сообщений

Файл `messages/states.rs` определяет два перечисления:

**`WorkflowState`** – состояние рабочего процесса:
- `New`, `Reviewed`, `NeedsAction`, `Waiting`, `Done`, `Archived`, `Muted`, `Spam`.
- Определены допустимые переходы через метод `valid_transitions()`:
  - `New` → Reviewed, NeedsAction, Archived, Muted, Spam
  - `Reviewed` → New, NeedsAction, Waiting, Done, Archived, Muted, Spam
  - `NeedsAction` → Waiting, Done, Archived, Reviewed
  - `Waiting` → NeedsAction, Done, Archived, Reviewed
  - `Done` → Archived, Reviewed, NeedsAction
  - `Archived` → Reviewed, NeedsAction, Done
  - `Muted` → Reviewed, Archived
  - `Spam` → Reviewed, Archived, New

**`LocalMessageState`** – локальное состояние:
- `Active` – активное сообщение (фильтр БД: `active`)
- `Trash` – в корзине (фильтр БД: `trash`)
- `All` – все сообщения (без фильтра).

#### Событийная интеграция с провайдерами

Файл `messages/provider_observation_projection.rs` (частично обрезан) обрабатывает события, поступающие от внешних провайдеров:

- Константа `COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER` = `"communication_provider_observation_projection"`.
- Поддерживаемые события сигналов:
  - `signal.accepted.mail.message`
  - `signal.accepted.telegram.message`
  - `signal.accepted.whatsapp.message`
- Поддерживаемые события доставки почты:
  - `signal.accepted.mail.delivery_status`
  - `signal.accepted.mail.read_receipt`
- Поддерживаемые наблюдения Telegram (контент, метаданные, состояние доставки, закрепление, загрузка вложений).
- Функция `project_provider_observation_event` мультиплексирует обработку: сигналы доставки почты идут в `consume_accepted_mail_delivery_signal`, базовые сигналы — в `consume_accepted_signal_event`, наблюдения Telegram проекцируются отдельно.
- Для email: сообщение проецируется либо из payload, либо из blob (если есть `raw_blob_storage_path`).
- Для WhatsApp: из payload извлекаются `provider_chat_id`, `chat_title`, `sender_display_name`, `text`, `delivery_state`; `channel_kind` определяется по `provenance.provider_kind` (по умолчанию `whatsapp_web`).
- Для Telegram: аналогично, плюс поддержка пустого тела (`allow_empty_body_text`), если `runtime == "tdlib"` и есть `tdlib_raw`.

### Юридические документы

Файл `backend/src/domains/communications/legal.rs` реализует управление юридическими документами.

**`LegalDocType`** – тип документа:
- `Contract`, `Nda`, `Msa`, `Dpa`, `Agreement`, `LegalNotice`, `Claim`, `CourtDocument`, `TaxNotice`, `GovernmentDoc`, `Other`.

**`LegalDocStatus`** – статус:
- `Active`, `Expired`, `PendingReview`, `Signed`, `Terminated`, `Draft`.

**`LegalDocument`** – модель документа с полями:
- `document_id`, `message_id` (опционально), `document_type`, `title`, `parties` (Vec<String>), `effective_date`, `expiry_date`, `amount`, `currency`, `status`, `linked_project_id`, `risks` (Vec<String>), `metadata` (JSON), `created_at`, `updated_at`.

**`LegalDocumentStore`** – хранилище (PgPool):
- `upsert` – вставляет или обновляет документ (конфликт по `document_id`). При вставке валидирует, что `document_id` и `title` не пусты.
- `list` – возвращает список документов с опциональной фильтрацией по типу и статусу, сортировка по `effective_date` (или `created_at`).

### События папок

Файл `backend/src/domains/communications/folders/events.rs` генерирует события для операций с папками:

- Типы событий: `mail.folder.created`, `mail.folder.updated`, `mail.folder.deleted`.
- Для сообщений в папках: `mail.folder_message.copied`, `mail.folder_message.moved`.
- Идентификатор события формируется как `mail_folder_event:{event_type}:{subject_id}:{timestamp_nanos_hex}`.
- Конверт события (`NewEventEnvelope`) включает:
  - `kind`: `"mail_folder_api"` или `"mail_folder_message"`
  - `actor`: `"makosh-frontend"`
  - `payload`: сериализованная структура папки или ответа операции
  - `provenance`: `source_kind = "local_api"`
  - `correlation_id`: идентификатор папки или сообщения.

### Ошибки

Модуль `messages/errors.rs` определяет общее перечисление `MessageProjectionError`, включающее варианты:
- `Sqlx` (ошибки БД)
- `CommunicationStorage` (ошибки хранилища)
- `Rfc822` (ошибки парсинга RFC822)
- `ObservationStore` (ошибки хранилища наблюдений)
- `MissingPayloadField`, `EmptyField` (валидация полей)
- `RawRecordTupleMismatch`, `RawRecordNotFound` (несоответствие сырых записей)
- `InvalidStoredRecipients`, `InvalidMessageMetadata` (некорректный формат данных)
- `UnsupportedRawBlobStorageKind` (неподдерживаемый тип blob-хранилища)
- `InvalidLimit` (лимит вне диапазона 1..5000)
- `InvalidCursor` (некорректный курсор пагинации)
- `MessageNotFound`, `InvalidWorkflowState`, `InvalidLocalState`, `InvalidImportanceScore`.

Также есть отдельный тип `FixtureEmailImportError` в `import.rs`, агрегирующий ошибки источника и приёма.
```

## Source coverage / Покрытие источников

### `backend/src/domains/communications/folders/events.rs`
- Типы событий папок: `mail.folder.created`, `mail.folder.updated`, `mail.folder.deleted`
- Типы событий сообщений в папках: `mail.folder_message.copied`, `mail.folder_message.moved`
- Формат идентификатора события: `mail_folder_event:{event_type}:{subject_id}:{timestamp_nanos_hex}`
- Структура `NewEventEnvelope`: kind, actor (`makosh-frontend`), payload, provenance (`local_api`), correlation_id

### `backend/src/domains/communications/import.rs`
- Функция `import_fixture_email_messages` и `import_fixture_email_messages_with_records`
- Формат `raw_record_id`: `raw:v1:<len-account_id>:<account_id>:<len-record_kind>:email_message:<len-provider_record_id>:<provider_record_id>`
- `FixtureEmailImportRequest` и `FixtureEmailImportReport`
- Ошибка `FixtureEmailImportError`

### `backend/src/domains/communications/ingestion.rs`
- Списки ключевых слов для эвристик
- Логика `heuristic_score` и `heuristic_category`
- Детекторы спама и фишинга
- Автоматическое определение `WorkflowState` (Spam, NeedsAction, New)
- Сохранение анализа через `set_ai_analysis` и `transition_workflow_state`

### `backend/src/domains/communications/legal.rs`
- `LegalDocType` (variants: Contract, Nda, Msa, Dpa, Agreement, LegalNotice, Claim, CourtDocument, TaxNotice, GovernmentDoc, Other)
- `LegalDocStatus` (variants: Active, Expired, PendingReview, Signed, Terminated, Draft)
- `LegalDocument` структура
- `LegalDocumentStore` с методами `upsert` и `list`
- Валидация `document_id` и `title` не пусты

### `backend/src/domains/communications/messages.rs`
- Перечень реэкспортов (не детализируется на странице, но подтверждает структуру модуля)

### `backend/src/domains/communications/messages/errors.rs`
- Полный перечень вариантов `MessageProjectionError`

### `backend/src/domains/communications/messages/ids.rs`
- Формат `message_id`: `msg:v1:<len>:<account_id>:<len>:<provider_record_id>`

### `backend/src/domains/communications/messages/models.rs`
- `NewProjectedMessage` и его валидация
- `ProjectedMessage` и все его поля
- `ProjectedMessageSummary`, `ProjectedMessagePage`, `WorkflowStateCount`
- `MessageSearchQuery` (plain_terms, subject_contains, body_contains, sender_contains, all_contains, match_mode, expression)
- `ProjectedMessagePageQuery`
- `MessageSearchMatchMode` (All, Any)

### `backend/src/domains/communications/messages/payload.rs`
- Функции `required_payload_string`, `required_payload_string_array`, `recipients_from_value`

### `backend/src/domains/communications/messages/projection.rs`
- `project_raw_email_message` – проекция из payload
- `project_raw_email_message_from_blob` – проекция из blob (поддерживается только `local_fs`)
- `parse_raw_email_message_from_blob` – чтение blob и парсинг RFC822
- `project_parsed_raw_email_message`

### `backend/src/domains/communications/messages/provider_channel_store.rs`
- Методы: `message_by_id`, `message_by_provider_record_id`, `recent_messages`, `messages_by_ids`, `search_messages`, `pinned_messages`, `body_text`, `message_ids_by_metadata_string`, `message_id_by_provider_record_id`, `reference_summaries`
- SQL-запросы с фильтрацией по `channel_kinds`, `account_id`, `conversation_id`
- Поддержка поиска по `ILIKE` и фильтрации закреплённых по `message_metadata->>'is_pinned'` или `'pinned'`

### `backend/src/domains/communications/messages/provider_observation_projection.rs`
- `COMMUNICATION_PROVIDER_OBSERVATION_CONSUMER`
- Поддерживаемые типы событий: `signal.accepted.mail.message`, `signal.accepted.telegram.message`, `signal.accepted.whatsapp.message`, а также наблюдения доставки и Telegram
- Логика проекции для mail (из payload или blob), WhatsApp, Telegram
- Специфичная обработка для Telegram с `tdlib` и пустым телом
- Функция `accepted_signal_projection_runtime_allows`

### `backend/src/domains/communications/messages/query_parser.rs`
- Правила парсинга: `subject:`, `body:`, `from:`, `all:`, `mode:any` / `mode:all`
- Явные выражения со скобками, `AND`, `OR`, кавычками
- Структуры `MessageSearchRule`, `SearchToken`, `ExplicitSearchParser`

### `backend/src/domains/communications/messages/rows.rs`
- `row_to_projected_message` и `row_to_projected_message_summary`
- Извлечение `WorkflowState` и `LocalMessageState` из строк БД с fallback на `New`/`Active`

### `backend/src/domains/communications/messages/search.rs`
- `append_message_search_filter` – построение SQL WHERE
- Поддержка `Contains` через `ILIKE` и `Equals` через `=`, оба с `lower()`
- Комбинированная цель поиска для `all`-поля через `concat_ws` (subject, sender, body_text, provider_record_id, sender_display_name)

### `backend/src/domains/communications/messages/states.rs`
- `WorkflowState` и все допустимые переходы
- `LocalMessageState` (Active, Trash, All) с фильтрами БД

### `backend/src/domains/communications/messages/store.rs`
- Определение структуры `MessageProjectionStore`

### `backend/src/domains/communications/messages/store/local_state.rs`
- `move_to_local_trash` / `move_to_local_trash_with_observation`
- `restore_from_local_trash` / `restore_from_local_trash_with_observation`
- Привязка к наблюдению через транзакцию

### `backend/src/domains/communications/messages/store/metadata.rs`
- `set_ai_analysis` с валидацией важности в `0..=100`
- `set_message_metadata` / `set_message_metadata_with_observation`

### `backend/src/domains/communications/messages/store/participants.rs`
- `upsert_email_participant` с `ON CONFLICT` и привязкой к наблюдению

### `backend/src/domains/communications/messages/store/queries.rs`
- `recent_messages`, `message`, `list_messages`, `list_messages_page`
- Пагинация с курсором (base64, URL-safe, без padding)
- `count_messages_by_state` / `count_messages_by_state_with_local_state`
- Поддержка фильтрации по `channel_kind` с группировкой по алиасам (mail, telegram, whatsapp)

## Drift candidates / Кандидаты на drift

В предоставленном контексте отсутствуют существующие wiki-страницы, ADR или документация, с которыми можно было бы сравнить код. Расхождения между кодом и документацией не выявлены.
