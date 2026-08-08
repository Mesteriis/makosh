### Summary / Резюме

Обновить страницу `operations/backend-tests.md` русской Obsidian‑wiki, добавив описание структуры, контрактов и покрытия интеграционных тестов Telegram‑интеграции Макошь. Основанием служат только встроенные исходные файлы тестов. Страница должна объяснить операторам и разработчикам, что тестируется, какие таблицы проверяются, как организовано тестовое окружение, и какие архитектурные ограничения существуют.

---

### Proposed pages / Предлагаемые страницы

**`operations/backend-tests.md`**

```markdown
# Бэкенд-тесты Макошь (Telegram-интеграция)

## Обзор

Бэкенд-тесты покрывают интеграцию с Telegram на уровне HTTP API
и внутренних сервисов. Тесты используют модуль `testkit` для управления
временной базой данных и роутер приложения (`build_router_with_database`).
Все HTTP-запросы выполняются с секретом `LOCAL_API_TOKEN`.

Общий вспомогательный модуль: `backend/tests/telegram_support.rs`.

## Структура тестов

Каждый файл содержит один или несколько асинхронных тестов, сгруппированных
по функциональным областям.

### Файлы тестов

| Файл | Назначение |
|------|-----------|
| `telegram_account_setup_capabilities.rs` | Создание учётных записей (боты и пользователи), сохранение токенов, определение runtime |
| `telegram_architecture.rs` | Архитектурный контроль: файлы реализации не должны превышать 700 строк |
| `telegram_commands_query_filters.rs` | Фильтрация команд по чату и типу команды |
| `telegram_core.rs` | Входящие сообщения: генерация кандидатов решений (decisions) и обязательств (obligations); проверка возможностей API |
| `telegram_dialog_actions.rs` | Действия с диалогами: восстановление видимости сообщений, добавление/удаление реакций |
| `telegram_dialog_capability_gates.rs` | Блокировка действий над диалогами в фикстурном рантайме |
| `telegram_dialog_read_reconciliation.rs` | Сверка состояния диалогов (mark_read, pin, archive, mute) с состоянием провайдера |
| `telegram_folder_actions.rs` | Действия с папками: добавление, удаление, переназначение |
| `telegram_manual_send_capability_gates.rs` | Блокировка отправки сообщений после удаления учётной записи |
| `telegram_media_projection.rs` | Обработка медиасообщений без текста; отказ в скачивании медиа без live-рантайма |
| `telegram_media_upload.rs` | Загрузка медиафайлов: импорт вложения, команда отправки, защита от вредоносных файлов |
| `telegram_members_admin_roster.rs` | Состав участников чата с администраторами (порядок выдачи) |
| `telegram_members_inactive_filter.rs` | Фильтрация неактивных участников (left, banned, absent_exhaustive) |
| `telegram_members_sync_exhaustive_absence.rs` | Скрытие участников, помеченных как отсутствующие после полной сверки состава |

## Подготовка тестового окружения

Каждый тест:
- Создаёт `TestContext` (временная БД).
- Подключает `Database` через `Database::connect`.
- Строит роутер приложения (`build_router_with_database`) с dev-режимом (`with_test_dev_mode()`) или с кастомными переменными окружения (для хранилища секретов).
- Генерирует уникальный суффикс (`unique_suffix()`) для идентификаторов.

Для фикстурных аккаунтов используется эндпоинт:
`POST /api/v1/integrations/telegram/fixtures/accounts`
и фикстурные сообщения:
`POST /api/v1/integrations/telegram/fixtures/messages`.

## Ключевые проверяемые таблицы

После HTTP-запросов тесты обращаются напрямую к БД через `sqlx` для проверки:

- `communication_provider_accounts` – учётные записи провайдеров и конфигурация.
- `telegram_provider_write_commands` – команды записи к провайдеру.
- `telegram_message_reactions` – реакции на сообщения.
- `telegram_chat_participants` – участники чатов.
- `event_log` – события предметной области.
- `api_audit_log` – аудит операций API.
- `observations`, `observation_links` – записи наблюдений.
- `decisions`, `decision_evidence` – решения и их доказательства.
- `task_candidates`, `obligations` – кандидаты задач и обязательства.
- `encrypted_secret_vault_entries` – записи шифрованного хранилища секретов.

## Покрытие функциональности

### Создание учётных записей

- **Бот-токен**: при создании аккаунта `telegram_bot` через `POST /api/v1/integrations/telegram/accounts` бот-токен сохраняется в `host_vault`, а не в БД; runtime возвращается как `live_blocked`; secret_kind = `api_token`, store_kind = `host_vault`.
- **QR-авторизация**: для `telegram_user` с `qr_authorized = true` credential_bindings пусты, runtime = `tdlib_qr_authorized`; бот-токен и api_hash не попадают в config.
- **Инференс QR-авторизации**: если `qr_authorized` не передан явно, runtime определяется как `tdlib_qr_authorized`.

### Обработка входящих сообщений

- При поступлении фикстурного сообщения через `POST /api/v1/integrations/telegram/fixtures/messages`:
  - Генерируются события `signal.raw.telegram.message.observed` и `signal.accepted.telegram.message` (ровно по одному).
  - Отсутствуют legacy-события `integration.telegram.*`.
  - Создаётся цепочка событий (observation capture → raw → accepted → recorded) с корректным correlation_id.
  - Создаётся кандидат решения (`decisions`) со статусом `suggested`, связанный с сообщением через `decision_evidence`.
  - Создаётся кандидат задачи (`task_candidates`) типа `obligation_task` со статусом `suggested`, но фактическая задача (`tasks`) и обязательство (`obligations`) не создаются.

### Проверка возможностей (capabilities)

- Эндпоинт `GET /api/v1/integrations/telegram/capabilities` возвращает:
  - `runtime_mode`: `"fixture"` (в dev-режиме).
  - `telegram_app_credentials_configured`: `false`.
  - `qr_login_ready`: `false`.
  - Список возможностей с состояниями: `telegram_fixture_runtime` = `available`, `tdlib_live_runtime` = `blocked`, и т.д.
  - Списки `unsupported_features` (например, `hidden_recording`) и `planned_features` (например, `bot_runtime`, `ai_review_flows`).

### Действия с диалогами

- **Восстановление видимости**: `POST .../messages/{message_id}/restore-visibility` создаёт команду `restore_visibility`.
- **Реакции**: `POST .../messages/{message_id}/reactions` создаёт реакцию и команду `react`; `DELETE` по тому же ресурсу создаёт команду `unreact` и меняет флаг `is_active` на `false`.
- **Фикстурная блокировка**: для фикстурного аккаунта все действия (pin, unpin, archive, unarchive, mute, unmute, read, unread, folder add/remove/reassign) возвращают `400 BAD_REQUEST` и не создают команд, аудит-записей или событий; состояние чата не меняется.
- **Реконсилиация чтения**: вызов `reconcile_mark_read_commands_from_provider_state` завершает команду `mark_read` со статусом `completed` и reconciliation_status `observed`, если `last_read_inbox_provider_message_id` провайдера >= запрошенного.
- **Реконсилиация пина**: если команда `unpin` была отправлена, а провайдер сообщает `is_pinned = true`, команда помечается как `failed` с причиной mismatch.
- **Реконсилиация архива**: аналогично для `unarchive` при `is_archived = true`.
- **Реконсилиация mute**: команда `unmute` помечается как `failed` с mismatch, если провайдер сообщает отличное от ожидаемого состояние заглушения; детали запроса `unmute` обрезаны, но проверка mismatch подтверждена.

### Папки чатов

- **Добавление в папку**: `POST .../conversations/{telegram_chat_id}/folders/{folder_id}` создаёт команду `folder_add` со статусом `queued`.
- **Удаление из папки**: `POST .../folders/{folder_id}/remove` создаёт команду `folder_remove`.
- **Переназначение папок**: вызов `folders/reassign` должен создавать команды добавления и удаления на основе текущего членства в папках (детали обрезаны).

### Отправка сообщений

- После удаления аккаунта (`DELETE /api/v1/integrations/telegram/accounts/{id}`) попытка отправки сообщения через `POST .../provider-commands/messages/send` возвращает `400 BAD_REQUEST`.
- Количество сообщений в чате не изменяется.
- Аудит-запись `telegram.message.send` и событие `telegram.command.status_changed` не создаются.

### Медиа

- **Приём медиа без текста**: tdlib-проекция корректно обрабатывает `messagePhoto` с пустым текстом; сообщение сохраняется с пустым `body_text`.
- **Скачивание без live-рантайма**: `POST .../provider-media/download` с фикстурным аккаунтом завершается ошибкой `400 BAD_REQUEST` с сообщением о требовании активного TDLib-актора; записываются события `telegram.media.download.started` и `telegram.media.download.failed`.
- **Загрузка медиа**:
  - Импорт вложения через `POST /api/v1/communications/attachments/import`.
  - Запрос на отправку через `POST .../provider-media/upload` создаёт команду `send_media` со статусом `queued` и `reconciliation_status = not_observed`.
  - Повторный запрос с тем же attachment_id возвращает тот же `command_id` (идемпотентность).
  - Аудит-запись не содержит `caption`.
  - Если вложение помечено как `malicious` (после сканирования), команда не создаётся, возвращается `400 BAD_REQUEST`.

### Участники чатов

- **Выдача участников**: `GET /api/v1/communications/conversations/{id}/members` возвращает список.
  - Приоритет отдаётся администраторам (админ идёт первым).
  - Участники со статусами `left`, `banned`, `absent_exhaustive` исключаются.
  - После вызова `mark_absent_members_from_exhaustive_roster` участники, отсутствующие в переданном списке, помечаются как `absent_exhaustive` и перестают возвращаться.

## Архитектурный лимит строк

Тест `telegram_architecture.rs` проверяет, что ни один файл в следующих директориях
не превышает 700 строк (нарушения печатаются в ошибке):

- `backend/src/app/api_support`
- `backend/src/integrations/telegram`
- `backend/tests/*` (только файлы с префиксом `telegram`)
- `frontend/src/integrations/telegram` (TypeScript/Vue)
```

---

### Source coverage / Покрытие источников

| Файл | Покрытые факты |
|------|----------------|
| `backend/tests/telegram_account_setup_capabilities.rs` | API создания аккаунтов; сохранение бот-токена в `host_vault` с `SecretKind::ApiToken` и `SecretStoreKind::HostVault`; QR-авторизация без секрета; инференс runtime как `tdlib_qr_authorized`; таблица `communication_provider_accounts` и её config. |
| `backend/tests/telegram_architecture.rs` | Ограничение 700 строк для файлов реализации Telegram в директориях `src/app/api_support`, `src/integrations/telegram`, `tests` (с префиксом `telegram`), `frontend/src/integrations/telegram`. |
| `backend/tests/telegram_commands_query_filters.rs` | Эндпоинт `GET /api/v1/integrations/telegram/commands` с параметрами `account_id`, `provider_chat_id`, `provider_message_id`, `command_kinds`, `limit`; фильтрация команд по чату и типу; функция `insert_command` из `lifecycle`. |
| `backend/tests/telegram_core.rs` (truncated) | Входящее сообщение генерирует события `signal.raw.telegram.message.observed` и `signal.accepted.telegram.message`; отсутствие `integration.telegram.*`; цепочка событий с correlation_id; создание кандидатов `decisions` и `task_candidates`; проверка capabilities через `GET /api/v1/integrations/telegram/capabilities`. |
| `backend/tests/telegram_dialog_actions.rs` | Команды `restore_visibility`, `react`, `unreact` создаются через API; таблица `telegram_provider_write_commands`; реакции в `telegram_message_reactions`; observation links с `TELEGRAM_MESSAGE_REACTION` и relationship `local_add`/`local_remove`. |
| `backend/tests/telegram_dialog_capability_gates.rs` | Фикстурный аккаунт блокирует pin/unpin/archive/unarchive/mute/unmute/read/unread/folder-добавление/удаление/переназначение (400 BAD_REQUEST); отсутствие команд, аудита, событий; неизменность метаданных чата. |
| `backend/tests/telegram_dialog_read_reconciliation.rs` (truncated) | Реконсилиация mark_read: команда завершается как `completed` с `reconciliation_status = observed`; pin: команда `unpin` → `failed` с mismatch и `is_pinned = true`; archive: `unarchive` → `failed` с mismatch; mute: `unmute` → `failed` с mismatch (завершающая часть обрезана). |
| `backend/tests/telegram_folder_actions.rs` (truncated) | Команды `folder_add` и `folder_remove` создаются со статусом `queued`; таблица `telegram_provider_write_commands` с полями `command_kind`, `payload.provider_folder_id`, `action_class = provider_write`. Reassign обрезан. |
| `backend/tests/telegram_manual_send_capability_gates.rs` | Удалённый аккаунт блокирует отправку сообщения (400 BAD_REQUEST); количество сообщений не увеличивается; аудит `telegram.message.send` и событие `telegram.command.status_changed` не создаются. |
| `backend/tests/telegram_media_projection.rs` | Пустое медиасообщение (messagePhoto) корректно проецируется; `body_text` пуст; скачивание без live‑рантайма возвращает 400 с сообщением; события `telegram.media.download.started` и `failed`. |
| `backend/tests/telegram_media_upload.rs` | Импорт вложения; команда `send_media` со статусом `queued`; идемпотентность по attachment_id; аудит без `caption`; вредоносное вложение (`malicious`) блокирует загрузку (400, нет команды). |
| `backend/tests/telegram_members_admin_roster.rs` | Эндпоинт `GET .../members` возвращает участников; администраторы выводятся первыми; тест возвращает всех вставленных участников (2). |
| `backend/tests/telegram_members_inactive_filter.rs` | Участники со статусами `left`, `banned`, `absent_exhaustive` исключаются из выдачи; остаётся только активный (`member`). |
| `backend/tests/telegram_members_sync_exhaustive_absence.rs` | После вызова `mark_absent_members_from_exhaustive_roster` участники, не переданные в списке, становятся `absent_exhaustive` и перестают возвращаться в `/members`. |

---

### Drift candidates / Кандидаты на drift

- **`telegram_members_admin_roster.rs`**: название теста утверждает `admin_only provider roster rows` (только администраторы), однако тест вставляет двух участников – администратора и обычного пользователя – и проверяет, что в ответе присутствуют оба (`items.len(), 2`). Поведение не совпадает с названием теста. Возможен drift в именовании: либо название должно быть скорректировано, либо логика теста должна проверять только администраторов.

- **`telegram_account_setup_capabilities.rs` (truncated)**: тест `telegram_live_account_setup_api_require` обрезан – невозможно подтвердить, какие именно требования API он проверяет. Покрытие этого теста неполное, и страница не может отразить его утверждения.

- **`telegram_folder_actions.rs` (truncated)**: тест `telegram_folder_reassign_action_queues_add_and_remove_commands_from_current_membership` обрезан – детали логики переназначения папок не подтверждены.

- **`telegram_dialog_read_reconciliation.rs` (truncated)**: тест `dialog_mute_reconciliation_marks_mismatched_unmute_command_failed` обрезан, последние утверждения не видны; описание реконсилиации mute может быть неполным.

- **`telegram_core.rs` (truncated)**: тест `telegram_api_exercises_policy_and_call_foundation` обрезан, видна только начальная часть (capabilities и создание аккаунта). Полные контракты политики и вызовов не подтверждены.
