---
chunk_id: 026-source-backend-part-006
batch_id: batch-20260628T214902
group: backend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 026-source-backend-part-006 — backend/source

- Target index: [[components/backend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Создаётся страница `components/backend.md` русской Obsidian‑wiki. Страница описывает ключевые механизмы бэкенда: централизованную обработку ошибок, защиту API через middleware `require_secret` и структуру обработчиков запросов. Вся информация строится исключительно на встроенных в чанк исходных файлах.

## Предложенные страницы

`components/backend.md`

```markdown
# Backend

Бэкенд‑часть проекта `makosh` написана на Rust с использованием веб‑фреймворка **axum**.
Основной код приложения сосредоточен в `backend/src/app/`.

## Обработка ошибок

### Централизованный тип `ApiError`

Файл `backend/src/app/error/types.rs` содержит перечисление `ApiError`, объединяющее все возможные ошибки приложения:
ошибки базы данных (`DatabaseNotConfigured`), ошибки хранилища событий (`Store`), ошибки доменов (knowledge, persons, review, tasks),
ошибки интеграций (`Telegram`, `WhatsappWeb`, `Zoom`, `YandexTelemost`, `Call`) и многие другие.

Также определён тип `AppError` для инфраструктурных ошибок (ввод‑вывод, хранилище).

### Маппинг ошибок в HTTP‑ответы

Каждый домен или интеграция имеет отдельный модуль маппинга в `backend/src/app/error/response/`,
преобразующий конкретный вариант ошибки в кортеж `ErrorParts`:

```rust
(StatusCode, &'static str, String, bool)
```

Четвёртый элемент во всех видимых мапперах равен `false`.

#### Интеграции

- **Вызовы** (`call.rs`) – `CallError::InvalidRequest` → `400`, `CallError::Sqlx` → `500`.
- **Telegram** (`telegram.rs`) – варианты `TelegramError` маппятся в `400`, `404`, `502`, `503`. Внутренние ошибки (`ProviderAccountStore`, `MediaStorage` и др.) логируются через `tracing::error!` и возвращают `500`.
- **WhatsApp Web** (`whatsapp.rs`) – `WhatsappWebError` → `400` или `500` (с логированием).
- **Yandex Telemost** (`yandex_telemost.rs`) – `YandexTelemostError` → `400` (для `InvalidRequest`, некорректных настроек), остальные → `500`.
- **Zoom** (`zoom.rs`) – `ZoomError` → `400` или `500` (для HTTP‑ошибок провайдера, сериализации, I/O и т.д.).

#### Домены

- **Knowledge** (`knowledge.rs`) – ошибки графа, проектов, link‑review. Варианты `Invalid*` → `400`, `*NotFound` → `404`, остальные → `500`. Последний рукав `match` вызывает `unreachable!` для не‑knowledge вариантов.
- **Persons** (`persons.rs`) – ошибки персон и проекций. Аналогичное разделение: невалидный запрос → `400`, отсутствие сущности → `404`, хранилище → `500`. Ошибки `PersonProjectionError` дополнительно разбираются на валидационные (`EmptyEmailAddress` и др.) и внутренние (`Sqlx`, `Observation`).
- **Platform** (`platform.rs`) – ошибки уровня платформы: `DatabaseNotConfigured` / `SecretVaultNotConfigured` → `503`, конфликт событий (unique violation) → `409`, настройки → `400` или `500`, Signal Hub → `400`/`404`/`412`/`500`.
- **Review** (`review.rs`) – ошибки review‑подсистемы: невалидные запросы → `400`, отсутствие сущности → `404`, ошибки хранилища → `500`. Отдельно обрабатывается `Consistency` → `500`.
- **Tasks** (`tasks.rs`) – пока только `InvalidTaskQuery` → `400`.

Для многократно встречающегося шаблона «внутренняя ошибка» мапперы используют вспомогательную функцию `internal`, которая пишет `tracing::error!` и возвращает `(500, код, сообщение, false)`.

## Защита API (Guard)

Файл `backend/src/app/guard.rs` реализует middleware `require_secret`:

- Проверяет наличие и совпадение секрета в HTTP‑заголовке `x-makosh-secret`.
- Для WebSocket‑эндпоинтов `/api/events/ws` и `/api/events/realtime/ws` дополнительно разрешена передача секрета через query‑параметр `makosh_secret`.
- Если ожидаемый секрет пуст (не задан в конфигурации), любой запрос отклоняется с `403`.
- При несовпадении возвращается JSON‑ответ с ошибкой `"invalid_api_secret"` и HTTP‑статусом `403`.

## Обработчики запросов

Модуль `backend/src/app/handlers.rs` объявляет следующие обработчики (каждый вынесен в отдельный субмодуль):

- `automation`
- `calendar`
- `calls`
- `communications`
- `consistency`
- `decisions`
- `documents`
- `events`
- `graph`
- `obligations`
- `organizations`
- `persons`
- `projects`
- `relationships`
- `review`
- `settings`
- `signal_hub`
- `tasks`
- `telegram`
- `whatsapp`
- `yandex_telemost`
- `zoom`

### Примеры видимых обработчиков

#### Automation

Файл `automation.rs` содержит функции для управления шаблонами и политиками автоматизации:

- `post_policy_template` / `get_policy_templates` – создание и получение шаблонов политик `AutomationTemplate`.
- `post_policy` / `get_policies` – создание и получение `AutomationPolicy`.
- `post_telegram_send_dry_run` – «сухая» отправка через Telegram по заданному шаблону. При отклонении записывается решение (`CapabilityDecision`) в аудит‑лог.

#### Calendar

Обработчики календаря разбиты на несколько файлов:

- `accounts.rs` – управление аккаунтами календаря (`CalendarAccount`) и источниками (`CalendarSource`). Поддерживаются CRUD‑операции с ручным созданием/обновлением.
- `analytics.rs` – аналитика загрузки: распределение времени, фокус‑баланс, back‑to‑back встречи. Период задаётся опциональными query‑параметрами `from`/`to`.
- `brain.rs` – «мозговые» функции: брифинг встречи, генерация повестки, недельный брифинг, аналитика загрузки, ответ на произвольный вопрос.
- `events/` – вложенный модуль для CRUD событий и связанных данных:
  - `crud.rs` – создание, чтение, обновление, удаление `CalendarEvent`. Поддерживается фильтрация по аккаунту, источнику, интервалу дат, статусу, типу.
  - `agenda.rs` – получение и установка повестки (`EventAgenda`).
  - `checklist.rs` – получение и установка чек‑листа (`EventChecklist`).
  - `context_pack.rs` – получение и установка контекстного пакета (`EventContextPack`).
  - `participants.rs` – получение и добавление участников (`EventParticipant`).
  - `relations.rs` – получение и добавление связей события с другими сущностями (`EventRelation`).
  - `status.rs` – изменение статуса: перенос (`reschedule`) и отмена (`cancel`) события.

Все обработчики извлекают пул соединений с БД через `state.database.pool()`, а при его отсутствии возвращают `ApiError::DatabaseNotConfigured` (503).

## Структура проекта (контекст чанка)

- `backend/src/app/error/types.rs` – определение `ApiError` и `AppError`.
- `backend/src/app/error/response/` – маппинг ошибок в HTTP‑ответы (по доменам/интеграциям).
- `backend/src/app/guard.rs` – middleware `require_secret`.
- `backend/src/app/handlers.rs` – модуль со списком обработчиков.
- `backend/src/app/handlers/automation.rs` – обработчики автоматизации.
- `backend/src/app/handlers/calendar/` – обработчики календаря (accounts, analytics, brain, events).
```

## Покрытие источников

| Source file | Covered facts |
|-------------|---------------|
| `backend/src/app/error/response/integrations/call.rs` | Маппинг `CallError` в HTTP‑статусы и коды ошибок |
| `backend/src/app/error/response/integrations/telegram.rs` | Маппинг `TelegramError`, логирование, вспомогательная функция `internal` |
| `backend/src/app/error/response/integrations/whatsapp.rs` | Маппинг `WhatsappWebError` |
| `backend/src/app/error/response/integrations/yandex_telemost.rs` | Маппинг `YandexTelemostError`, специальная обработка `Settings(is_invalid_request)` |
| `backend/src/app/error/response/integrations/zoom.rs` | Маппинг `ZoomError` |
| `backend/src/app/error/response/knowledge.rs` | Маппинг knowledge‑вариантов `ApiError`, `unreachable!` для не‑knowledge |
| `backend/src/app/error/response/persons.rs` | Маппинг persons‑вариантов `ApiError`, разбор `PersonProjectionError` |
| `backend/src/app/error/response/platform.rs` | Маппинг platform‑вариантов `ApiError`, статусы 409, 503 |
| `backend/src/app/error/response/review.rs` | Маппинг review‑вариантов `ApiError`, вспомогательные функции `bad_request`, `not_found`, `internal_store` |
| `backend/src/app/error/response/tasks.rs` | Маппинг единственного tasks‑варианта `InvalidTaskQuery` |
| `backend/src/app/error/types.rs` | Определение `ApiError` (все варианты) и `AppError` |
| `backend/src/app/guard.rs` | Middleware `require_secret`, проверка заголовка и query‑параметра для WebSocket |
| `backend/src/app/handlers.rs` | Объявления модулей обработчиков |
| `backend/src/app/handlers/automation.rs` | Обработчики шаблонов/политик, `post_telegram_send_dry_run` |
| `backend/src/app/handlers/calendar/accounts.rs` | Обработчики аккаунтов и источников календаря |
| `backend/src/app/handlers/calendar/analytics.rs` | Обработчики аналитики (time distribution, focus balance, back‑to‑back) |
| `backend/src/app/handlers/calendar/brain.rs` | Обработчики «мозга» (брифинг, повестка, недельный брифинг, аналитика) |
| `backend/src/app/handlers/calendar/events.rs` | Ре‑экспорт субмодулей событий |
| `backend/src/app/handlers/calendar/events/agenda.rs` | Обработчики повестки |
| `backend/src/app/handlers/calendar/events/checklist.rs` | Обработчики чек‑листа |
| `backend/src/app/handlers/calendar/events/context_pack.rs` | Обработчики контекстного пакета |
| `backend/src/app/handlers/calendar/events/crud.rs` | CRUD‑обработчики событий |
| `backend/src/app/handlers/calendar/events/participants.rs` | Обработчики участников |
| `backend/src/app/handlers/calendar/events/relations.rs` | Обработчики связей |
| `backend/src/app/handlers/calendar/events/status.rs` | Обработчики переноса и отмены событий |

## Исходные файлы

- [`backend/src/app/error/response/integrations/call.rs`](../../../../backend/src/app/error/response/integrations/call.rs)
- [`backend/src/app/error/response/integrations/telegram.rs`](../../../../backend/src/app/error/response/integrations/telegram.rs)
- [`backend/src/app/error/response/integrations/whatsapp.rs`](../../../../backend/src/app/error/response/integrations/whatsapp.rs)
- [`backend/src/app/error/response/integrations/yandex_telemost.rs`](../../../../backend/src/app/error/response/integrations/yandex_telemost.rs)
- [`backend/src/app/error/response/integrations/zoom.rs`](../../../../backend/src/app/error/response/integrations/zoom.rs)
- [`backend/src/app/error/response/knowledge.rs`](../../../../backend/src/app/error/response/knowledge.rs)
- [`backend/src/app/error/response/persons.rs`](../../../../backend/src/app/error/response/persons.rs)
- [`backend/src/app/error/response/platform.rs`](../../../../backend/src/app/error/response/platform.rs)
- [`backend/src/app/error/response/review.rs`](../../../../backend/src/app/error/response/review.rs)
- [`backend/src/app/error/response/tasks.rs`](../../../../backend/src/app/error/response/tasks.rs)
- [`backend/src/app/error/types.rs`](../../../../backend/src/app/error/types.rs)
- [`backend/src/app/guard.rs`](../../../../backend/src/app/guard.rs)
- [`backend/src/app/handlers.rs`](../../../../backend/src/app/handlers.rs)
- [`backend/src/app/handlers/automation.rs`](../../../../backend/src/app/handlers/automation.rs)
- [`backend/src/app/handlers/calendar/accounts.rs`](../../../../backend/src/app/handlers/calendar/accounts.rs)
- [`backend/src/app/handlers/calendar/analytics.rs`](../../../../backend/src/app/handlers/calendar/analytics.rs)
- [`backend/src/app/handlers/calendar/brain.rs`](../../../../backend/src/app/handlers/calendar/brain.rs)
- [`backend/src/app/handlers/calendar/events.rs`](../../../../backend/src/app/handlers/calendar/events.rs)
- [`backend/src/app/handlers/calendar/events/agenda.rs`](../../../../backend/src/app/handlers/calendar/events/agenda.rs)
- [`backend/src/app/handlers/calendar/events/checklist.rs`](../../../../backend/src/app/handlers/calendar/events/checklist.rs)
- [`backend/src/app/handlers/calendar/events/context_pack.rs`](../../../../backend/src/app/handlers/calendar/events/context_pack.rs)
- [`backend/src/app/handlers/calendar/events/crud.rs`](../../../../backend/src/app/handlers/calendar/events/crud.rs)
- [`backend/src/app/handlers/calendar/events/participants.rs`](../../../../backend/src/app/handlers/calendar/events/participants.rs)
- [`backend/src/app/handlers/calendar/events/relations.rs`](../../../../backend/src/app/handlers/calendar/events/relations.rs)
- [`backend/src/app/handlers/calendar/events/status.rs`](../../../../backend/src/app/handlers/calendar/events/status.rs)

## Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не выявлено.
