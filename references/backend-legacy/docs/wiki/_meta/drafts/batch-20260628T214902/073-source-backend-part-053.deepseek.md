### Summary / Резюме

Обновить страницу `components/backend.md` русской wiki в Obsidian, добавив или актуализировав документацию по ключевым компонентам бэкенда на основе актуальных исходных файлов. Включить описания модуля хранилища (Storage), локального шифрованного хранилища секретов (Host Vault) и рабочих процессов (Consistency Review, Email Fixture Pipeline, Email Intelligence). Изменения отражают текущие реализации, модели ошибок, жизненный цикл шифрования, схему базы данных и интеграции.

### Proposed pages / Предлагаемые страницы

#### `components/backend.md`

```markdown
# Компоненты бэкенда

## Хранилище (Storage)

Модуль `backend/src/platform/storage` предоставляет абстракцию над PostgreSQL.

### Модели готовности

`ReadinessStatus` – перечисление статусов готовности:

- `Ok` – компонент доступен.
- `NotConfigured` – не настроен (отсутствует `DATABASE_URL`).
- `Unavailable` – недоступен (с произвольным сообщением).

`DatabaseReadiness` и `MigrationReadiness` – сериализуемые структуры, содержащие статус `ReadinessStatus` и статическое сообщение.

Конструкторы:

- `DatabaseReadiness::ok()` – "database is reachable".
- `DatabaseReadiness::not_configured()` – "DATABASE_URL is not configured".
- `DatabaseReadiness::unavailable(message)`.
- `MigrationReadiness::ok()` – "required database migrations are applied".
- `MigrationReadiness::not_configured()` – "DATABASE_URL is not configured".
- `MigrationReadiness::unavailable(message)`.

### Ошибки хранилища

`StorageError` – перечисление ошибок:

- `Connect(#[from] sqlx::Error)` – ошибка подключения.
- `Io(#[from] std::io::Error)`.
- `EventStore(#[from] EventStoreError)` – прозрачная ошибка хранилища событий.
- `Settings(#[from] SettingsError)` – прозрачная ошибка настроек.
- `Invalid(String)` – некорректное значение.

Публичный реэкспорт: `Database` (структура), `StorageError`, модели готовности, а также компоненты подмодуля `communication_media` (импорт вложений, сканирование безопасности и т.д.).

## Хранилище секретов (Host Vault)

Модуль `backend/src/vault` – локальное шифрованное хранилище конфиденциальных данных с накоплением энтропии. Ключевая структура – `HostVault` (реализует `SecretResolver`, `Clone`).

### Конфигурация

`HostVaultConfig`:

- `home: PathBuf` – корневая директория хранилища (по умолчанию `$HOME/.makosh/vault`).
- `dev_mode: bool` – флаг режима разработки (запрещён в release-сборках).
- `dev_key_path: PathBuf` – путь к dev-ключу (по умолчанию `$HOME/.makosh/dev/master.key`).

### Состояния

`VaultMode` (сериализуется как `snake_case`):

- `Uninitialized` – хранилище не создано.
- `Locked` – хранилище создано, но мастер-ключ не загружен.
- `Unlocked` – мастер-ключ в памяти, операции разрешены.

`VaultStatus` (возвращается методами жизненного цикла и `status()`):

- `state: VaultMode`.
- `needs_entropy: bool` – требуется сбор энтропии (если не инициализировано и событий < `MIN_ENTROPY_EVENTS`).
- `needs_biometric: bool` – требуется биометрическая разблокировка (инициализировано, не dev_mode).
- `needs_recovery: bool` – не инициализировано.
- `version: u16` – версия хранилища (константа `VAULT_VERSION = 1`).
- `recoverable: bool` – существует ли файл восстановления `makosh-recovery.key`.
- `entropy_progress: u8` – процент накопленной энтропии (0–100).

### Энтропия

`EntropyEvent` – событие энтропии от датчиков:

```json
{
  "x": f64, "y": f64, "dx": f64, "dy": f64,
  "timestamp_ms": f64, "velocity": f64, "acceleration": f64, "interval_ms": f64
}
```

Константы:

- `MIN_ENTROPY_EVENTS = 2000` – минимальное число событий для создания хранилища.
- `MASTER_KEY_LEN = 32` – длина мастер-ключа в байтах.

Методы:

- `collect_entropy(events: Vec<EntropyEvent>)` – добавляет события в буфер; возвращает актуальный `VaultStatus`. Пустой батч вызывает `EmptyEntropyBatch`.
- `entropy_progress(events: usize)` – вычисляет процент как `min(events, MIN_ENTROPY_EVENTS) * 100 / MIN_ENTROPY_EVENTS`.

### Жизненный цикл

- `HostVault::new(config)` – создаёт экземпляр, инициализирует SQLite-базу (`vault.db`), проверяет dev-режим и права доступа.
- `status()` – возвращает `VaultStatus`.
- `create()` – генерирует мастер-ключ из OS‑random и накопленной энтропии (если событий ≥ 2000), сохраняет ключ, разблокирует хранилище, записывает контрольную запись целостности. Повторный вызов после инициализации вызывает `AlreadyInitialized`.
- `unlock()` – загружает мастер-ключ из хранилища ключей, разблокирует, проверяет vault check.
- `unlock_existing()` – если ключ сохранён, вызывает `unlock()`, иначе возвращает текущий статус.
- `lock()` – переводит состояние в `Locked`, удаляя ключ из памяти.

### Хранение мастер-ключа

- `has_stored_master_key()`: в dev‑режиме проверяет наличие файла `dev_key_path`, иначе на macOS использует `keyring` (service = `"makosh"`, user = `"host-vault-master-key"`). На других платформах release-сборка возвращает `UnsupportedPlatform`.
- `store_master_key()` / `load_master_key()` – кодирование base64, запись/чтение через Keychain или безопасный файл.

Безопасное файловое хранение:

- `ensure_secure_dir` – создаёт директорию с правами `0o700` (unix).
- `ensure_secure_file` – выставляет `0o600`.
- `write_secure_file` – атомарная запись через временный файл с синхронизацией.
- `guard_release_dev_mode` – dev‑режим запрещён вне debug-сборок.

### Криптография

Функции модуля `crypto`:

- `derive_master_key(os_random, entropy)`:
  1. `SHA512(os_random || entropy_json || timestamp_nanos)`.
  2. `HKDF(SHA256, ikm = digest, info = "makosh-host-vault:master:v1")` → 32 байта.
- `derive_domain_key(master_key, label)`:
  1. `HKDF(SHA256, ikm = master_key, info = "makosh-host-vault:v1:" || label)` → 32 байта.
- `entry_aad(secret_ref, context)` – строка ассоциированных данных для AEAD:
  `"v={VAULT_VERSION};ref={secret_ref};kind={entry_kind};account_id={account_id};purpose={purpose};secret_kind={secret_kind}"`.
- `recovery_phrase(master_key)` – 16 групп шестнадцатеричных байт через пробел.
- `master_key_from_recovery_phrase(phrase)` – обратное преобразование; ожидает `MASTER_KEY_LEN * 2` символов без пробелов.
- `decode_master_key(encoded)` – base64-декодирование.
- `validate_non_empty(field, value)` – проверка непустого значения.

### База данных SQLite

Инициализация (`initialize_database`): PRAGMA `journal_mode = WAL`, `foreign_keys = ON`. Таблицы:

- `vault_entries` (secret_ref TEXT PK, entry_kind, account_id, purpose, version, nonce, ciphertext, aad, created_at, updated_at) – зашифрованные значения секретов.
- `account_secret_manifest` (secret_ref TEXT PK, entry_kind, account_id, purpose, secret_kind, store_kind, label, metadata, updated_at) – манифест секретов.
- `vault_check` (id INTEGER PK CHECK (id=1), version, nonce, ciphertext, aad, updated_at) – проверка целостности хранилища.

Целостность:

- `write_vault_check` – шифрует строку `"makosh-host-vault"` доменным ключом `"integrity"` (XChaCha20Poly1305) и сохраняет.
- `read_vault_check` – расшифровывает и сверяет; при ошибке версий возвращает `UnsupportedVaultVersion`, при ошибке расшифровки – `Crypto`.

### Операции с секретами

`HostVault` реализует типаж `SecretResolver` через `resolve_host_secret`.

- `store_secret(secret_ref, value, context)`:
  1. Валидация полей (secret_ref, value, entry_kind, account_id, purpose).
  2. Доменный ключ `"encryption"`, шифрование `XChaCha20Poly1305` с AAD `entry_aad`.
  3. UPSERT в `vault_entries`.
  4. Вызов `upsert_manifest_entry`.
- `read_secret(secret_ref)`:
  1. Запрос `vault_entries`.
  2. Проверка версии (`VAULT_VERSION`).
  3. Расшифровка доменным ключом `"encryption"`, AAD из записи.
  4. UTF-8 декодирование.
- `delete_secret(secret_ref)` – удаление из `vault_entries` и манифеста.
- `resolve_host_secret(reference)` – проверяет `store_kind == SecretStoreKind::HostVault`, иначе `UnsupportedStoreKind`.

### Манифест секретов

- `account_secret_manifest()` – список всех записей манифеста (упорядочен аккаунт, цель, ссылка).
- `upsert_account_secret_manifest_entry(secret_ref, context)` – валидация и вызов `upsert_manifest_entry`, который выполняет UPSERT в `account_secret_manifest`, всегда проставляя `store_kind = 'host_vault'`.
- `delete_manifest_entry(secret_ref)` – возвращает `true`, если запись удалена.

### Восстановление

- `export_recovery()`:
  1. Генерирует фразу восстановления из текущего мастер-ключа.
  2. Вычисляет recovery-ключ как `derive_domain_key(key, b"recovery")`.
  3. Шифрует мастер-ключ XChaCha20Poly1305 (nonce, ciphertext) и сохраняет в защищённый файл `makosh-recovery.key` (JSON с версией, nonce, ciphertext).
  4. Возвращает `RecoveryExportResponse { path, recovery_phrase }`.
- `import_recovery(recovery_phrase)` – восстанавливает ключ из фразы, сохраняет, разблокирует, записывает vault check.

### Ошибки (HostVaultError)

Основные варианты:

- `Uninitialized`, `AlreadyInitialized`, `Locked`, `StatePoisoned` – ошибки жизненного цикла.
- `InsufficientEntropy { collected, required }` – недостаточно событий энтропии.
- `EmptyEntropyBatch` – пустой батч в `collect_entropy`.
- `Crypto`, `Random`, `InvalidEncoding`, `InvalidRecoveryPhrase` – криптографические ошибки.
- `UnsupportedVaultVersion(u16)` – несовместимая версия хранилища.
- `MissingSecret { secret_ref }` – секрет не найден.
- `DevModeForbiddenInRelease`, `UnsupportedPlatform` – ограничения платформы.
- `EmptyField(&'static str)` – пустое обязательное поле.
- `Io`, `Sqlite`, `Json`, `Keyring` (macOS) – обёрнутые ошибки.

Публичное сообщение формируется через `public_message()`; функция `host_secret_store_failure(error)` преобразует в `SecretResolutionError::StoreFailure`.

## Рабочие процессы (Workflows)

### Consistency Review

`ContradictionReviewService` управляет ручным ревью противоречий, обнаруженных движком Consistency.

- `review_manual(observation_id, review_state, resolution)`:
  1. Захватывает наблюдение `REVIEW_TRANSITION`.
  2. Обновляет `review_state` противоречия через `ContradictionObservationPort`.
  3. Вызывает `sync_contradiction_review_item` для синхронизации с Review Inbox.

`sync_contradiction_review_item` (и внутренние транзакционные варианты):

- `ensure_contradiction_review_item_in_transaction` – создаёт `NewReviewItem` со статусом `New` и привязывает evidence-observation с деталями противоречия.
- `sync_contradiction_review_state_in_transaction` – маппинг состояний:

  - `Suggested` → `ReviewItemStatus::New`
  - `UserConfirmed` → `Approved`
  - `UserRejected` → `Dismissed`

Ошибки: `ContradictionReviewWorkflowError` (Sqlx, Consistency, ReviewInbox, Observation), `ContradictionReviewServiceError` (Consistency, Observation, ReviewWorkflow).

### Email Fixture Pipeline

Предоставляет импорт фикстурных email-сообщений для разработки и проекцию в граф знаний.

- `import_fixture_email_messages_for_dev(request)` – upsert аккаунта провайдера (Gmail, iCloud, IMAP, Telegram и т.д.) с заготовленной конфигурацией, затем импорт через `import_fixture_email_messages_with_records`.
- `project_fixture_email_messages(request)`:
  1. Импорт записей.
  2. Для каждой raw-записи: `dispatch_mail_raw_signal` → `project_accepted_signal_if_runtime_allows`.
  3. Сбор участников, upsert персон.
  4. Запуск `GraphProjectionService::project_from_v1`.
  5. Возврат отчёта с количеством записей, сообщений, персон, метриками графа.

Модели: `EmailFixturePipelineRequest`, `EmailFixtureImportPipelineReport`, `EmailFixtureProjectionPipelineReport`.

Ошибки: `EmailFixturePipelineError` – агрегирует ошибки `CommunicationIngestionError`, `FixtureEmailImportError`, `MessageProjectionError`, `SignalHubError`, `CommunicationSignalProjectionError`, `PersonProjectionError`, `GraphProjectionError`, `GraphProjectionPortError`.

### Email Intelligence

Сервис анализа email-сообщений с AI и эвристиками.

#### Категории

`EmailCategory`: `Critical`, `Important`, `Personal`, `Work`, `Finance`, `Legal`, `Notification`, `Newsletter`, `Marketing`, `Spam`, `Scam`, `Phishing`, `Suspicious`. Методы `as_str()` и `parse()`.

#### Эвристики

- `heuristic_score(message)` – оценка важности (0–100) на основе ключевых слов в теме/теле: urgent/add 15, finance/add 20, legal/add 25, вопросы/add 10, вложения/add 10, спам‑слова/sub 20, короткое тело/sub 10.
- `heuristic_category(message)` – определение категории по ключевым словам (finance, legal, marketing, notification, suspicious).
- `structured_summary(message)` – структурированное извлечение key_points, action_items, risks, deadlines, кандидатов событий, персон, организаций, документов, соглашений из текста письма.

#### AI‑анализ

`EmailIntelligenceService` принимает `Option<SharedAiRuntimePort>`.

- `analyze_message(message)` – если runtime доступен, отправляет промпт (`build_email_analysis_prompt`), парсит JSON‑ответ в `EmailAnalysis`.
- `analyze_and_persist(store, message)` – выполняет анализ и сохраняет результат:
  - `set_ai_analysis` (категория, саммари, важность).
  - Обновляет `message_metadata` полем `ai_summary_contract`.
  - Если `is_spam` или `is_phishing`, переводит в `WorkflowState::Spam`.
  - Если `importance_score >= 80`, переводит в `NeedsAction`.

Версия промпта: `v3-email-intelligence-mail-knowledge-candidates-2026-06-15`. Промпт запрашивает категорию, саммари, ключевые пункты, действия, риски, дедлайны, кандидатов, оценку, флаги спама/фишинга, язык.

Модель `EmailAnalysis` содержит все возвращаемые поля плюс `model` и `prompt_version`.

#### Ошибки

`EmailIntelligenceError`: `AiRuntimePortError`, `MessageProjectionError`, `ParseError`.
```

### Source coverage / Покрытие источников

- **`backend/src/platform/storage/errors.rs`**: `StorageError` enum и его варианты (`Connect`, `Io`, `EventStore`, `Settings`, `Invalid`); прозрачная связь с `EventStoreError` и `SettingsError`.
- **`backend/src/platform/storage/mod.rs`**: публичный реэкспорт `Database`, `StorageError`, типов подмодуля `communication_media`, моделей `DatabaseReadiness`, `MigrationReadiness`, `ReadinessStatus`.
- **`backend/src/platform/storage/models.rs`**: структуры `DatabaseReadiness`, `MigrationReadiness` (конструкторы `ok`, `not_configured`, `unavailable`, геттеры `status`, `message`); перечисление `ReadinessStatus` (`Ok`, `NotConfigured`, `Unavailable`) c `serde(rename_all = "snake_case")`.
- **`backend/src/vault/constants.rs`**: константы `VAULT_VERSION = 1`, `MIN_ENTROPY_EVENTS = 2000`, `MASTER_KEY_LEN = 32`.
- **`backend/src/vault/crypto.rs`**: функции `derive_master_key`, `derive_domain_key`, `entry_aad`, `recovery_phrase`, `master_key_from_recovery_phrase`, `decode_master_key`, `entropy_progress`, `validate_non_empty`; используемые алгоритмы (SHA512, HKDF-SHA256, base64).
- **`backend/src/vault/errors.rs`**: полный `HostVaultError` enum, метод `public_message()`, функция `host_secret_store_failure`.
- **`backend/src/vault/files.rs`**: функции безопасной работы с файлами (`write_secure_file`, `ensure_secure_dir`, `ensure_secure_file`, `guard_release_dev_mode`), права доступа `0o700`/`0o600`.
- **`backend/src/vault/key_store.rs`**: методы `has_stored_master_key`, `store_master_key`, `load_master_key`, использование Keychain (`"makosh"`/`"host-vault-master-key"`) или dev-файла; base64-кодирование.
- **`backend/src/vault/lifecycle.rs`**: методы жизненного цикла (`new`, `status`, `collect_entropy`, `create`, `unlock`, `unlock_existing`, `lock`), внутренние `domain_key`, `current_master_key`, `set_unlocked`, `is_unlocked`; зависимость от энтропии, генерация OS‑random.
- **`backend/src/vault/manifest.rs`**: методы `account_secret_manifest`, `upsert_account_secret_manifest_entry`, `upsert_manifest_entry`, `delete_manifest_entry`; структура SQL-запросов, таблица `account_secret_manifest`, store_kind `'host_vault'`.
- **`backend/src/vault/mod.rs`**: структура `HostVault` с полями `home`, `dev_mode`, `dev_key_path`, `state`, `entropy`; публичный реэкспорт.
- **`backend/src/vault/models.rs`**: все модели (`HostVaultConfig`, `VaultMode`, `VaultStatus`, `EntropyEvent`, `SecretEntryContext`, `RecoveryExportResponse`, `HostVaultManifestEntry`, `RecoveryFile`, `StoredVaultEntry`, `SessionKey`, `HostVaultState`; `Zeroize` на `SessionKey`).
- **`backend/src/vault/paths.rs`**: функции `default_vault_home`, `default_dev_key_path` с путями `$HOME/.makosh/vault` и `$HOME/.makosh/dev/master.key`.
- **`backend/src/vault/recovery.rs`**: методы `export_recovery` (создание recovery-файла и фразы) и `import_recovery` (восстановление по фразе); использование XChaCha20Poly1305, доменного ключа `b"recovery"`.
- **`backend/src/vault/secrets.rs`**: реализация `SecretResolver`; методы `store_secret`, `resolve_host_secret`, `read_secret`, `delete_secret`; шифрование XChaCha20Poly1305 с ключом `b"encryption"` и AAD; таблица `vault_entries`.
- **`backend/src/vault/storage.rs`**: инициализация SQLite (`initialize_database`) с таблицами `vault_entries`, `account_secret_manifest`, `vault_check`; проверка целостности (`write_vault_check`, `read_vault_check`) с доменным ключом `b"integrity"`; пути `vault.db`, `makosh-recovery.key`.
- **`backend/src/workflows/consistency_review.rs`**: `ContradictionReviewService` и `review_manual`; функции синхронизации `sync_contradiction_review_item`, `ensure_contradiction_review_item_in_transaction`, `sync_contradiction_review_state_in_transaction`; маппинг статусов `Suggested`→`New`, `UserConfirmed`→`Approved`, `UserRejected`→`Dismissed`; ошибки `ContradictionReviewWorkflowError`, `ContradictionReviewServiceError`.
- **`backend/src/workflows/email_fixture_pipeline.rs`**: функции `import_fixture_email_messages_for_dev`, `project_fixture_email_messages`; модели `EmailFixturePipelineRequest`, `EmailFixtureImportPipelineReport`, `EmailFixtureProjectionPipelineReport`; конфигурации провайдеров (Gmail, iCloud, IMAP); ошибка `EmailFixturePipelineError`.
- **`backend/src/workflows/email_intelligence/mod.rs`**: публичный реэкспорт `EmailCategory`, `EmailIntelligenceError`, `EmailAnalysis`, `EmailKnowledgeCandidate`, `EmailSummaryContract`, `EmailIntelligenceService`.
- **`backend/src/workflows/email_intelligence/categories.rs`**: enum `EmailCategory` (13 вариантов), методы `as_str()`, `parse()`.
- **`backend/src/workflows/email_intelligence/errors.rs`**: `EmailIntelligenceError` (Runtime, MessageProjection, ParseError).
- **`backend/src/workflows/email_intelligence/heuristics.rs`**: функции `heuristic_score` (система баллов), `heuristic_category` (правила), `structured_summary` (извлечение key_points, action_items, рисков, дедлайнов, кандидатов); списки ключевых слов.
- **`backend/src/workflows/email_intelligence/models.rs`**: структуры `EmailAnalysis`, `EmailKnowledgeCandidate`, `EmailSummaryContract`.
- **`backend/src/workflows/email_intelligence/prompt.rs`**: константа версии промпта, функция `build_email_analysis_prompt`.
- **`backend/src/workflows/email_intelligence/service.rs`**: `EmailIntelligenceService` с `analyze_message`, `analyze_and_persist`, `heuristic_score`, `heuristic_category`, `heuristic_structured_summary`; логика сохранения AI‑анализа, обновление метаданных, переходы `WorkflowState`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (только исходные файлы, без текущего содержимого wiki-страниц и ADR) расхождения кода, документации и ADR не выявлены. Для обнаружения потенциального drift-а необходимо сравнение с актуальным текстом wiki, который не был включён в этот context pack.
