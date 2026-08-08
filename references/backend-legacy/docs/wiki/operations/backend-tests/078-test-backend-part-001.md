---
chunk_id: 078-test-backend-part-001
batch_id: batch-20260628T214902
group: backend
role: test
source_status: pending
source_count: 17
generated_by: code-wiki-ru
---

# 078-test-backend-part-001 — backend/test

- Target index: [[operations/backend-tests]]
- Batch: `batch-20260628T214902`
- Source files: `17`

## Резюме

Чанк `078-test-backend-part-001` (роль `test`) содержит исходные файлы тестов бэкенда Макошь: E2E API тесты на Python, интеграционные Rust‑тесты Telegram (реконсилиация, парсинг снапшотов, QR‑логин, запросы), AI‑тесты, архитектурный тест и вспомогательный код. На основе этих файлов предлагается создать/обновить wiki‑страницу `operations/backend-tests.md`, которая документирует организацию, покрытие и ключевые проверки бэкенд‑тестов.

## Предложенные страницы

#### `operations/backend-tests.md`

```markdown
# Тесты бэкенда

Тесты бэкенда Макошь покрывают API‑, интеграционные и архитектурные сценарии. Они разделены на несколько уровней:

- **E2E‑тесты** — сквозные тесты HTTP API на Python (pytest).
- **Интеграционные тесты на Rust** — тесты логики интеграции с Telegram (реконсилиация команд, парсинг снапшотов, QR‑логин, построение запросов).
- **Тесты AI** — тесты API‑эндпоинтов AI, семантического хранилища и пайплайнов.
- **Вспомогательный код** — builders для `AppConfig` и фабрики хранилищ.

## E2E‑тесты (Python)

Файл: `backend/e2e/test_api.py`

Запуск (из исходного кода):

```bash
MAKOSH_API_SECRET=... pytest backend/e2e/test_api.py -v
```

Требуется запущенный бэкенд; URL задаётся через `MAKOSH_BACKEND_URL` (по умолчанию `http://127.0.0.1:18082`).

### Проверки

- **Health**: `GET /healthz`, `GET /readyz` возвращают `{"status": "ok"}`.
- **Аутентификация**: запросы без заголовка `x-makosh-secret` или с неверным значением возвращают `403 FORBIDDEN` с ошибкой `"invalid_api_secret"`.
- **Статус API**: `GET /api/v1/status` с корректным секретом возвращает `version: "1.0"` и флаги активных «поверхностей» (`messages: true`, `persons: true`).
- **Организации**: полный CRUD (создание, получение, обновление, архивация, поиск, под‑ресурсы, toggle watchlist). Идентификаторы организации начинаются с префикса `org:`.
- **Календарь**: CRUD аккаунтов и событий календаря, reschedule, cancel, participants, а также read‑only эндпоинты (`deadlines`, `focus-blocks`, `watchtower`, `health`, `weekly-brief`, `search`, `rules`, `analytics/*`).
- **Задачи**: CRUD задач (детали обрезаны в предоставленном фрагменте, но файл содержит дополнительную логику).

## Интеграционные тесты Telegram (Rust)

Расположение: `backend/src/integrations/telegram/`

### Реконсилиация команд

Файлы:
- `runtime/manager/chat_events/tests/archive_reconciliation.rs`
- `runtime/manager/chat_events/tests/mark_unread_reconciliation.rs`
- `runtime/manager/chat_events/tests/pin_mute_reconciliation.rs`

Тесты проверяют, что функция `publish_chat_position_event` корректно сверяет состояние чата с провайдером и обновляет запись в таблице `telegram_provider_write_commands`:

- При совпадении ожидаемого и наблюдаемого состояния (архив/неархив, пометка непрочитанным, пин, mute) команда переходит в статус `completed` и `reconciliation_status = "observed"`.
- При расхождении — статус `failed`, `reconciliation_status = "mismatch"`, поле `last_error` содержит сообщение о несовпадении, а `provider_state` фиксирует ожидаемые и наблюдаемые значения (например, `expected_is_archived` vs `observed_is_archived`).
- Дополнительно проверяется, что в `event_log` генерируются события `telegram.command.status_changed` и `telegram.command.reconciled`.

### Парсинг снапшотов TDLib

Файл: `tdjson/tests/parsing_snapshots.rs`

Покрывает функции парсинга JSON‑ответов TDLib в типизированные структуры:

- `parse_tdlib_file_snapshot` — информация о файле.
- `parse_tdlib_chat_snapshot` — данные чата.
- `parse_tdlib_chat_member_list` — участники с ролями и правами.
- `parse_tdlib_basic_group_member_list` — участники basic‑группы.
- `parse_tdlib_typing_snapshot` — события набора текста.
- `parse_tdlib_topic_update_snapshot` — обновление форум‑топика.
- `parse_tdlib_chat_unread_snapshot` — непрочитанные сообщения и упоминания.
- `parse_tdlib_chat_marked_as_unread_snapshot` — пометка «непрочитано».
- `parse_tdlib_chat_notification_settings_snapshot` — настройки уведомлений.
- `parse_tdlib_chat_position_snapshot` — позиция чата (архив/основной/папка, закрепление).
- `parse_tdlib_chat_removed_from_list_snapshot` — удаление из списка.
- `parse_tdlib_chat_folder_snapshot` — свойства папки.

Все функции возвращают `Option` или `Result`, тесты валидируют корректное извлечение полей (например, `provider_chat_id`, `role`, `is_pinned`).

### QR‑логин

Файл: `tdjson/tests/qr_login_flows.rs`

Покрывает жизненный цикл QR‑логина Telegram:

- Состояние `authorizationStateWaitPassword` не допускает QR‑запрос.
- Ответ `password_waiting_response` не содержит утекшего QR‑токена.
- `ready_response` для уже авторизованной сессии возвращает статус `Ready` и не раскрывает QR‑токен.
- `qr_preparing_response` при подготовке возвращает статус `WaitingQrScan`.
- Отправка пароля (`submit_qr_login_password`) пересылает команду `CheckPassword` воркеру.
- Отправка пароля, когда статус не `WaitingPassword`, возвращает ошибку `InvalidRequest`.
- Отмена QR‑логина (`cancel_qr_login`) удаляет сессию и отправляет команду `Cancel`.
- Запуск нового QR‑логина (`cancel_existing_qr_logins_for_account`) отменяет существующие сессии той же учётной записи, но не чужие.

### Построение запросов к TDLib

Файл: `tdjson/tests/request_builders.rs`

Проверяет JSON‑структуры, отправляемые в TDLib:

- `set_tdlib_parameters_request` — параметры (`api_id`, `api_hash`, `encryption_key` в base64, путь к данным).
- `check_database_encryption_key_request` — ключ шифрования в base64 без plaintext‑секрета.
- `tdlib_send_text_message_request` — отправка текстового сообщения с форматированием.
- `tdlib_send_media_message_request` — отправка медиа с локального пути (с reject пустого пути).
- `tdlib_get_chat_history_request` — запрос истории с ограничением лимита 100.
- `tdlib_download_file_request` — синхронная загрузка файла.
- `tdlib_create_forum_topic_request` — создание форум‑топика (с reject пустого названия).
- `tdlib_edit_chat_folder_remove_chat_request` — редактирование папки (удаление чата из списков, перемещение в excluded).
- `tdlib_toggle_forum_topic_is_closed_request` — закрытие/открытие топика.
- `tdlib_edit_message_text_request` — редактирование сообщения (с reject пустого текста).
- `tdlib_delete_messages_request` — удаление сообщений по ID.
- `tdlib_add_message_reaction_request` / `tdlib_remove_message_reaction_request` — реакции.
- `tdlib_pin_chat_message_request` / `tdlib_unpin_chat_message_request` — закрепление сообщений.
- (Фрагмент обрезан, файл содержит дополнительные тесты, в том числе `tdlib_toggle_chat_marked_as_unread_re`.)

### Окружение TDLib

Файл: `tdjson/tests/environment.rs`

- `macos_tdjson_candidates_prefer_bundled_tauri_resources` — проверяет, что при запуске из `/Applications/Макошь.app` bundled‑ресурс (`Contents/Resources/tdlib/...`) приоритетнее `homebrew`.
- `renders_tdlib_qr_link_as_svg` — проверяет, что `render_qr_svg` генерирует SVG длиной > 100 символов.

## Тесты AI

Расположение: `backend/tests/ai/` + модуль `backend/tests/ai.rs`.

### Answers API

Файл: `ai/answers.rs`

- `POST /api/v1/ai/answers` с агентом `MNEMOSYNE` возвращает `202 Accepted` и `run_id`; итоговый ответ читается из `ai_agent_runs` / `GET /api/v1/ai/runs/{run_id}`. Там сохраняются `agent_id`, `agent_persona_id`, `owner_persona_id`, модель, `embedding_model`, `duration_ms` и список цитирований (`citations`) с `source_kind` и `source_id`.
- В `event_log` появляются канонические `ai.run.requested` / `ai.run.completed` и `ai.hub.requested` / `ai.hub.completed`.
- Если Signal Hub содержит политику `Muted` для источника `ai`, запрос блокируется с `412 PRECONDITION_FAILED` и сообщением «AI runtime is disabled by Signal Hub policy».
- При отсутствии route публичный запрос всё равно принимается, но завершается failed run с причиной `route_not_configured`.
- `POST /api/v1/ai/task-candidates/refresh` тоже возвращает `202 Accepted`, а затем создаёт кандидатов задач (`task_candidates`) со статусом `suggested`, не создавая активных задач. Также создаются `review_items` с kind `potential_task`.
- `POST /api/v1/ai/model-downloads` для встроенной Ollama-модели помечает модель доступной без автосоздания route и пишет `ai.hub.model_download.requested` / `ai.hub.model_download.completed`; при ошибке pull пишет `ai.hub.model_download.failed`.

### Agents API

Файл: `ai/agents.rs`

- `GET /api/v1/ai/agents` защищён секретом (без токена — `403`).
- Список агентов содержит 5 записей, включая `HESTIA` и `HEPHAESTUS`.
- Каждый агент имеет `persona_id`, `persona_type="ai_agent"`, `persona_email`.
- В БД проверяется материализация персон: записи в `persons`, `person_identities`, `graph_nodes`.

### Semantic Store

Файл: `ai/semantic_store.rs`

- Проверяет, что расширение `vector` (pgvector) установлено.
- `SemanticEmbeddingStore::upsert_embedding` сохраняет эмбеддинги (2560 измерений) для сообщений и документов. Повторный upsert идемпотентен.
- Создаются observation‑записи типа `AI_SEMANTIC_EMBEDDING` с relationship `upsert`.
- Поиск (`search`) возвращает результаты с наибольшим `score`; первый результат — наиболее релевантный.

### Поддержка тестов AI

Файл: `ai/support.rs`

- Предоставляет fake‑сервер Ollama (эмулирует `/api/version`, `/api/tags`, `/api/embed`, `/api/chat`).
- Хелперы: `spawn_fake_ollama`, `configure_fake_ollama_setting`, `unit_embedding`, `seed_message`, `seed_document`, `json_post_request_with_actor`.
- Константа `AI_RUNTIME_TEST_LOCK` (мьютекс) для сериализации AI тестов.

### Архитектурный тест

Файл: `ai_architecture.rs`

- `ai_tests_stay_below_architecture_line_limit` — рекурсивно обходит директорию `tests/` и проверяет, что файлы в подкаталоге `ai` (и сам `ai.rs`) не превышают 700 строк.
- При нарушении тест падает с перечислением файлов и количеством строк.

## Тестовая поддержка

### AppConfig

Файл: `src/platform/config/app_config/test_support.rs`

Предоставляет методы `impl AppConfig` для создания тестовых конфигураций:

- `test_with_api_secret` / `test_with_api_secret_and_database_url` — конфигурация с заданным секретом и выключенными планировщиками Zoom.
- `with_test_database_url` / `with_test_api_secret` — fluent‑установка URL БД и секрета.
- `with_test_pairs` — применение пар ключ‑значение (env‑переменных) через `apply_config_pair`.
- `with_test_dev_mode` — включает dev‑режим.
- `with_test_dev_vault_paths` — устанавливает пути хранилища и ключа.
- `with_test_tdjson_path` — задаёт путь к tdjson.
- `with_test_telegram_app_credentials` — устанавливает `api_id` и `api_hash`.

### Общие хелперы

Файл: `src/test_support.rs`

Фабрики хранилищ, используемые в интеграционных тестах:

- `communication_provider_account_store`, `communication_provider_secret_binding_store`
- `telegram_store`, `whatsapp_web_store`
- `upsert_telegram_runtime_account`
- `restore_signal_hub_system_sources`
- `set_signal_runtime_state`
- `load_communication_raw_record`

Эти функции принимают `PgPool` и возвращают экземпляры хранилищ с внедрёнными зависимостями.

## Фикстуры Signal Hub

Файл: `fixtures/signal_hub/test_signals.toml`

Определяет тестовый сигнал:

- `fixture_id = "fixture_basic_message"`
- `event_type = "signal.raw.fixture.message.observed"`
- `subject_kind = "signal"`
- `payload` содержит `message_key`, `summary`, `text`
- `provenance: catalog = "signal_hub"`, `kind = "test_fixture"`
```

## Покрытие источников

| Исходный файл | Факты, покрытые страницей |
|---|---|
| `backend/e2e/test_api.py` | Запуск через `pytest`, env‑переменные `MAKOSH_BACKEND_URL`/`MAKOSH_API_SECRET`; тесты health, auth (403 при отсутствии/неверном секрете), статус API (версия, поверхности); тесты организаций (CRUD, префикс `org:`, под‑ресурсы, watchlist); тесты календаря (CRUD аккаунтов/событий, reschedule, cancel, participants, read‑only эндпоинты); тесты задач (CRUD, начало теста видно). |
| `backend/fixtures/signal_hub/test_signals.toml` | Тестовый сигнал с `fixture_id`, `event_type`, `payload` и `provenance`. |
| `archive_reconciliation.rs` | Примирение archive/unarchive команд через `publish_chat_position_event`; совпадение → `completed`/`observed`, расхождение → `failed`/`mismatch` с `last_error` и `provider_state`; проверка событий `telegram.command.status_changed` и `telegram.command.reconciled` в `event_log`. |
| `mark_unread_reconciliation.rs` | Примирение mark_unread команд; совпадение → `completed`/`observed`, расхождение → `failed`/`mismatch` с сообщением «Provider observed a different unread state»; проверка событий. |
| `pin_mute_reconciliation.rs` | Примирение pin/unmute команд (через `publish_chat_position_event` и `publish_chat_notification_settings_event`); расхождение в pin → `failed`/`mismatch` с `"Provider observed a different dialog pin state"`; расхождение в unmute → `failed`/`mismatch` с `"Provider observed a different mute state"`. |
| `environment.rs` | Приоритет bundled Tauri‑ресурса tdjson над homebrew на macOS; `render_qr_svg` генерирует SVG > 100 символов. |
| `parsing_snapshots.rs` | Функции парсинга (`parse_tdlib_file_snapshot`, `parse_tdlib_chat_snapshot`, `parse_tdlib_chat_member_list`, `parse_tdlib_basic_group_member_list`, `parse_tdlib_typing_snapshot`, `parse_tdlib_topic_update_snapshot`, `parse_tdlib_chat_unread_snapshot`, `parse_tdlib_chat_marked_as_unread_snapshot`, `parse_tdlib_chat_notification_settings_snapshot`, `parse_tdlib_chat_position_snapshot`, `parse_tdlib_chat_removed_from_list_snapshot`, `parse_tdlib_chat_folder_snapshot`); извлечение полей (`provider_chat_id`, `role`, `is_pinned` и др.). |
| `qr_login_flows.rs` | Состояния QR‑логина: `authorizationStateWaitPassword` не допускает QR‑запрос; `password_waiting_response` не раскрывает QR‑токен; `ready_response` для активной сессии возвращает `Ready`; `qr_preparing_response` → `WaitingQrScan`; отправка пароля (`submit_qr_login_password`) и валидация статуса; отмена сессии с отправкой `Cancel`; отмена существующих сессий той же учётной записи при старте новой. |
| `request_builders.rs` | Построение TDLib‑запросов: `set_tdlib_parameters_request`, `check_database_encryption_key_request`, `tdlib_send_text_message_request`, `tdlib_send_media_message_request` (reject пустого пути), `tdlib_get_chat_history_request` (лимит 100), `tdlib_download_file_request` (синхронный), `tdlib_create_forum_topic_request` (reject пустого названия), `tdlib_edit_chat_folder_remove_chat_request`, `tdlib_toggle_forum_topic_is_closed_request`, `tdlib_edit_message_text_request` (reject пустого текста), `tdlib_delete_messages_request`, `tdlib_add_message_reaction_request`, `tdlib_remove_message_reaction_request`, `tdlib_pin_chat_message_request`, `tdlib_unpin_chat_message_request`. |
| `app_config/test_support.rs` | Методы `test_with_api_secret`, `test_with_api_secret_and_database_url`, `with_test_*`; управление параметрами конфигурации для тестов. |
| `test_support.rs` | Фабрики `communication_provider_account_store`, `telegram_store`, `whatsapp_web_store`; `upsert_telegram_runtime_account`, `restore_signal_hub_system_sources`, `set_signal_runtime_state`, `load_communication_raw_record`. |
| `ai/answers.rs` | `/api/v1/ai/answers` и `/api/v1/ai/task-candidates/refresh` работают как accepted/run API: принимают запрос, возвращают `run_id`, затем материализуют completion или failure в `ai_agent_runs` и `event_log`; блокировка при Signal Hub политике `Muted` для источника `ai` (412); `task-candidates/refresh` создаёт кандидатов задач и `review_items` с kind `potential_task`. |
| `ai/agents.rs` | `GET /api/v1/ai/agents` требует секрет; 5 агентов, включая `HESTIA` и `HEPHAESTUS`; материализация персон в `persons`, `person_identities`, `graph_nodes`. |
| `ai/semantic_store.rs` | Установка расширения `vector`; upsert эмбеддингов (2560 dim), идемпотентность; создание observation‑записей `AI_SEMANTIC_EMBEDDING`; поиск с разницей в score. |
| `ai/support.rs` | Fake Ollama (4 маршрута), хелперы `spawn_fake_ollama`, `unit_embedding`, `seed_message`, `seed_document`; `AI_RUNTIME_TEST_LOCK`. |
| `ai_architecture.rs` | Проверка, что файлы в `tests/ai/` не превышают 700 строк; рекурсивный обход. |

*Примечание:* файлы `test_api.py`, `archive_reconciliation.rs`, `parsing_snapshots.rs`, `request_builders.rs`, `answers.rs` предоставлены не полностью (обрезаны). Документированы только факты из включённых символов.

## Исходные файлы

- [`backend/e2e/test_api.py`](../../../../backend/e2e/test_api.py)
- [`backend/fixtures/signal_hub/test_signals.toml`](../../../../backend/fixtures/signal_hub/test_signals.toml)
- [`backend/src/integrations/telegram/runtime/manager/chat_events/tests/archive_reconciliation.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_events/tests/archive_reconciliation.rs)
- [`backend/src/integrations/telegram/runtime/manager/chat_events/tests/mark_unread_reconciliation.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_events/tests/mark_unread_reconciliation.rs)
- [`backend/src/integrations/telegram/runtime/manager/chat_events/tests/pin_mute_reconciliation.rs`](../../../../backend/src/integrations/telegram/runtime/manager/chat_events/tests/pin_mute_reconciliation.rs)
- [`backend/src/integrations/telegram/tdjson/tests/environment.rs`](../../../../backend/src/integrations/telegram/tdjson/tests/environment.rs)
- [`backend/src/integrations/telegram/tdjson/tests/parsing_snapshots.rs`](../../../../backend/src/integrations/telegram/tdjson/tests/parsing_snapshots.rs)
- [`backend/src/integrations/telegram/tdjson/tests/qr_login_flows.rs`](../../../../backend/src/integrations/telegram/tdjson/tests/qr_login_flows.rs)
- [`backend/src/integrations/telegram/tdjson/tests/request_builders.rs`](../../../../backend/src/integrations/telegram/tdjson/tests/request_builders.rs)
- [`backend/src/platform/config/app_config/test_support.rs`](../../../../backend/src/platform/config/app_config/test_support.rs)
- [`backend/src/test_support.rs`](../../../../backend/src/test_support.rs)
- [`backend/tests/ai.rs`](../../../../backend/tests/ai.rs)
- [`backend/tests/ai/agents.rs`](../../../../backend/tests/ai/agents.rs)
- [`backend/tests/ai/answers.rs`](../../../../backend/tests/ai/answers.rs)
- [`backend/tests/ai/semantic_store.rs`](../../../../backend/tests/ai/semantic_store.rs)
- [`backend/tests/ai/support.rs`](../../../../backend/tests/ai/support.rs)
- [`backend/tests/ai_architecture.rs`](../../../../backend/tests/ai_architecture.rs)

## Кандидаты на drift

Контекст не содержит существующей wiki‑страницы `operations/backend-tests.md` или связанных ADR. Расхождения кода и документации на основе предоставленных файлов не видны.
