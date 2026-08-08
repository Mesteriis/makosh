# Summary / Резюме

В русскую Obsidian-вики добавляется страница `operations/backend-tests.md`, описывающая структуру, категории и ключевые проверки набора интеграционных и юнит-тестов бэкенда из `backend/tests`.
Основанием служат только встроенные файлы исходников; внешние знания не добавляются.
Страница объясняет, какие аспекты поведения верифицируются тестами (ingestion, архитектурные ограничения, конфигурация, ConnectRPC API, движок consistency, context packs), и ссылается на конкретные проверяемые утверждения из кода.

# Proposed pages / Предлагаемые страницы

## `operations/backend-tests.md`

```markdown
# Тесты бэкенда

Тесты бэкенда расположены в каталоге `backend/tests/` и покрывают следующие направления:

- Корректность работы слоя приёма коммуникаций (`communication_ingestion`)
- Архитектурные ограничения (состав доменов, маршруты, размер файлов)
- Парсинг и валидацию конфигурации (`config`)
- ConnectRPC-интерфейс коммуникаций (`communications_connectrpc`)
- Движок обнаружения противоречий (`consistency_contradiction`)
- Хранилище контекстных пакетов (`context_packs`)

Тесты используют общий модуль поддержки `testkit::context::TestContext` для получения строки подключения к БД (PostgreSQL), а также хелперы вроде `unique_suffix()` для генерации уникальных идентификаторов.

## Тесты приёма коммуникаций (`communication_ingestion`)

Тесты этого модуля проверяют запись сырых записей (raw sources), привязку секретов к учётным записям провайдеров и чтение учётных данных через `ProviderCredentialReader`.

### Запись сырых записей (`raw_records.rs`)

Тест `communication_ingestion_records_raw_sources_idempotently_against_postgres` проверяет:

- Запись `NewRawCommunicationRecord` возвращает `raw_record_id`, `observation_id` и `payload`.
- Повторная запись для того же `(account_id, record_kind, provider_record_id)` возвращает **тот же** `raw_record_id`, `observation_id` и `payload` (идемпотентность).
- В базе после двух записей остаётся ровно одна строка в `communication_raw_records`.
- Для каждого нового raw-источника создаётся наблюдение (`observations`) с типом `COMMUNICATION_MESSAGE` и ссылкой `source_ref` в формате `communication://{account_id}/{record_kind}/{provider_record_id}`.
- Попытка обновить `payload` сырой записи через прямой SQL вызывает ошибку — таблица `communication_raw_records` работает в режиме **append-only**.

### Привязка секретов (`secret_bindings.rs`)

Тест `communication_ingestion_binds_provider_accounts_to_secret_refs_against_postgres` проверяет, что для аккаунтов Gmail, iCloud и IMAP можно завести соответствующие ссылки на секреты (`secret_ref`) и привязать их с нужной целью (`ProviderAccountSecretPurpose`). Полученные биндинги содержат корректные `secret_ref`.

Тест `communication_ingestion_scopes_secret_refs_by_provider_account_against_postgres` с несколькими аккаунтами одного и того же типа провайдера проверяет:

- Каждый аккаунт получает свой уникальный `secret_ref`.
- Разные аккаунты Gmail не пересекаются по `secret_ref`.
- Разные аккаунты iCloud не пересекаются по `secret_ref`.
- Разрешённое значение секрета из `InMemorySecretResolver` соответствует ожидаемому runtime-значению для каждого аккаунта.

### Чтение учётных данных (`credential_reader.rs`)

`ProviderCredentialReader` тестируется на:

- Успешное разрешение связанного секрета (`provider_credential_reader_resolves_bound_account_secret_against_postgres`) — проверяется полное совпадение `account_id`, `secret_purpose`, `secret_ref`, `secret_kind` и runtime-значения. Debug-вывод не раскрывает секрет.
- Ошибку при отсутствии привязки (`provider_credential_reader_reports_missing_binding_against_postgres`) — возвращается `ProviderCredentialError::MissingBinding` с правильными `account_id` и `secret_purpose`.
- Проброс ошибки резолвера (`provider_credential_reader_propagates_resolver_failures_against_postgres`) — когда хранилище секретов имеет неподдерживаемый `SecretStoreKind` (`OsKeychain`), возникает `ProviderCredentialError::SecretResolution` с `SecretResolutionError::UnsupportedStoreKind("os_keychain")`.
- Несовместимый тип секрета (`provider_credential_reader_rejects_incompatible_secret_kind_against_postgres`) — если запрошенный `secret_purpose` требует `OauthToken`, а в `secret_ref` записан `SecretKind::Password`, возвращается `ProviderCredentialError::IncompatibleSecretKind`.

## Архитектурные тесты

### Лимиты размера файлов

- `communication_ingestion_architecture.rs` и `consistency_contradiction_architecture.rs` содержат по одному тесту, проверяющему, что ни один файл тестов из соответствующей предметной области не превышает **700 строк** (`MAX_TEST_FILE_LINES`).

### Архитектура коммуникаций (`communications_architecture_target.rs`)

Тест `channel_providers_are_not_product_domains_or_user_routes` проверяет следующие ограничения (файл обрезан в контексте, но видны основные проверки):

- В бэкенде отсутствует домен `mail`; доменом коммуникаций является `communications`.
- Во фронтенде отсутствуют домены `telegram` и `whatsapp`.
- Модуль `backend/src/domains/mod.rs` не экспортирует `mail`, но экспортирует `communications`, и не содержит legacy runtime-модули почты.
- Интеграционный модуль `mail` экспортирует `accounts`, `sync`, `rfc822`, `send`, `imap_write`.
- `mail_background_sync` находится в `workflows`, а не в интеграциях.
- Код роутера не импортирует старый домен `mail`; отсутствуют legacy маршруты вида `/api/v1/telegram`, `/api/v1/whatsapp`, `/api/v1/email-accounts`.
- Маршруты настройки/runtime провайдеров находятся под `/api/v1/integrations/telegram`, `/api/v1/integrations/whatsapp`, `/api/v1/integrations/mail`.
- Отсутствуют провайдер-специфичные маршруты под `/api/v1/communications/{mail,telegram,whatsapp}`, а также устаревшие провайдерские маршруты `provider-conversations`, `provider-messages`, `provider-web-messages`.
- Маршруты бизнес-логики коммуникаций (`/api/v1/communications/messages`, `/api/v1/communications/search`) являются провайдер-нейтральными.
- Хендлеры Telegram-чатов и поиска учитывают фильтрацию `channel_kind` и не смешивают WhatsApp и Telegram данные.
- Во фронтенде маршруты `/telegram` и `/whatsapp` отсутствуют; ключи кэшей Telegram и WhatsApp живут в домене `communications`, а не в `integrations`.

Тест `app_messaging_handlers_are_thin` проверяет, что:

- В `backend/src/app/handlers/telegram` нет файлов реализации; сам фасад ссылается на `provider_runtime_handlers::telegram`.
- Хендлеры не вызывают напрямую `telegram_store()` / `whatsapp_store()` или клиентские модули интеграций.

(Файл обрезан, поэтому возможны дополнительные проверки, не отражённые в этом контексте.)

## Тесты конфигурации (`config.rs`)

Набор тестов покрывает `AppConfig`:

- **Значения по умолчанию**: HTTP-адрес `127.0.0.1:8080`, имя сервиса `makosh-backend`, отсутствие `database_url`, `local_api_secret`, `secret_vault_path`, `secret_vault_key`, `tdjson_path`; AI-провайдер `Ollama` с моделями `qwen3:4b` (чат) и `qwen3-embedding:4b` (эмбеддинги), таймаутом 120 секунд; планировщики Zoom включены по умолчанию.
- **Переопределение через `MAKOSH_HTTP_ADDR`, `DATABASE_URL`, `MAKOSH_LOCAL_API_SECRET`**.
- **Секретный vault**: флаги `MAKOSH_SECRET_VAULT_PATH` и `MAKOSH_SECRET_VAULT_KEY`; значение ключа не раскрывается в Debug.
- **Ollama-параметры**: `MAKOSH_OLLAMA_BASE_URL`, `_CHAT_MODEL`, `_EMBED_MODEL`, `_TIMEOUT_SECONDS`.
- **OmniRoute**: `MAKOSH_AI_PROVIDER=omniroute`, `MAKOSH_OMNIROUTE_BASE_URL`, `_CHAT_MODEL`, `_EMBED_MODEL`, `_TIMEOUT_SECONDS`, `_API_KEY`; ключ не раскрывается в Debug.
- **TDLib**: `MAKOSH_TDJSON_PATH`.
- **Telegram-учётка**: `MAKOSH_TELEGRAM_API_ID` и `_API_HASH`; хэш не раскрывается в Debug.
- **Тогглы Zoom-планировщиков**: `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`, `_RECORDING_SYNC_`, `_RETENTION_CLEANUP_` через `false` отключают соответствующие планировщики.
- **Валидация**: пустые или некорректные значения вызывают ошибки `ConfigError::InvalidHttpAddr`, `EmptyDatabaseUrl`, `EmptyLocalApiSecret`, `EmptySecretVaultPath`, `EmptySecretVaultKey`, `EmptyTdjsonPath`, `InvalidTelegramApiId`, `EmptyTelegramApiHash`, `InvalidAiProvider`, `EmptyOllama*`, `InvalidOllamaTimeout`, `EmptyOmniRoute*`, `InvalidOmniRouteTimeout`, `EmptyOmniRouteApiKey`.

## ConnectRPC-тесты (`communications_connectrpc.rs`)

Тест `communications_connect_api_requires_local_api_secret` проверяет, что запрос без корректного секрета получает `StatusCode::FORBIDDEN`, а запрос, сформированный хелпером `post_json`, — `StatusCode::OK`.

Тест `communications_connect_api_exposes_provider_neutral_queries_and_send` (файл обрезан, видна начальная часть) выполняет полный цикл операций через ConnectRPC:

- Создание провайдер-аккаунта, запись raw-сообщения, проекция сообщения через `project_raw_email_message`.
- Второе raw-сообщение (newsletter).
- Создание вложений (текстовый файл и PDF) через `seed_connectrpc_attachment`.
- Создание черновика (`CommunicationDraftStore`) и элемента outbox (`CommunicationOutboxStore`).
- Вызов `ListMessages` — проверяется наличие спроецированного `message_id` и `workflowState: "new"`.
- Вызов `GetMessage` — проверяется `subject` и `attachments[0].attachmentId`.
- `TransitionMessageWorkflowState` переводит в состояние `"reviewed"`, возвращает `previousState: "new"`.
- `TrashMessage` устанавливает `localState: "trash"`.
- `RestoreMessage` возвращает `localState: "active"`.
- `MarkMessageRead` — `markedRead: true`, `workflowState` сохраняется `"reviewed"`.
- `DeleteMessageFromProvider` — `deleted: true`, `localState: "trash"`.
- Повторное восстановление после удаления у провайдера возвращает `localState: "active"`.
- `BulkMessageAction` с параметрами `action: "trash"` и списком `message_ids` возвращает `action: "trash"` и `updatedCount: 1`.

(Файл обрезан, дальнейшие проверки не видны.)

## Тесты движка противоречий (`consistency_contradiction`)

### Ядро движка (`engine.rs`)

Тесты проверяют:

- `ConsistencyEngine::detect_claim_contradictions` находит прямое противоречие между принятым фактом (`AcceptedClaim`) и новым утверждением (`NewEvidenceClaim`). Атрибуты наблюдения включают `old_source_kind`, `new_source_kind`, `conflict_type="direct_contradiction"`, `confidence`, `severity=Medium`, `review_state=Suggested`.
- Нормализация значений (trim, lowercase) приводит к тому, что `" Active "` и `"active"` не считаются противоречием.
- `extract_evidence_claims` извлекает структурированные утверждения из текста вида `"Location: Madrid\nStatus = active"`, а также детерминированные утверждения на естественном языке (`"I am now in Madrid."` → `location=Madrid`, `"The project status is blocked"` → `status=blocked`).
- `detect_evidence_contradictions` обнаруживает противоречия между памятью и документом через извлечение claim'ов.

### Хранилище наблюдений (`observation_store.rs`)

- `ContradictionObservationStore::upsert` идемпотентен: повторный upsert возвращает тот же `observation_id`.
- Состояние ревью обновляется через `set_review_state` (поддерживаются `UserConfirmed`, ревьюер и комментарий).
- Валидация перед записью в БД: `confidence` должен быть в диапазоне `0.0..1.0`; значение `1.2` вызывает ошибку `"confidence must be between 0.0 and 1.0: 1.2"`.

### Обновление противоречий из различных источников

Тесты следующих файлов общим паттерном проверяют, что при наличии активного факта персоны (`person_facts`) и появлении нового evidence (сообщение, заметка встречи, транскрипт звонка, документ, Telegram/WhatsApp сообщение) refresh-операция создаёт наблюдение противоречия, **не перезаписывая** исходный факт в памяти.

- `refresh_message_document.rs`:
  - `contradiction_refresh_detects_message_claim_against_active_person_fact_without_overwriting_memory` — сообщение электронной почты.
  - `contradiction_refresh_detects_natural_language_message_claim_against_active_person_fact_without_overwriting_memory` — сообщение с естественным языком.
  - `contradiction_refresh_detects_document_claim_against_active_person_fact_without_overwriting_memory` — документ.
- `refresh_event_call.rs`:
  - Заметка встречи (`MeetingNoteStore`).
  - Транскрипт звонка (`NewCallTranscript`), привязанный к персоне через `person_identities`.
- `refresh_provider_messages.rs`:
  - Telegram-сообщение (через `seed_telegram_message`).
  - WhatsApp-сообщение (через `seed_whatsapp_message`).

Для всех refresh-тестов атрибуты наблюдения одинаковы: `conflict_type="direct_contradiction"`, `confidence=0.8`, `severity=Medium`, `old_source_kind=Memory`, `new_source_kind` зависит от типа источника, `metadata` содержит `detector: "structured_evidence_claim"`, а исходный факт остаётся неизменным (`value = "Berlin"`).

## Тесты контекстных пакетов (`context_packs.rs`)

- `context_pack_store_persists_derived_pack_with_explicit_sources_against_postgres` проверяет создание контекстного пакета (`ContextPackKind::Meeting`) с явными источниками (Observation, DomainEntity, Knowledge) через `upsert_with_sources`, а также чтение источников через `list_sources`.
- `context_pack_store_rejects_pack_without_sources_before_database_write` проверяет, что пакет без источников вызывает ошибку `ContextPackStoreError::MissingSources` до обращения к БД (используется ленивое подключение).

## Вспомогательная инфраструктура

- `support.rs` в каждом модуле предоставляет `unique_suffix()` на основе `SystemTime::now()`, а также фабрику подключения к тестовой БД через `testkit::context::TestContext`.
- Для тестов consistency реализованы `seed_message`, `seed_telegram_message` и `seed_whatsapp_message`, которые создают провайдер-аккаунты, сырые записи, диспатчат сигналы через `dispatch_telegram_raw_signal`/`dispatch_whatsapp_raw_signal` и проецируют сообщения через `consume_accepted_signal_event`.
```

# Source coverage / Покрытие источников

- `backend/tests/communication_ingestion/credential_reader.rs`
  - `ProviderCredentialReader` успешно разрешает связанный секрет; проверяются `account_id`, `secret_purpose`, `secret_ref`, `secret_kind`, runtime-значение; Debug-вывод не раскрывает секрет.
  - Отсутствующая привязка → `ProviderCredentialError::MissingBinding`.
  - Сбой резолвера при неподдерживаемом `SecretStoreKind` → `ProviderCredentialError::SecretResolution(SecretResolutionError::UnsupportedStoreKind(...))`.
  - Несовместимый `SecretKind` → `ProviderCredentialError::IncompatibleSecretKind`.

- `backend/tests/communication_ingestion/raw_records.rs`
  - Идемпотентность записи `NewRawCommunicationRecord` при одинаковом `(account_id, record_kind, provider_record_id)`.
  - В БД остаётся одна строка в `communication_raw_records`.
  - На каждую новую запись создаётся наблюдение с `kind_code = "COMMUNICATION_MESSAGE"` и `source_ref` в формате `communication://{account_id}/...`.
  - Прямое SQL-обновление `payload` запрещено (append-only).

- `backend/tests/communication_ingestion/secret_bindings.rs`
  - Привязка секретов к Gmail (OAuth), iCloud (IMAP), IMAP (IMAP) через `bind_provider_account_secret`.
  - Загрузка биндингов через `provider_account_secret_bindings`.
  - Уникальность `secret_ref` для разных аккаунтов одинакового типа провайдера.
  - Корректное разрешение runtime-значений через `InMemorySecretResolver`.

- `backend/tests/communication_ingestion/support.rs`
  - Импорты: `CommunicationIngestionStore`, `EmailProviderKind`, `NewProviderAccount`, `NewRawCommunicationRecord`, `ProviderAccountSecretPurpose`, `ProviderCredentialReader`, `ProviderCredentialError`, `InMemorySecretResolver`, `SecretKind`, `SecretReferenceStore`, `SecretResolutionError`, `SecretStoreKind`, `Database`, `json!`.
  - `test_database_url` и `connect_database` используют `TestContext`.
  - `unique_suffix` на основе `SystemTime`.

- `backend/tests/communication_ingestion_architecture.rs`
  - Лимит 700 строк на файл тестов `communication_ingestion`.
  - Рекурсивный обход `tests/` и проверка для файлов внутри `communication_ingestion/` и `communication_ingestion.rs`, `communication_ingestion_architecture.rs`.

- `backend/tests/communications_architecture_target.rs` (частично обрезан)
  - Проверка отсутствия домена `mail` и наличия `communications`.
  - Проверка отсутствия фронтенд-доменов `telegram` и `whatsapp`.
  - Ограничения на `backend/src/domains/mod.rs`: нет `mod mail`, есть `mod communications`, нет legacy runtime-модулей.
  - Интеграционный модуль `mail` экспортирует перечисленные подмодули.
  - `mail_background_sync` — workflow, не интеграция.
  - Код роутера не содержит legacy импортов `domains::mail` и legacy маршрутов `/api/v1/telegram`, `/api/v1/whatsapp`, `/api/v1/email-accounts`.
  - Маршруты интеграций: `/api/v1/integrations/telegram`, `/api/v1/integrations/whatsapp`, `/api/v1/integrations/mail`.
  - Отсутствие WhatsApp message рута под интеграциями.
  - Провайдер-нейтральные маршруты `/api/v1/communications/messages`, `/api/v1/communications/search`.
  - Фильтрация `channel_kind` в хендлерах Telegram.
  - Фронтенд-роутер не содержит `/telegram`, `/whatsapp`.
  - Ключи кэшей Telegram/WhatsApp живут в домене `communications`.
  - `app_messaging_handlers_are_thin` проверяет структуру хендлеров и запрещает прямые вызовы store/клиентских модулей.

- `backend/tests/communications_connectrpc.rs` (частично обрезан)
  - Аутентификация через `MAKOSH_LOCAL_API_SECRET`.
  - End-to-end тест операций ConnectRPC: `ListMessages`, `GetMessage`, `TransitionMessageWorkflowState`, `TrashMessage`, `RestoreMessage`, `MarkMessageRead`, `DeleteMessageFromProvider`, `BulkMessageAction`, с проверкой возвращаемых состояний и полей.

- `backend/tests/config.rs`
  - Значения по умолчанию: HTTP addr, service name, database URL, vault, Ollama/Qwen модели.
  - Переопределение через env-пары.
  - Секретный vault: путь и ключ, сокрытие в Debug.
  - Ollama и OmniRoute параметры (chat/embed model, таймаут), OmniRoute API key.
  - TDLib path.
  - Telegram API ID и hash (сокрытие).
  - Тогглы Zoom-планировщиков.
  - Валидация некорректных значений (все перечисленные `ConfigError` варианты).

- `backend/tests/consistency_contradiction.rs`
  - Служит индексным файлом, ссылающимся на inlined-модули.

- `backend/tests/consistency_contradiction/engine.rs`
  - `detect_claim_contradictions` детектит прямое противоречие.
  - Нормализация убирает пробелы и регистр.
  - `extract_evidence_claims` из структурированного и естественного текста.
  - `detect_evidence_contradictions` работает с документом.

- `backend/tests/consistency_contradiction/observation_store.rs`
  - Идемпотентный `upsert`.
  - Обновление состояния ревью через `set_review_state`.
  - Валидация `confidence` (домен 0.0–1.0) до записи в БД.

- `backend/tests/consistency_contradiction/refresh_event_call.rs`
  - Refresh находит противоречие между meeting note и фактом персоны; исходный факт не перезаписан.
  - Refresh находит противоречие между call transcript и фактом персоны; исходный факт не перезаписан.

- `backend/tests/consistency_contradiction/refresh_message_document.rs`
  - Refresh находит противоречие между email-сообщением (структурированный и естественный язык) и фактом персоны; факт не перезаписан.
  - Refresh находит противоречие между документом и фактом персоны; факт не перезаписан.

- `backend/tests/consistency_contradiction/refresh_provider_messages.rs`
  - Refresh для Telegram-сообщений.
  - Refresh для WhatsApp-сообщений.

- `backend/tests/consistency_contradiction/support.rs`
  - `live_consistency_pool` создаёт пул через `TestContext`.
  - `seed_message` создаёт провайдер-аккаунт и проекцию email-сообщения.
  - `seed_telegram_message` и `seed_whatsapp_message` создают аккаунт, raw-запись, диспатчат сигнал и проецируют сообщение.

- `backend/tests/consistency_contradiction_architecture.rs`
  - Лимит 700 строк для файлов consistency contradiction.

- `backend/tests/context_packs.rs`
  - `ContextPackStore::upsert_with_sources` сохраняет пакет с источниками (Observation, DomainEntity, Knowledge).
  - `list_sources` возвращает ожидаемые источники.
  - `upsert_with_sources` без источников → `ContextPackStoreError::MissingSources` на ленивом пуле.

# Drift candidates / Кандидаты на drift

Из предоставленного контекста не видны расхождения между кодом тестов, документацией и ADR, поскольку в чанк не встроены ADR-файлы, существующая wiki-страница `operations/backend-tests.md` (или её предыдущая версия) или другие документы. Нет явных противоречий между поведением, описанным в тестах, и кодом бэкенда, так как код бэкенда также не встроен, за исключением тестов. Утверждения о поведении взяты напрямую из ассертов тестовых файлов.
