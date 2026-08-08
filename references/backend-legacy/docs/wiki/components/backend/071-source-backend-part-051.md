---
chunk_id: 071-source-backend-part-051
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 071-source-backend-part-051 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создать или обновить страницу `components/backend.md` в русской wiki-документации. На основе предоставленных исходников платформы бэкенда описать ключевые модули: события, форматирование, граф, наблюдения, проекции, realtime-звонки, секреты. Страница даёт обзор сервисов платформы и их основных API/структур.

## Предложенные страницы

- `components/backend.md`

```markdown
# Backend (Платформа)

## Обзор

Бэкенд-платформа `makosh` (`backend/src/platform`) содержит модули, реализующие сквозную функциональность приложения:

- **События (`events`)** – конверты событий, контекст трассировки, валидация.
- **Форматирование (`formatting`)** – утилиты для текстовых превью.
- **Граф (`graph`)** – модель узлов графа и генерация идентификаторов.
- **Наблюдения (`observations`)** – захват, хранение и связывание наблюдений с доменными сущностями.
- **Проекции (`projections`)** – пакетная обработка событий для построения проекций.
- **Realtime-звонки (`realtime_conversation`)** – модели бандлов звонков, провайдеры, события.
- **Секреты (`secrets`)** – шифрованные хранилища, ссылки на секреты, резолверы.
- **(Не описанные в данном чанке)** `ai_runtime`, `audit`, `calls`, `capabilities`, `communications`, `config`, `settings`, `storage` – перечислены в `platform/mod.rs`, исходный код этих модулей не включён в контекст.

## События (`events`)

### Контекст трассировки (`TraceContext`)

Структура `TraceContext` содержит:

- `correlation_id: String` – идентификатор корреляции;
- `causation_id: Option<String>` – идентификатор причинного события (опционально).

Методы:

- `TraceContext::root(root_id)` – создаёт корневой контекст (без `causation_id`).
- `TraceContext::child_of(parent: &EventEnvelope)` – порождает дочерний контекст от переданного конверта: `correlation_id` берётся из `parent.correlation_id` (если есть) или из `parent.event_id`; `causation_id` становится `parent.event_id`.
- `TraceContext::child_of_stored(parent: &StoredEventEnvelope)` – аналог для хранимого конверта (делегирует к `child_of`).
- `apply(self, builder: NewEventEnvelopeBuilder)` – применяет контекст к строителю конверта: устанавливает `correlation_id` и, при наличии, `causation_id`.

### Валидация событий

Внутренние (`pub(super)`) функции:

- `validate_non_empty(field_name, value)` – возвращает ошибку, если значение после `trim` пустое.
- `validate_object(field_name, value: &Value)` – возвращает ошибку, если JSON-значение не является объектом.

Обе возвращают `EventEnvelopeError`.

## Форматирование (`formatting`)

```rust
pub(crate) fn text_preview(value: &str, max_chars: usize) -> String
```

Обрезает строку до `max_chars` символов и добавляет `"…"`, если исходная длиннее. Доступна только внутри крейта.

## Граф (`graph`)

### Типы узлов (`GraphNodeKind`)

Перечисление:

- `Person`
- `EmailAddress`
- `Message`
- `Document`
- `Project`
- `Organization`
- `Task`
- `Event`
- `Decision`
- `Obligation`
- `Knowledge`

Метод `as_str()` возвращает snake_case-строковое представление (например `"email_address"`).

### Идентификаторы узлов

```rust
pub fn node_id(kind: GraphNodeKind, stable_key: &str) -> String
```

Формат: `graph:node:v1:{kind}:{stable_key}`, где `kind` – строковое представление типа узла.

## Наблюдения (`observations`)

Публичный API модуля реэкспортирует:

- `ObservationStoreError` (он же `ObservationPortError`) – перечисление ошибок хранилища.
- Модели: `NewObservation`, `Observation`, `ObservationKindDefinition`, `NewObservationLink`, `ObservationLink`, `NewObservationIngestionRun`, `ObservationIngestionRun`, `ObservationIngestionRunStatus`, `ObservationOriginKind`.
- `ObservationStore` (тип-алиас `ObservationPort`).
- Внутренние функции связывания `link_domain_entity`, `materialize_review_transition_link` (доступны для крейта).

### Модели данных

#### `ObservationOriginKind`

Варианты:

- `VaultSource`
- `Manual`
- `BrowserCapture`
- `VoiceMemo`
- `FileImport`
- `LocalRuntime`
- `TestFixture`

Методы:

- `as_str()` – строковое представление.
- `parse(value)` – разбор из строки; неизвестная строка дает ошибку `UnknownOriginKind`.

#### `Observation`

Основные поля: `observation_id`, `kind_definition_id`, `kind_code`, `origin_kind`, `vault_source_id`, `observed_at`, `captured_at`, `payload` (JSON `Value`), `confidence` (f64), `content_hash`, `source_ref`, `provenance`.

#### `NewObservation`

Используется для создания наблюдения. Поля: `kind_code`, `origin_kind`, `vault_source_id`, `observed_at`, `payload`, `confidence`, `source_ref`, `provenance`.

Конструктор `new(kind_code, origin_kind, observed_at, payload, source_ref)` задает значения по умолчанию: `vault_source_id = None`, `confidence = 1.0`, `provenance = json!({})`.

Методы-строители: `vault_source_id(id)`, `confidence(value)`, `provenance(value)`.

Валидация (`validate`):

- `kind_code`, `source_ref` – непустые.
- Если задан `vault_source_id` – непустой.
- `payload` и `provenance` – JSON-объекты.
- `confidence` ∈ [0.0, 1.0].

Ошибки: `ObservationStoreError` (EmptyField, InvalidJsonObject, InvalidScore).

#### `ObservationKindDefinition`

Поля: `kind_definition_id`, `code`, `name`, `version`, `category`, `description`, `created_at`, `updated_at`.

#### `ObservationLink`

Поля: `observation_id`, `domain`, `entity_kind`, `entity_id`, `relationship_kind`, `confidence`, `metadata`, `created_at`.

#### `NewObservationLink`

Конструктор `new(observation_id, domain, entity_kind, entity_id)` задает `relationship_kind = "evidence_for"`, `confidence = 1.0`, `metadata = {}`.

Методы-строители: `relationship_kind(kind)`, `confidence(value)`, `metadata(value)`.

Валидация: `observation_id`, `domain`, `entity_kind`, `entity_id`, `relationship_kind` – непустые; `metadata` – JSON-объект; `confidence` ∈ [0.0, 1.0].

#### `ObservationIngestionRunStatus`

Варианты: `Running`, `Succeeded`, `Failed`, `Skipped`. Методы `as_str()` и `parse(value)`.

#### `ObservationIngestionRun`

Поля: `ingestion_run_id`, `observation_id`, `pipeline`, `status`, `started_at`, `finished_at` (optional), `output` (Value), `error_message` (optional).

#### `NewObservationIngestionRun`

Конструктор `new(ingestion_run_id, observation_id, pipeline)`. Валидация: все три идентификатора непустые.

### Хранилище (`ObservationStore`)

Пул соединений с PostgreSQL (`PgPool`).

Основные методы:

- `capture(observation: &NewObservation)` – валидирует, в транзакции ищет `kind_definition_id` по `code` и версии 1, вычисляет `content_hash` (SHA-256), генерирует `observation_id`. При вставке используется `ON CONFLICT (observation_id) DO NOTHING`. После успешной вставки вызывает `append_observation_captured_event`. Возвращает `Observation`.
- `get(observation_id)` – получает одно наблюдение.
- `list_kind_definitions()` – список определений видов наблюдений.
- `upsert_link(link)` – вставка или обновление связи наблюдения с доменной сущностью (UPSERT по `(observation_id, domain, entity_kind, entity_id, relationship_kind)`).
- `list_links(observation_id)` – список связей.
- `start_ingestion_run(run)` – создание запуска пайплайна обработки.
- `finish_ingestion_run(ingestion_run_id, status, output, error_message)` – завершение запуска.
- `list_ingestion_runs(observation_id)` – список запусков.

### Связи обзора (`review_links`)

Внутренние функции (`pub(crate)`) для материализации связей:

- `materialize_review_transition_link(pool, observation_id, domain, entity_kind, entity_id, state_field, state_value, metadata)` – создает связь с `relationship_kind = "review_transition"` и метаданными `{ state_field: state_value }`. Если `observation_id` пуст, связь не создается.
- `link_domain_entity(pool, observation_id, domain, entity_kind, entity_id, relationship_kind, confidence, metadata)` – создает связь с доменной сущностью. По умолчанию используется `relationship_kind = "evidence_for"`. При пустом `observation_id` возвращает ошибку `EmptyField`.

Оба метода имеют версии `_in_transaction`, работающие в переданной транзакции.

## Проекции (`projections`)

```rust
pub async fn run_projection_batch<F, Fut>(
    events: &EventStore,
    cursors: &ProjectionCursorStore,
    projection_name: &str,
    batch_size: u32,
    handler: F,
) -> Result<ProjectionBatchOutcome, ProjectionRunnerError>
```

Функция для пакетной обработки событий:

- Проверяет, что `batch_size > 0`.
- Получает начальную позицию из `ProjectionCursorStore` (`last_processed_position`).
- Запрашивает батч событий после этой позиции из `EventStore`.
- Обрабатывает каждое событие пользовательским обработчиком.
- После успешной обработки события сохраняет его позицию как последнюю обработанную через `cursors.save_position`.
- Возвращает `ProjectionBatchOutcome` с количеством обработанных событий и последней позицией.

Ошибки:

- `ProjectionHandlerError` – обёртка строки сообщения.
- `ProjectionRunnerError` – варианты: `EventStore` (ошибка хранилища событий), `Handler` (ошибка обработчика), `InvalidBatchSize`.

## Realtime-звонки (`realtime_conversation`)

### Виды провайдеров (`RealtimeConversationProviderKind`)

Варианты:

- `YandexTelemost`
- `Zoom`
- `GoogleMeet`
- `Jitsi`
- `Discord`
- `SignalCalls`
- `Unknown`

Метод `as_str()` возвращает snake_case-представление (например `"yandex_telemost"`).

### Возможности провайдеров

Структура `RealtimeConversationProviderCapabilities` описывает поддерживаемые функции:

- `supports_conference_create`
- `supports_visible_webview`
- `supports_audio_capture`
- `supports_participant_events`
- `supports_speaker_hints`
- `supports_chat_capture`
- `supports_screen_share_detection`
- `supports_screenshot_hints`
- `supports_recording`
- `supports_provider_transcript`
- `supports_reactions`
- `evidence` – JSON-поле с информацией об источнике.

Фабричные функции:

- `yandex_telemost_provider_capabilities()` – YandexTelemost: создание конференций, видимый webview, аудиозахват, подсказки спикеров, скриншоты, запись (без расшифровки, чата, реакций).
- `zoom_provider_capabilities()` – Zoom: поддерживаются практически все возможности, включая события участников, чат, скриншейр, провайдерскую расшифровку, реакции.
- `generic_webview_provider_capabilities(provider_kind, provider_shape)` – минимальные возможности для любого провайдера: только видимый webview, аудиозахват, скриншоты и запись.

Трейт `RealtimeConversationProvider` объявляет методы `provider_kind()`, `provider_shape()`, `capabilities()`.

### Бандл звонка

Функция `default_call_bundle_layout(root)` возвращает `CallBundleLayout` с фиксированными именами файлов (например `manifest.json`, `audio.mp3`, `transcript.json` и многие другие).

Функция `build_call_bundle_manifest(...)` формирует `CallBundleManifest` со схемой версии 1, информацией о провайдере, артефактами (аудио, подсказки спикеров, трек событий), состоянием пайплайна (`queued_from_local_recording`) и политикой приватности (`local_visible_capture`).

### Состояние пайплайна (`CallBundlePipelineState`)

Поля, описывающие статус каждого этапа: `audio_capture`, `speaker_hints`, `transcription`, `diarization`, `speaker_identity`, `topic_timeline`, `decision_detection`, `action_detection`, `screen_intelligence`, `knowledge_extraction`, `radar_projection`.

`CallBundlePipelineState::queued_from_local_recording()` задаёт состояния: аудиозахват – `"running_or_completed"`, подсказки – `"collecting_hint_not_truth"`, остальные – `"queued"`.

### Политика приватности (`CallBundlePrivacyPolicy`)

`CallBundlePrivacyPolicy::local_visible_capture()` устанавливает:

- `owner_visible_capture_only: true`
- `hidden_headless_capture: "forbidden"`
- `consent_required: true`
- `local_first: true`
- `provider_dom_truth_status: "hint_not_truth"`

### События звонка

Модуль `events` определяет константы-строки для событий:

- `realtime_conversation.session.opened` / `.ended`
- `realtime_conversation.call_bundle.created`
- `realtime_conversation.audio_capture.started` / `.completed`
- `realtime_conversation.speaker_hint.observed`
- `realtime_conversation.event_track.observed`
- `realtime_conversation.screenshot_hint.captured`
- `realtime_conversation.transcript.requested` / `.completed`
- `realtime_conversation.knowledge.extracted`
- `realtime_conversation.radar_signals.detected`

## Секреты (`secrets`)

### Модели секретов

#### Тип секрета (`SecretKind`)

`OauthToken`, `AppPassword`, `Password`, `ApiToken`, `PrivateKey`, `Other`.

#### Тип хранилища (`SecretStoreKind`)

`OsKeychain`, `EncryptedVault`, `DatabaseEncryptedVault`, `HostVault`, `ExternalVault`, `TestDouble`.

Оба типа имеют `as_str()` и реализуют `TryFrom<&str>` с соответствующими ошибками.

#### Ссылка на секрет (`SecretReference`)

Поля: `secret_ref`, `secret_kind`, `store_kind`, `label`, `metadata` (JSON), `created_at`, `updated_at`.

`NewSecretReference` – конструктор `new(secret_ref, secret_kind, store_kind, label)`, метод `metadata()`.

#### Разрешённый секрет (`ResolvedSecret`)

Содержит значение `value: String`. Создается через `ResolvedSecret::new(value)` с проверкой непустоты. Метод `expose_for_runtime()` возвращает `&str`. `Debug` выводит значение как `<redacted>`.

### Хранилище ссылок (`SecretReferenceStore`)

Работает с таблицей `secret_references`. Методы:

- `upsert_secret_reference(reference)` – вставка или обновление записи (UPSERT по `secret_ref`).
- `secret_reference(secret_ref)` – получение одной ссылки.
- `delete_secret_reference(secret_ref)` – удаление.

### Шифрованные хранилища секретов

#### Файловое хранилище (`EncryptedSecretVault`)

Путь к файлу по умолчанию: `~/.config/makosh/secrets.vault.json`.

Формат файла (JSON):

- `version: u8` (текущая `1`)
- `kdf: "argon2id:v1"`
- `salt: String` (base64)
- `entries: BTreeMap<String, EncryptedVaultEntry>` – ключ `secret_ref`, значение `{ nonce, ciphertext }`.

Шифрование: AES-256-GCM, ключ выводится из мастер-ключа (`ResolvedSecret`) и соли через Argon2id. AAD – `secret_ref`. Nonce генерируется случайно.

Метод `store_secret(secret_ref, value)` загружает или создаёт файл, шифрует значение, обновляет запись и атомарно записывает файл (через временный).

Метод `resolve_secret(reference)` проверяет `store_kind == EncryptedVault`, загружает файл, расшифровывает запись.

Реализует типаж `SecretResolver` (resolve возвращает `SecretResolutionFuture`).

Ошибки: `EncryptedVaultError` (Io, Json, UnsupportedVaultFormat, InvalidEncoding, Crypto, EmptyField). Публичное сообщение через `public_message()` не раскрывает детали (например "invalid vault key or corrupted encrypted vault").

#### База данных (`DatabaseEncryptedSecretVault`)

Аналогичное шифрование, но хранит записи в таблице `encrypted_secret_vault_entries`. Поля: `secret_ref`, `kdf`, `salt`, `nonce`, `ciphertext`, `updated_at`.

Метод `store_secret` использует `ON CONFLICT (secret_ref) DO UPDATE`. `resolve_secret` проверяет `store_kind == DatabaseEncryptedVault`, ищет запись, расшифровывает.

Реализует `SecretResolver`.

Ошибки: `DatabaseEncryptedVaultError` с публичным сообщением.

### Криптография (`crypto`)

Константы:

- `VAULT_VERSION: u8 = 1`
- `VAULT_KDF: &str = "argon2id:v1"`
- `SALT_LEN: usize = 16`
- `NONCE_LEN: usize = 12`

Функции:

- `random_bytes<const N: usize>()` – генерирует случайный массив через `OsRng`.
- `encrypted_vault_cipher(master_key, encoded_salt)` – возвращает `Aes256Gcm` (для файлового хранилища), ошибка `EncryptedVaultError`.
- `database_vault_cipher(master_key, encoded_salt)` – то же для базы данных, ошибка `DatabaseEncryptedVaultError`.

### Резолверы секретов

Трейт `SecretResolver`:

```rust
fn resolve<'a>(&'a self, reference: &'a SecretReference) -> SecretResolutionFuture<'a>;
```

Тип `SecretResolutionFuture<'a>` – `Pin<Box<dyn Future<Output = Result<ResolvedSecret, SecretResolutionError>> + Send + 'a>>`.

`InMemorySecretResolver` – реализация для `TestDouble` (`store_kind == TestDouble`). Хранит значения в `HashMap`. Метод `insert(secret_ref, value)`.

`SecretResolutionError` варианты: `EmptySecretRef`, `EmptySecretValue`, `MissingSecret`, `UnsupportedStoreKind`, `StoreFailure`.

## Прочие модули

В `platform/mod.rs` объявлены публичные модули, исходный код которых не включён в данный контекст:

- `ai_runtime`
- `audit`
- `calls`
- `capabilities`
- `communications`
- `config`
- `settings`
- `storage`

Детали их реализации не подтверждены данным чанком.
```

## Покрытие источников

- `backend/src/platform/events/trace_context.rs`
  - `TraceContext` struct, поля `correlation_id`, `causation_id`
  - Методы `root`, `child_of`, `child_of_stored`, `apply`
- `backend/src/platform/events/validation.rs`
  - Функции `validate_non_empty`, `validate_object` (возвращают `EventEnvelopeError`)
- `backend/src/platform/formatting.rs`
  - Функция `text_preview(value, max_chars)`, логика обрезки с "…"
- `backend/src/platform/graph.rs`
  - `GraphNodeKind` enum, все 11 вариантов, метод `as_str()`
  - Функция `node_id(kind, stable_key)`, формат идентификатора
- `backend/src/platform/observations/errors.rs`
  - `ObservationStoreError` enum, все варианты ошибок
- `backend/src/platform/observations/mod.rs`
  - Перечень публичного API модуля (реэкспорты)
- `backend/src/platform/observations/models.rs`
  - `ObservationOriginKind` (7 вариантов), методы `as_str`, `parse`
  - `ObservationKindDefinition` поля
  - `Observation` поля
  - `NewObservation` поля, конструктор `new`, методы-строители, `validate`
  - `ObservationLink` поля
  - `NewObservationLink` поля, конструктор `new`, методы-строители, `validate`
  - `ObservationIngestionRunStatus` (4 варианта), методы `as_str`, `parse`
  - `ObservationIngestionRun` поля
  - `NewObservationIngestionRun` поля, конструктор `new`, `validate`
  - Внутренние функции валидации `validate_non_empty`, `validate_json_object`, `validate_score`
- `backend/src/platform/observations/review_links.rs`
  - Функции `materialize_review_transition_link` (и `_in_transaction`), логика построения `NewObservationLink` с `relationship_kind = "review_transition"` и обработка пустого `observation_id`
  - Функции `link_domain_entity` (и `_in_transaction`), логика построения `NewObservationLink` с `relationship_kind` по умолчанию `"evidence_for"`, проверка обязательного `observation_id`
- `backend/src/platform/observations/store.rs` (первые 12000 символов)
  - `ObservationStore` (пул `PgPool`)
  - Метод `capture` (валидация, транзакция, поиск `kind_definition_id`, вычисление хеша, вставка, событие)
  - Методы `get`, `list_kind_definitions`, `upsert_link`, `list_links`, `start_ingestion_run`, `finish_ingestion_run`, `list_ingestion_runs`
  - Внутренний `capture_in_transaction` с логикой `ON CONFLICT DO NOTHING` и повторным SELECT
  - Вспомогательные SQL-функции
- `backend/src/platform/projections.rs`
  - `ProjectionBatchOutcome` struct
  - `run_projection_batch` (параметры, проверка `batch_size`, итерация событий, сохранение позиций курсора)
  - Ошибки `ProjectionHandlerError`, `ProjectionRunnerError`
- `backend/src/platform/realtime_conversation/bundle.rs`
  - `default_call_bundle_layout` (все поля `CallBundleLayout`)
  - `build_call_bundle_manifest` (создание `CallBundleManifest`, артефакты, состояние, приватность, провенанс)
- `backend/src/platform/realtime_conversation/events.rs`
  - Все константы событий (12 строк)
- `backend/src/platform/realtime_conversation/mod.rs`
  - Реэкспорты публичных элементов модуля
- `backend/src/platform/realtime_conversation/models.rs`
  - `RealtimeConversationProviderKind` enum, `as_str()`
  - `RealtimeConversationProviderCapabilities` struct, `evidence_source`
  - `CallBundleArtifact`, `CallBundleLayout`, `CallBundlePipelineState` (и `queued_from_local_recording`), `CallBundlePrivacyPolicy` (и `local_visible_capture`), `CallBundleManifest`
  - `SpeakerTimelineHint`, `MeetingTimelineEvent`, `TopicTimelineSegment`
- `backend/src/platform/realtime_conversation/provider.rs`
  - Трейт `RealtimeConversationProvider`
  - `yandex_telemost_provider_capabilities()`, `zoom_provider_capabilities()`, `generic_webview_provider_capabilities()` – перечни поддерживаемых функций
- `backend/src/platform/secrets/crypto.rs`
  - Константы `VAULT_VERSION`, `VAULT_KDF`, `SALT_LEN`, `NONCE_LEN`
  - Функции `random_bytes`, `encrypted_vault_cipher`, `database_vault_cipher`
- `backend/src/platform/secrets/database_vault.rs`
  - `DatabaseEncryptedSecretVault` (пул, мастер-ключ)
  - Метод `store_secret` (шифрование с AAD, вставка/обновление)
  - Метод `resolve_secret` (проверка `store_kind`, загрузка, расшифровка)
  - Реализация `SecretResolver`
  - Ошибка `DatabaseEncryptedVaultError` и `public_message()`
- `backend/src/platform/secrets/errors.rs`
  - `SecretReferenceError` enum (варианты)
  - `SecretResolutionError` enum (варианты `EmptySecretRef`, `EmptySecretValue`, `MissingSecret`, `UnsupportedStoreKind`, `StoreFailure`)
- `backend/src/platform/secrets/file_vault.rs`
  - `EncryptedSecretVault` (путь, мастер-ключ)
  - Метод `store_secret` (загрузка/создание файла, шифрование, атомарная запись)
  - Метод `resolve_secret` (проверка `store_kind`, загрузка, расшифровка)
  - Реализация `SecretResolver`
  - Структуры `EncryptedVaultFile`, `EncryptedVaultEntry`
  - Ошибка `EncryptedVaultError` и `public_message()`
- `backend/src/platform/secrets/models.rs`
  - `SecretKind` (6 вариантов), `TryFrom<&str>`, `as_str()`
  - `SecretStoreKind` (6 вариантов), `TryFrom<&str>`, `as_str()`
  - `SecretReference` поля
  - `ResolvedSecret` (конструктор с проверкой, `expose_for_runtime`, Debug с redacted)
  - `NewSecretReference` (конструктор, `metadata`)
- `backend/src/platform/secrets/paths.rs`
  - `default_vault_path(home_dir)` – путь `~/.config/makosh/secrets.vault.json`
- `backend/src/platform/secrets/resolver.rs`
  - Типаж `SecretResolver`, `SecretResolutionFuture`
  - `InMemorySecretResolver` (`HashMap`, `insert`, `resolve_reference` с проверкой `TestDouble`)
- `backend/src/platform/secrets/store.rs`
  - `SecretReferenceStore` (пул)
  - Методы `upsert_secret_reference`, `secret_reference`, `delete_secret_reference`
  - Внутренняя функция `row_to_secret_reference`
- `backend/src/platform/mod.rs`
  - Объявления публичных модулей платформы (16 модулей, 8 из которых не описаны в чанке)

## Исходные файлы

- [`backend/src/platform/events/trace_context.rs`](../../../../backend/src/platform/events/trace_context.rs)
- [`backend/src/platform/events/validation.rs`](../../../../backend/src/platform/events/validation.rs)
- [`backend/src/platform/formatting.rs`](../../../../backend/src/platform/formatting.rs)
- [`backend/src/platform/graph.rs`](../../../../backend/src/platform/graph.rs)
- [`backend/src/platform/mod.rs`](../../../../backend/src/platform/mod.rs)
- [`backend/src/platform/observations/errors.rs`](../../../../backend/src/platform/observations/errors.rs)
- [`backend/src/platform/observations/mod.rs`](../../../../backend/src/platform/observations/mod.rs)
- [`backend/src/platform/observations/models.rs`](../../../../backend/src/platform/observations/models.rs)
- [`backend/src/platform/observations/review_links.rs`](../../../../backend/src/platform/observations/review_links.rs)
- [`backend/src/platform/observations/store.rs`](../../../../backend/src/platform/observations/store.rs)
- [`backend/src/platform/projections.rs`](../../../../backend/src/platform/projections.rs)
- [`backend/src/platform/realtime_conversation/bundle.rs`](../../../../backend/src/platform/realtime_conversation/bundle.rs)
- [`backend/src/platform/realtime_conversation/events.rs`](../../../../backend/src/platform/realtime_conversation/events.rs)
- [`backend/src/platform/realtime_conversation/mod.rs`](../../../../backend/src/platform/realtime_conversation/mod.rs)
- [`backend/src/platform/realtime_conversation/models.rs`](../../../../backend/src/platform/realtime_conversation/models.rs)
- [`backend/src/platform/realtime_conversation/provider.rs`](../../../../backend/src/platform/realtime_conversation/provider.rs)
- [`backend/src/platform/secrets.rs`](../../../../backend/src/platform/secrets.rs)
- [`backend/src/platform/secrets/crypto.rs`](../../../../backend/src/platform/secrets/crypto.rs)
- [`backend/src/platform/secrets/database_vault.rs`](../../../../backend/src/platform/secrets/database_vault.rs)
- [`backend/src/platform/secrets/errors.rs`](../../../../backend/src/platform/secrets/errors.rs)
- [`backend/src/platform/secrets/file_vault.rs`](../../../../backend/src/platform/secrets/file_vault.rs)
- [`backend/src/platform/secrets/models.rs`](../../../../backend/src/platform/secrets/models.rs)
- [`backend/src/platform/secrets/paths.rs`](../../../../backend/src/platform/secrets/paths.rs)
- [`backend/src/platform/secrets/resolver.rs`](../../../../backend/src/platform/secrets/resolver.rs)
- [`backend/src/platform/secrets/store.rs`](../../../../backend/src/platform/secrets/store.rs)

## Кандидаты на drift

Из предоставленного контекста расхождения между кодом, документацией или ADR не видны – в чанке отсутствуют существующие wiki-страницы или ADR для сравнения.
