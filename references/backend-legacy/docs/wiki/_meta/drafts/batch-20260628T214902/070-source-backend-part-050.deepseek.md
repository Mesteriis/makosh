### Summary / Резюме

Обновить страницу `components/backend.md` русской вики: описать две ключевые платформенные подсистемы backend — конфигурацию (`platform/config`) и событийную архитектуру (`platform/events`). Предоставленные исходные файлы раскрывают детали реализации, которые необходимо зафиксировать в документации.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Backend-компоненты

Backend представляет собой серверное приложение, в котором ключевые платформенные подсистемы включают конфигурацию и событийно-ориентированную архитектуру (event sourcing). Здесь описаны модули `platform/config` и `platform/events`.

## Конфигурация (`platform/config`)

### Точка входа

При запуске вызывается `AppConfig::from_env()`, которая читает все переменные окружения через `env::vars()` и передаёт их в `from_pairs` (`backend/src/platform/config/app_config/env.rs`).

`from_pairs` инициализирует конфигурацию по умолчанию (`AppConfig::default()`), применяет встроенный Google OAuth клиент (`apply_bundled_google_oauth_client`) и затем для каждой пары ключ-значение вызывает диспетчер `apply_config_pair`.

### Применение переменных окружения

Функция `apply_config_pair` последовательно передаёт пару в обработчики:

- `apply_core_env` — основные настройки (HTTP-адрес, провайдер AI, секреты и т.д.)
- `apply_provider_env` — настройки провайдеров (Telegram, Google OAuth, Zoom scheduler)
- `apply_ai_env` — настройки AI-моделей (Ollama, OmniRoute)

Если ни один обработчик не распознал ключ, он игнорируется.

Такое проектирование позволяет расширять конфигурацию независимыми модулями.

### Переменные окружения провайдеров

В модуле `provider_env` (`backend/src/platform/config/app_config/provider_env.rs`) определены следующие переменные:

- `MAKOSH_TDJSON_PATH` — путь к библиотеке tdjson (Telegram).
- `MAKOSH_TELEGRAM_API_ID` — API ID Telegram (парсится как `i64`, должен быть > 0).
- `MAKOSH_TELEGRAM_API_HASH` — секрет Telegram (хранится как `ResolvedSecret`).
- `MAKOSH_GOOGLE_OAUTH_CLIENT_ID` и `MAKOSH_GOOGLE_OAUTH_CLIENT_SECRET` — учётные данные Google OAuth.
- `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_JSON` — JSON-конфигурация клиента в виде строки.
- `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` — путь к файлу с JSON-конфигурацией.
- `MAKOSH_ZOOM_TOKEN_MAINTENANCE_SCHEDULER_ENABLED`, `MAKOSH_ZOOM_RECORDING_SYNC_SCHEDULER_ENABLED`, `MAKOSH_ZOOM_RETENTION_CLEANUP_SCHEDULER_ENABLED` — булевы флаги для планировщиков Zoom.

Существует встроенный JSON клиента Google OAuth, задаваемый через константу времени компиляции `MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON` и применяемый функцией `apply_bundled_google_oauth_client`.

Булевы переменные парсятся с помощью `parse_bool_env` (`backend/src/platform/config/parsing.rs`), которая принимает `"true"`, `"1"`, `"yes"`, `"on"` как истину, и `"false"`, `"0"`, `"no"`, `"off"` как ложь.

### Константы по умолчанию

Файл `constants.rs` (`backend/src/platform/config/constants.rs`) задаёт:

- HTTP-адрес: `127.0.0.1:8080`
- Имя сервиса: `makosh-backend`
- Ollama: базовый URL `http://127.0.0.1:11434`, чат-модель `qwen3:4b`, модель эмбеддингов `qwen3-embedding:4b`, таймаут 120 сек.
- OmniRoute: базовый URL `https://ai.sh-inc.ru/v1`, чат-модель `codex/gpt-5.5`, модель эмбеддингов `openai-compatible-chat-ollama-pve/qwen3-embedding:4b`, таймаут 120 сек.

### Обработка ошибок конфигурации

Перечисление `ConfigError` (`backend/src/platform/config/errors.rs`) содержит варианты для некорректных значений (`InvalidHttpAddr`, `InvalidAiProvider`, `InvalidBoolEnv`), пустых обязательных полей (`EmptyDatabaseUrl`, `EmptyTelegramApiHash`, `EmptyOmniRouteApiKey` и др.), а также ошибки ввода-вывода (например, `GoogleOAuthClientConfigRead`).

### Google OAuth клиент

`GoogleOAuthClientConfig` (`backend/src/platform/config/google.rs`) создаётся из JSON в формате секретов Google Cloud. Поддерживаются типы клиента `Installed` и `Web`. Распарсенные поля: `client_id`, `client_secret` (опционально, хранится как `ResolvedSecret`), `authorization_endpoint`, `token_endpoint`, `redirect_uris`. Валидация полей выполняется с помощью `required_trimmed` из модуля `parsing`.

## События (`platform/events`)

Подсистема реализует паттерн event sourcing с append-only журналом, outbox-диспетчером, потребителями (consumers) и проекциями. Публичное API модуля экспортирует основные структуры (`backend/src/platform/events.rs`).

### Модели событий

- `NewEventEnvelope` — событие до записи в журнал.
- `EventEnvelope` — записанное событие с полем `recorded_at`.
- `StoredEventEnvelope` — запись журнала с монотонно возрастающим `position`.
- `EventOutboxItem` / `DispatchableEventOutboxItem` — элементы outbox-таблицы для надёжной публикации.

Сборка `NewEventEnvelope` выполняется через `NewEventEnvelopeBuilder` (`backend/src/platform/events/builder.rs`), который проверяет:
- `event_id` и `event_type` непустые,
- `schema_version` > 0,
- поля `source`, `subject`, `payload`, `provenance` (и `actor`, если задан) являются JSON-объектами.

При отсутствии `correlation_id` он автоматически устанавливается равным `event_id`.

### Шина событий

- `InMemoryEventBus` (`backend/src/platform/events/bus.rs`) — in-process шина на основе `tokio::sync::broadcast` с кольцевым буфером ёмкостью 4096 событий. Поддерживает подписку, подсчёт подписчиков и метод `broadcast_stored`, преобразующий `EventEnvelope` в `NewEventEnvelope`.
- `NatsJetStreamEventBus` (`backend/src/platform/events/nats.rs`) — публикует события в NATS JetStream в стрим `makosh_events` с субъектами по шаблону `signal.>`. Конкретный субъект формируется из `event_type` события.

### Типы событий

Модуль `bus.rs` определяет константы для типов событий Telegram, WhatsApp, Zoom и Yandex Telemost. Примеры: `telegram.message.created`, `whatsapp.sync.started`, `zoom.recording.observed`, `integration.yandex_telemost.conference.created`.

Функция `sanitize_event_payload` удаляет из JSON-объекта чувствительные ключи: `raw_body`, `tdlib_raw`, `access_token`, `api_hash`, `session_key`, `bot_token`, `proxy_password`, `password`.

### Хранилище событий (`EventStore`)

`EventStore` (`backend/src/platform/events/store.rs`) инкапсулирует пул соединений PostgreSQL и предоставляет методы:

- Добавления событий: `append`, `append_idempotent`, `append_in_transaction`, `append_for_dispatch`, `append_for_dispatch_idempotent`.
- Чтения: `get_by_id`, `list_matching` (с гибкой фильтрацией через `EventLogQuery`), `list_after_position`.
- Трассировки: `list_by_correlation_id`, `list_children`, `trace_by_event_id`, `trace_by_correlation_id`.

Метод `append_for_dispatch` помимо вставки в `event_log` создаёт запись в `event_outbox` со статусом `'pending'`, гарантируя последующую отправку через диспетчер.

### Outbox-диспетчер

`EventOutboxDispatcher` (`backend/src/platform/events/dispatcher.rs`) координирует надёжную публикацию событий:

- Метод `dispatch_pending_once` восстанавливает "зависшие" (stale) элементы outbox и выбирает пакет элементов с локом `FOR UPDATE SKIP LOCKED`.
- Для каждого элемента публикует событие через `NatsJetStreamEventBus::publish`, при успехе помечает как `published`, при неудаче планирует повторную попытку с экспоненциальной задержкой (до 300 секунд).
- Опционально дублирует событие в `InMemoryEventBus` (realtime bus), если он передан через `with_realtime_bus`.

### Потребители событий (consumers)

`EventConsumerRunner` (`backend/src/platform/events/consumers.rs`) — универсальный механизм обработки событий с поддержкой:

- позиционного курсора (таблица `event_consumers`),
- повторных попыток с экспоненциальной задержкой (`retry_base_seconds`),
- dead letter queue при превышении `max_attempts` (по умолчанию 5),
- ручного повтора dead-сообщений через `replay_dead_letter` (только в состоянии `ReplayRequested`).

Конфигурация потребителя задаётся через `EventConsumerConfig` (имя, размер пакета, max_attempts, retry_base_seconds). `EventConsumerStore` управляет служебными таблицами `event_consumers` и `event_consumer_failures`.

### Курсоры проекций

`ProjectionCursorStore` (`backend/src/platform/events/cursors.rs`) аналогичен потребительским курсорам, но используется для проекций. Поддерживает позиционное отслеживание и ручную перемотку (`rewind_position`).

### Состояния выполнения (runtime)

Функции из `runtime.rs` (`backend/src/platform/events/runtime.rs`) управляют состоянием обработки на основе сигнальных политик:

- `ensure_runtime_processing_state` создаёт запись в `signal_runtime_states`, если её нет.
- `source_runtime_state_from_policies` проверяет активные политики (`signal_policies`): режим `disabled` приводит к `stopped`, `paused` — к `paused`, `muted` — к `muted`; иначе — `running`.
- `runtime_allows_processing` возвращает `true` только для состояний `running`, `starting`, `reconnecting`.

### Миграции

Миграции выполняются через `run_migrations` (`backend/src/platform/events/migrations.rs`), компилирующую SQL-файлы из папки `./migrations`. `expected_migration_summary` возвращает `MigrationSummary` (количество миграций и последняя версия).

### Трассировка событий

`EventStore` строит трассу по `correlation_id` или `event_id` (`backend/src/platform/events/trace.rs`). Результат — `EventTrace`, содержащий:

- полный граф событий,
- рёбра причинности (`edges`),
- корневые и "осиротевшие" события (causation_id отсутствует или ссылается на отсутствующее событие),
- аннотации потребителей (`EventConsumerAnnotation`) из таблиц `event_consumer_processed_events` и `event_consumer_failures`,
- аннотации dead-писем (`EventDeadLetterAnnotation`) из `event_dead_letters`.

### Обработка ошибок в событиях

Подсистема использует два основных типа ошибок (`backend/src/platform/events/errors.rs`):

- `EventEnvelopeError` — ошибки валидации оболочки события (`EmptyField`, `InvalidSchemaVersion`, `NonObjectJson`).
- `EventStoreError` — ошибки хранилища: оборачивает ошибки `sqlx::Error`, `MigrateError`, `serde_json::Error`, а также содержит специфичные варианты (`DeadLetterNotFound`, `DeadLetterNotReplayRequested`, `ConsumerHandlerFailed`).

Метод `is_unique_violation` проверяет, является ли ошибка нарушением уникальности (код PostgreSQL `23505`).
```

### Source coverage / Покрытие источников

- `backend/src/platform/config/app_config/env.rs` — `from_env`, `from_pairs`, `apply_config_pair`.
- `backend/src/platform/config/app_config/provider_env.rs` — переменные провайдеров, встроенный Google OAuth, парсинг Telegram API ID, булевы флаги Zoom, `non_empty`.
- `backend/src/platform/config/constants.rs` — все значения констант по умолчанию.
- `backend/src/platform/config/errors.rs` — перечисление `ConfigError` и отдельные варианты.
- `backend/src/platform/config/google.rs` — `GoogleOAuthClientConfig`, типы клиента, поля, разбор JSON.
- `backend/src/platform/config/parsing.rs` — `parse_bool_env`, `required_trimmed`.
- `backend/src/platform/events.rs` — публичный API модуля.
- `backend/src/platform/events/builder.rs` — `NewEventEnvelopeBuilder`, правила валидации.
- `backend/src/platform/events/bus.rs` — `InMemoryEventBus`, константы событий для провайдеров, `sanitize_event_payload`.
- `backend/src/platform/events/consumers.rs` (частично) — `EventConsumerConfig`, `EventConsumerRunner`, `EventConsumerStore`, логика потребления, dead letter.
- `backend/src/platform/events/cursors.rs` — `ProjectionCursorStore`.
- `backend/src/platform/events/dispatcher.rs` — `EventOutboxDispatcher`, экспоненциальная задержка.
- `backend/src/platform/events/errors.rs` — `EventEnvelopeError`, `EventStoreError`, `is_unique_violation`.
- `backend/src/platform/events/migrations.rs` — `run_migrations`, `expected_migration_summary`.
- `backend/src/platform/events/models.rs` — модели `NewEventEnvelope`, `EventEnvelope`, `StoredEventEnvelope`, outbox-модели.
- `backend/src/platform/events/nats.rs` — `NatsJetStreamEventBus`, стрим `makosh_events`, субъект `signal.>`.
- `backend/src/platform/events/query.rs` — `EventLogQuery` и поля фильтрации.
- `backend/src/platform/events/rows.rs` — упомянуты как служебные (фактически покрыты через `EventStore`).
- `backend/src/platform/events/runtime.rs` — состояния выполнения и политики.
- `backend/src/platform/events/store.rs` — структура `EventStore`.
- `backend/src/platform/events/store/append.rs` — методы append.
- `backend/src/platform/events/store/outbox.rs` — outbox-операции, `claim_pending_outbox_batch`, `recover_stale_outbox_items`.
- `backend/src/platform/events/store/read.rs` — `get_by_id`, `list_matching`.
- `backend/src/platform/events/store/replay.rs` — `list_after_position`.
- `backend/src/platform/events/trace.rs` — `EventTrace`, аннотации, методы трассировки.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста не видно расхождений между кодом, документацией или ADR. Предыдущая версия страницы `components/backend.md` не включена в чанк, а сам код не содержит комментариев о drift. Дополнительные проверки на основе текущих данных не требуются.
