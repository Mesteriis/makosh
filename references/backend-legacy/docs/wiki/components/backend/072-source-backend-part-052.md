---
chunk_id: 072-source-backend-part-052
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 072-source-backend-part-052 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк `072-source-backend-part-052` содержит новую порцию исходного кода backend (модули `platform/settings`, `platform/storage/communication_media`, `platform/storage/database`, `platform/secrets/validation`). Предыдущее содержимое страницы `components/backend.md` не предоставлено в контексте, поэтому предлагается её первичное создание с описанием этих трёх крупных подсистем, строго на основе встроенного кода.

## Предложенные страницы

### `components/backend.md`

```markdown
---
tags:
  - backend
  - settings
  - storage
  - database
---

# Backend

## Подсистема настроек (`backend/src/platform/settings`)

Управление настройками приложения реализовано через декларативный реестр и персистентное хранение в PostgreSQL.

### Декларации настроек

Модуль `definitions` агрегирует все известные приложению настройки из подкатегорий:
- `server` — сетевые и служебные параметры бэкенда;
- `frontend` — настройки десктопного shell (API URL, тема, layout, сайдбар, UI-состояние, локаль);
- `privacy` — политики конфиденциальности для интеграций (Zoom, Yandex Telemost);
- `ai` — провайдер AI, модели, таймауты;
- `ui` — тема и плотность интерфейса.

Каждая настройка описывается структурой `DeclaredApplicationSetting` (ключ, категория, тип значения, значение по умолчанию, метаданные, флаг редактируемости). Метаданные определяют UI-контрол (`text`, `select`, `number`, `checkbox`, `hidden`, `appearance`, `json`, `language`), допустимые значения, ограничения `min`/`max`, размер и запрещённые ключи для JSON-настроек, флаг `bootstrap`, необходимость рестарта и др.

Примеры деклараций:
- `server.http_addr` (строка, по умолчанию `"127.0.0.1:8080"`; имеет `bootstrap: true`, `restart_required: true`, `env_var: "MAKOSH_HTTP_ADDR"`);
- `frontend.api_base_url` (строка, по умолчанию `"http://127.0.0.1:8080"`; `bootstrap: true`, `env_var: "VITE_MAKOSH_API_BASE_URL"`);
- `ai.provider` (select, значения `"ollama"` или `"omniroute"`, по умолчанию `"ollama"`; `stores_secret: false`);
- `ai.omniroute_base_url` (строка, по умолчанию `"https://ai.sh-inc.ru/v1"`; API-ключ из переменной окружения `MAKOSH_OMNIROUTE_API_KEY`, никогда не хранится в настройках);
- `frontend.theme` (JSON, версия схемы 1, описывает фон, яркость, акцентный цвет, прозрачность панели, blur);
- `frontend.layout` (JSON, версия схемы 2, хранит размещение виджетов);
- `frontend.sidebar` (JSON, версия схемы 3, задаёт группировку и порядок пунктов сайдбара);
- `frontend.ui_state` (JSON, скрытый, версия схемы 1, временное UI-состояние, макс. размер 65536 байт, запрещённые ключи: `["body", "html", "raw", "text", "password", "token", "secret"]`);
- `privacy.zoom_remote_transcript_download_enabled` (boolean, по умолчанию `false`, opt-in);
- `privacy.zoom_recording_import_retention_days` (integer, 0..3650, retention-политика);
- `ui.theme` (select: `system`, `dark`, `light`);
- `ui.density` (select: `comfortable`, `compact`).

### Валидация

Модуль `validation` обеспечивает:
- формат ключа: только `[a-z0-9_-.]`, начинается и заканчивается буквой или цифрой;
- запрет ключей, содержащих маркеры секретов (`secret`, `password`, `token`, `credential`, `private_key`) — `SECRET_LIKE_MARKERS`;
- проверка непустоты значений полей (`Validate_non_empty`);
- проверка ограничений JSON-метаданных (`max_bytes`, `forbidden_keys`);
- рекурсивный поиск запрещённых ключей в JSON-значениях (учёт `body`, `html`, `raw`, `text`, `password`, `token`, `secret` и их производных, включая вложенные объекты);
- валидацию самого `DeclaredApplicationSetting` (ключ, непустая категория и метка, объектность metadata, соответствие значения по умолчанию типу).

### Хранение и восстановление (Store)

`ApplicationSettingsStore` оперирует пулом соединений PostgreSQL. Основные операции:
- `list_settings` — возвращает все настройки, сортированные по категории и ключу, используя `declared_setting_keys()` как фильтр;
- `setting` — загружает конкретную настройку, предварительно валидируя ключ и проверяя наличие в реестре деклараций;
- `update_setting_value` — обновляет значение: проверки (ключ, актор, существование, редактируемость, тип и метаданные), при отсутствии записи — запускает `repair_declared_settings` и повторяет попытку;
- `ai_runtime_settings` — собирает `AiRuntimeSettings` из сохранённых настроек с фоллбэком на `AppConfig`;
- `repair_declared_settings` — создаёт таблицу (если её нет), для каждой декларации: вставляет отсутствующую, сбрасывает значение до значения по умолчанию при несовместимости (value_kind/валидация), обновляет метаданные/категорию/описание при расхождении. Возвращает `ApplicationSettingsRepairSummary` с количеством вставленных, отремонтированных и сброшенных значений.

### AiRuntimeSettings

Структура `AiRuntimeSettings` содержит поля: `provider`, `base_url`, `chat_model`, `embedding_model`, `timeout_seconds`. Сборка происходит либо из конфигурационного файла (`from_config`), либо из массива сохранённых `ApplicationSetting` с приоритетом выбора модели/URL в зависимости от `ai.provider`. Таймаут проверяется на >0, иначе используется значение из конфига, соответствующее текущему провайдеру.

Доступные ключи для AI-настроек:
- `ai.provider`
- `ai.ollama_base_url`
- `ai.omniroute_base_url`
- `ai.chat_model`
- `ai.omniroute_chat_model`
- `ai.embedding_model`
- `ai.omniroute_embedding_model`
- `ai.timeout_seconds`

AI-модели по умолчанию: чат `qwen3:4b` (ollama), `codex/gpt-5.5` (omniroute); эмбеддинги `qwen3-embedding:4b` (ollama), `openai-compatible-chat-ollama-pve/qwen3-embedding:4b` (omniroute, требование размерности 2560).

### Ошибки

`SettingsError` — перечисление:
- `Sqlx` (прозрачная обёртка `sqlx::Error`);
- `UnsupportedValueKind`, `EmptyField`, `InvalidSettingKey`, `SecretLikeSettingKey`, `InvalidValue`, `SettingNotFound`, `ReadOnlySetting`.

Метод `is_invalid_request()` возвращает `true` для всех ошибок, кроме `Sqlx` и `SettingNotFound`.

### Схема БД

Таблица `application_settings` создаётся через `CREATE TABLE IF NOT EXISTS` и содержит столбцы: `setting_key` (TEXT PK), `category`, `value_kind`, `value` (JSONB), `label`, `description`, `metadata` (JSONB), `is_editable`, `updated_by_actor_id`, временные метки. Ограничения:
- `setting_key` не пуст, формат `^[a-z0-9][a-z0-9_.-]*[a-z0-9]$`, не содержит секретоподобных маркеров;
- `category` и `label` не пусты;
- `value_kind` ∈ `{boolean, integer, string, json}`;
- `metadata` — объект.
Индекс: `(category, setting_key)`.

## Подсистема хранения (`backend/src/platform/storage`)

### Локальное файловое хранилище блобов

Функция `put_local_blob` сохраняет массив байт по пути `sha256/{первые_2_символа}/{hex}.blob` внутри корневой директории `root`. Вычисляется SHA256, путь хранения детерминирован. При сохранении создаются промежуточные директории, используется временный файл с атомарным переименованием, проверяется совпадение размера после записи. Возвращается `LocalBlobRecord` (storage_kind: `"local_fs"`, storage_path, sha256 с префиксом `sha256:`, size_bytes).

Функция `delete_local_blob` удаляет файл по валидированному `storage_path` (валидация запрещает абсолютные пути, обратные слэши, специальные компоненты вроде `..`) и затем рекурсивно подчищает пустые родительские директории, не выходя за `root`.

### Сканирование безопасности вложений

`scan_attachment` выполняет эвристическую проверку вложения (`SafetyScanRequest`):
- Детектирует исполнимые magic-байты (MZ и ELF) — статус `Malicious`;
- Проверяет расширение на принадлежность к списку активного контента (`.exe`, `.dll`, `.jar`, `.ps1`, `.scr`, `.vbs` и др.) — `Malicious`;
- Проверяет расширение на принадлежность к макросодержащим документам (`.docm`, `.dotm`, `.xlsm`, `.xltm`, `.pptm`, `.potm`) — `Suspicious`;
- Сравнивает Content-Type и расширение, при несовпадении — `Suspicious`;
- Статусы агрегируются с повышением серьёзности; при отсутствии причин возвращается `NotScanned`.

Результат упаковывается в `SafetyScanReport` с указанием движка `"makosh_heuristic_v1"`, временной метки, списка причин и полной метаинформации в `metadata`.

### Импорт вложений

Трейт `ImportedAttachmentStoragePort` определяет интерфейс для учёта импортированных вложений:
- `upsert_blob_record` — сохраняет метаданные блоба и возвращает `StoredBlobRecord`;
- `upsert_imported_attachment_record` — сохраняет запись об импорте (`ImportedAttachmentUpsert`: attachment_id, account_id, channel_kind, blob_id, filename, content_type, size_bytes, sha256, source_kind, imported_by, scan_report, metadata) и возвращает `ImportedAttachmentRecord`;
- `list_imported_attachment_records` — постраничная выборка по аккаунту и источнику;
- `list_expired_imported_attachment_records` — выборка записей, подлежащих удалению по retention-политике;
- `remove_imported_attachment_record` — удаление отдельной записи и связанных метаданных блоба, возвращает `ImportedAttachmentRemovalResult`.

## Подсистема базы данных (`backend/src/platform/storage/database`)

Структура `Database` инкапсулирует подключение к PostgreSQL:
- `connect` — создаёт пул (макс. 5 соединений), запускает миграции через `run_migrations`, проводит ремонт настроек (`repair_declared_settings`). При отсутствии URL БД возвращает экземпляр в отключённом режиме.
- `readiness` — проверяет `SELECT 1`; при ошибке возвращает `DatabaseReadiness::unavailable` с текстом ошибки.
- `migration_readiness` — запрашивает из `_sqlx_migrations` количество применённых, последнюю версию и число проваленных миграций; сравнивает с ожидаемым `MigrationSummary` (из модуля `events`). При расхождении или наличии проваленных миграций — `unavailable`.

## Валидация секретов (`backend/src/platform/secrets/validation`)

Набор `pub(super)` функций для проверки непустых строк и объектов в контексте работы с секретами:
- `validate_non_empty` → `SecretReferenceError::EmptyField`
- `validate_object` → `SecretReferenceError::NonObjectJson`
- `validate_secret_resolution_ref` → `SecretResolutionError::EmptySecretRef`
- `validate_vault_field` → `EncryptedVaultError::EmptyField`
- `validate_database_non_empty` → `DatabaseEncryptedVaultError::EmptyField`

Все проверки выполняют `.trim().is_empty()`, функциональность используется вышележащими слоями работы с хранилищами секретов.
```

## Покрытие источников

| Source file | Covered facts |
|---|---|
| `backend/src/platform/secrets/validation.rs` | Функции `validate_non_empty`, `validate_object`, `validate_secret_resolution_ref`, `validate_vault_field`, `validate_database_non_empty` и их типы ошибок |
| `backend/src/platform/settings.rs` | Публичные реэкспорты модуля settings |
| `backend/src/platform/settings/ai_runtime.rs` | Структура `AiRuntimeSettings`, метод `from_config`, функция `runtime_settings_from_values`, вспомогательные функции разрешения отдельных полей и ключи настроек AI |
| `backend/src/platform/settings/constants.rs` | Константы `SECRET_LIKE_MARKERS`, `UI_STATE_FORBIDDEN_KEYS`, `UI_STATE_MAX_BYTES` |
| `backend/src/platform/settings/definitions.rs` | Агрегация declared-настроек и публичные функции `declared_setting_keys`, `declared_setting`, `declared_application_settings` |
| `backend/src/platform/settings/definitions/ai.rs` | Агрегация настроек провайдера, моделей и runtime |
| `backend/src/platform/settings/definitions/ai/models.rs` | Декларации моделей чата и эмбеддингов для Ollama и OmniRoute, значения по умолчанию, метаданные |
| `backend/src/platform/settings/definitions/ai/provider.rs` | Декларации `ai.provider`, URL Ollama и OmniRoute, политика API-ключа через `MAKOSH_OMNIROUTE_API_KEY` |
| `backend/src/platform/settings/definitions/ai/runtime.rs` | Декларация `ai.timeout_seconds` с ограничениями 1..600 |
| `backend/src/platform/settings/definitions/frontend.rs` | Агрегация frontend-настроек |
| `backend/src/platform/settings/definitions/frontend/appearance.rs` | Декларация `frontend.theme`, версия схемы, допустимые значения фона, яркости, цветов, прозрачности, blur |
| `backend/src/platform/settings/definitions/frontend/bootstrap.rs` | Декларация `frontend.api_base_url`, флаг `bootstrap`, связь с `VITE_MAKOSH_API_BASE_URL` |
| `backend/src/platform/settings/definitions/frontend/layout.rs` | Декларации `frontend.layout` и `frontend.sidebar`, версии схем, структура сайдбара (rootItemIds, groups) |
| `backend/src/platform/settings/definitions/frontend/state.rs` | Декларации `frontend.locale` (en, ru) и `frontend.ui_state` (скрытое, max_bytes, forbidden_keys) |
| `backend/src/platform/settings/definitions/privacy.rs` | Декларации политик для Zoom и Yandex Telemost (разрешения на скачивание, retention дни), метаданные scope, policy_kind, stores_private_content |
| `backend/src/platform/settings/definitions/server.rs` | Декларации `server.http_addr` (bootstrap, restart required, env) и `signal_hub.active_profile` (hidden) |
| `backend/src/platform/settings/definitions/tests.rs` | Тесты валидации `frontend.locale`, `frontend.ui_state` (запрет приватных ключей, ограничение размера), а также тесты privacy-настроек Zoom/Telemost |
| `backend/src/platform/settings/definitions/ui.rs` | Декларации `ui.theme` (system/dark/light) и `ui.density` (comfortable/compact) |
| `backend/src/platform/settings/errors.rs` | Перечисление `SettingsError`, метод `is_invalid_request` |
| `backend/src/platform/settings/models.rs` | `ApplicationSettingsRepairSummary`, `DeclaredApplicationSetting`, `ApplicationSetting`, `SettingValueKind` (с db_value, validate_value, TryFrom) |
| `backend/src/platform/settings/persistence.rs` | SQL-схема таблицы `application_settings` (столбцы, ограничения, индекс), функции `ensure_application_settings_table`, `fetch_existing_setting_row`, `insert_declared_setting`, `update_declared_setting`, `row_to_setting` |
| `backend/src/platform/settings/store.rs` | `ApplicationSettingsStore` и методы `list_settings`, `setting`, `update_setting_value`, `ai_runtime_settings`, `repair_declared_settings`; логика ремонта и повторной загрузки при отсутствии настройки |
| `backend/src/platform/settings/validation.rs` | Функции `validate_declared_setting`, `validate_json_metadata_constraints`, `validate_setting_key`, `validate_non_empty`; логика запрета секретоподобных ключей и рекурсивного поиска запрещённых ключей в JSON |
| `backend/src/platform/storage/communication_media.rs` | Типы `LocalBlobRecord`, `StoredBlobRecord`, `SafetyScanStatus`, `SafetyScanReport`, `SafetyScanRequest`, `ImportedAttachmentRecord`, `ImportedAttachmentUpsert`, `ImportedAttachmentRemovalResult`; функции `put_local_blob` (SHA256, временный файл, проверка размера), `delete_local_blob` (валидация пути, очистка пустых родительских директорий), `scan_attachment` (эвристики: magic байты, расширения активного контента/макросов, mismatch контент-типа); трейт `ImportedAttachmentStoragePort` с методами upsert и удаления |
| `backend/src/platform/storage/database.rs` | `Database` (connect, disabled, pool, database_url, readiness, migration_readiness); `AppliedMigrationSummary`; логика создания пула, запуска миграций, ремонта настроек при старте, проверок готовности |

## Исходные файлы

- [`backend/src/platform/secrets/validation.rs`](../../../../backend/src/platform/secrets/validation.rs)
- [`backend/src/platform/settings.rs`](../../../../backend/src/platform/settings.rs)
- [`backend/src/platform/settings/ai_runtime.rs`](../../../../backend/src/platform/settings/ai_runtime.rs)
- [`backend/src/platform/settings/constants.rs`](../../../../backend/src/platform/settings/constants.rs)
- [`backend/src/platform/settings/definitions.rs`](../../../../backend/src/platform/settings/definitions.rs)
- [`backend/src/platform/settings/definitions/ai.rs`](../../../../backend/src/platform/settings/definitions/ai.rs)
- [`backend/src/platform/settings/definitions/ai/models.rs`](../../../../backend/src/platform/settings/definitions/ai/models.rs)
- [`backend/src/platform/settings/definitions/ai/provider.rs`](../../../../backend/src/platform/settings/definitions/ai/provider.rs)
- [`backend/src/platform/settings/definitions/ai/runtime.rs`](../../../../backend/src/platform/settings/definitions/ai/runtime.rs)
- [`backend/src/platform/settings/definitions/frontend.rs`](../../../../backend/src/platform/settings/definitions/frontend.rs)
- [`backend/src/platform/settings/definitions/frontend/appearance.rs`](../../../../backend/src/platform/settings/definitions/frontend/appearance.rs)
- [`backend/src/platform/settings/definitions/frontend/bootstrap.rs`](../../../../backend/src/platform/settings/definitions/frontend/bootstrap.rs)
- [`backend/src/platform/settings/definitions/frontend/layout.rs`](../../../../backend/src/platform/settings/definitions/frontend/layout.rs)
- [`backend/src/platform/settings/definitions/frontend/state.rs`](../../../../backend/src/platform/settings/definitions/frontend/state.rs)
- [`backend/src/platform/settings/definitions/privacy.rs`](../../../../backend/src/platform/settings/definitions/privacy.rs)
- [`backend/src/platform/settings/definitions/server.rs`](../../../../backend/src/platform/settings/definitions/server.rs)
- [`backend/src/platform/settings/definitions/tests.rs`](../../../../backend/src/platform/settings/definitions/tests.rs)
- [`backend/src/platform/settings/definitions/ui.rs`](../../../../backend/src/platform/settings/definitions/ui.rs)
- [`backend/src/platform/settings/errors.rs`](../../../../backend/src/platform/settings/errors.rs)
- [`backend/src/platform/settings/models.rs`](../../../../backend/src/platform/settings/models.rs)
- [`backend/src/platform/settings/persistence.rs`](../../../../backend/src/platform/settings/persistence.rs)
- [`backend/src/platform/settings/store.rs`](../../../../backend/src/platform/settings/store.rs)
- [`backend/src/platform/settings/validation.rs`](../../../../backend/src/platform/settings/validation.rs)
- [`backend/src/platform/storage/communication_media.rs`](../../../../backend/src/platform/storage/communication_media.rs)
- [`backend/src/platform/storage/database.rs`](../../../../backend/src/platform/storage/database.rs)

## Кандидаты на drift

1. **Тесты в `definitions/tests.rs` ссылаются на приватные API модуля `definitions`** (например, `declared_setting`), но сам файл тестов является подмодулем `definitions/tests` с `#[cfg(test)]`. Прямого расхождения с документацией нет, так как тесты не документированы на wiki, но любые внешние тесты, полагающиеся на публичное API реестра настроек, должны учитывать, что `declared_setting` доступна только внутри `settings` крейта. В предоставленном контексте такие тесты отсутствуют.

2. **Строка соединения `database_url` передаётся как `Option<&str>`**, а `Database::connect` при отсутствии URL возвращает отключённый экземпляр. В wiki отражено, что БД может быть не сконфигурирована, но не указано, какое поведение системы в этом режиме (вероятно, деградирует функциональность). Дрейфа нет, так как это архитектурное решение, а не расхождение.

3. **Валидация JSON-ключей на запрещённые значения (`is_forbidden_json_key`) содержит логику с производными строками** (`_{marker}`, `{marker}_`, `-{marker}` и т.д.), включая исключение для маркера `text`, когда он находится в конце ключа. Это деталь реализации, не зафиксированная явно в тексте wiki; потенциально может привести к недопониманию, если предполагается, что запрещены только точные совпадения. Дрейф между документацией и кодом не подтверждается, так как предыдущей документации не предоставлено; однако для будущей точности рекомендуется явно отразить механизм проверки.

4. **Метаданные `ai.omniroute_embedding_model` содержат `required_dimension: 2560`** — это ограничение упомянуто в wiki, но не раскрывается, как именно оно проверяется при валидации значения. В представленном коде проверка этой метаданной отсутствует (валидация `validate_json_metadata_constraints` не обрабатывает `required_dimension`). Возможный дрейф между декларацией и фактической валидацией: поле `required_dimension` пока не используется, но объявлено. В wiki указано «требование размерности 2560» без уточнения, что оно активно. Это кандидат на будущее расхождение.

5. **Трейт `ImportedAttachmentStoragePort` определён, но его реализации в контексте нет** — возможен дрейф, если где-либо в документах предполагается наличие конкретных БД-реализаций. В этом чанке не предоставлено ни реализаций, ни дополнительных модулей, поэтому сообщить о несоответствии невозможно.

Остальные расхождения между кодом и какой-либо существующей документацией из данного контекста не видны.
