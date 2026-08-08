---
chunk_id: 093-test-backend-part-016
batch_id: batch-20260628T214902
group: backend
role: test
source_status: pending
source_count: 10
generated_by: code-wiki-ru
---

# 093-test-backend-part-016 — backend/test

- Target index: [[operations/backend-tests]]
- Batch: `batch-20260628T214902`
- Source files: `10`

## Резюме

Страница `operations/backend-tests.md` должна описывать структуру и цели интеграционных тестов бэкенда, покрытых в предоставленных исходных файлах. На основе встроенных тестовых модулей нужно задокументировать, какие аспекты системы проверяются (шаблоны коммуникаций, API управления workflow, доменное API v2, интеграции с WhatsApp, Zoom, Yandex Telemost, а также правила верификации live‑smoke‑свидетельств). Страница пишется на русском языке в формате Obsidian Markdown, ссылаясь только на факты из самих тестов.

## Предложенные страницы

#### `operations/backend-tests.md`

```markdown
---
tags:
  - backend
  - testing
  - operations
---

# Backend‑тесты

Обзор ключевых интеграционных тестов бэкенда Макошь‑Hub.
Каждый тестовый файл проверяет контракты API, проекции событий или взаимодействие подсистем.
Описание базируется исключительно на коде тестов, встроенном в контекст репозитория.

## `v1_communications_templates.rs`

Тесты API шаблонов богатых сообщений (rich templates) версии 1.

Покрываемые операции:

- **Сохранение (save)** – `POST /api/v1/communications/templates/rich` с полями `template_id`, `name`, `subject_template`, `body_template`, `variables`, `language`. Ответ содержит `template_id`, массив `placeholder_variables`, а также `undeclared_variables`, `unused_variables` и `malformed_placeholders` (все пустые при корректном запросе).
- **Список (list)** – `GET /api/v1/communications/templates/rich` возвращает все шаблоны; проверяется наличие сохранённого шаблона и его `placeholder_variables`.
- **Рендеринг (render)** – `POST /api/v1/communications/templates/rich/render` с `template_id` и словарём `variables`. Ответ включает поля `rendered.subject` и `rendered.body` с подставленными значениями, а также `missing_variables`, `unresolved_variables`, `malformed_placeholders`.
- **Предпросмотр рассылки (mail‑merge preview)** – `POST /api/v1/communications/templates/rich/mail-merge-preview` с `template_id` и массивом `rows`. Ответ содержит `template_id`, `row_count`, `ready_count`, `blocked_count` и массив `items` с признаком `ready` и рендером (или набором отсутствующих переменных, например, `missing_variables: ["status"]`).
- **Удаление (delete)** – `DELETE /api/v1/communications/templates/rich/{template_id}`. Ответ включает `template_id` и `deleted: true`. После удаления шаблон отсутствует в списке.

Аутентификация: заголовок `x-makosh-secret`.

## `v1_workflow_actions.rs`

Тесты API управления рабочим процессом (workflow actions) версии 1.

Ключевые аспекты:

- **Эндпоинт без БД** – `POST /api/v1/workflow-actions` возвращает `503 SERVICE_UNAVAILABLE`, если база данных не сконфигурирована.
- **Смена состояния сообщения** – `PUT /api/v1/communications/messages/{message_id}/workflow-state` с телом `{"workflow_state": "reviewed"}`:
  - Возвращает `message_id`, `workflow_state` и `previous_state`.
  - В базе данных обновляется колонка `workflow_state` таблицы `communication_messages`.
  - Создаётся запись в `observation_links` с relationship_kind = `workflow_state_transition`, содержащая метаданные `previous_state` и `workflow_state`.
  - Запись в `observations` имеет `origin_kind = "manual"`.
- **Создание задачи (create_task)** – `POST /api/v1/workflow-actions` с `action: "create_task"` и `source: {kind: "communication_message"}`:
  - Идемпотентно: повторный вызов возвращает тот же `target`.
  - Создаётся ровно одна задача (`tasks` source_type = `observation`).
  - В `observation_links` образуются две связи: `task_create` и `workflow_action_projection`.
  - В `event_log` сохраняется событие с event_id = `"workflow_action:{command_id}"`, при этом в payload не должно быть строки `"Body for local trash API"`.
- **Создание контакта (create_contact)** – `POST /api/v1/workflow-actions` с `action: "create_contact"`:
  - Возвращает `target.kind = "person"`.
  - В `observation_links` появляются записи для `persona` и `identity` с relationship_kind = `workflow_action_projection`.
- **Создание заметки** – тест `workflow_action_create_note_creates_markdown_document` присутствует (детали обрезаны в предоставленном контексте).

Аутентификация помимо `x-makosh-secret` включает заголовок `x-makosh-actor-id` для передачи идентификатора инициатора.

## `v2_domain_api.rs`

Тесты маршрутов доменного API (задачи, персоны).

- **Проверка секрета** – запросы без `x-makosh-secret` возвращают `403` с `error: "invalid_api_secret"`. При наличии секрета, но отключённой БД, возвращается `503` с `error: "database_not_configured"`.
- **Список задач** – `GET /api/v1/tasks?limit=100` (PostgreSQL). Для задачи, созданной через `TaskStore::create`, проверяется, что ответ содержит поля `task_id`, `title`, `source_type` (ожидается `"observation"`), `makosh_status`, `confidentiality` (`"private_local"`), `task_metadata` (пустой объект).
- **Здоровье персоны** – `GET /api/v1/persons/{person_id}/health`. Проверяется, что ответ является единичным (не коллекцией), содержит `person_id`, `health_status` (`"at_risk"`), `communication_gap_days` (`42`) и не содержит массив `items`.

## `whatsapp.rs`

Тесты интеграции с WhatsApp (веб‑клиент и Business Cloud).

- **Provider‑ и secret‑kind** – `CommunicationProviderKind::try_from("whatsapp_web")` → `WhatsappWeb`; аналогично для `"whatsapp_business_cloud"`. Метод `is_whatsapp()` возвращает `true`, `is_email()` / `is_telegram()` – `false`.
  `ProviderAccountSecretPurpose::WhatsappWebSessionKey` принимает секреты типа `PrivateKey` и `Other`, отклоняет `Password` и `ApiToken`.
- **Фикстура WhatsApp Business Cloud** – созданная через `/api/v1/integrations/whatsapp/fixtures/accounts` возвращает:
  - `session.companion_runtime = "api_credentials"`
  - `session.link_state = "fixture"`
  - `session.metadata.provider_shape = "whatsapp_business_cloud"`
  - `session.metadata.setup_semantics = "business_cloud"`, `session_mode = "api_credentials"`
- **Фикстура WhatsApp Native MD** – создаётся с `provider_shape: "whatsapp_native_md"`. Затем проверяется:
  - Статус через `/api/v1/integrations/whatsapp/runtime/status?account_id=…` содержит `runtime_kind = "fixture"`.
  - Сообщение, отправленное через `/api/v1/integrations/whatsapp/fixtures/messages`, сохраняется и появляется в агрегатных маршрутах: список сессий (`/api/v1/integrations/whatsapp/sessions`) и список сообщений (`/api/v1/communications/messages?channel_kind=whatsapp`).
- **Нейтральные коммуникационные маршруты** – тест `whatsapp_provider_neutral_communications_routes_dispatch_to_whatsapp_commands` (находится в предоставленном фрагменте, детали обрезаны).

Аутентификация: заголовок `x-makosh-secret`.

## `whatsapp_signal_hub.rs`

Тесты системы сигналов для WhatsApp (Signal Hub) – проверки документированных контрактов и скриптов live‑smoke‑верификации.

- **Матрица фикстур** – для каждой фикстуры проверяется, что:
  - В коде `signal_hub/whatsapp.rs` присутствует маппинг raw‑record‑kind → signal‑kind.
  - Файл `docs/integrations/whatsapp/fixture-test-matrix.md` содержит метку фикстуры.
  - Обработчики событий в runtime содержат соответствующую константу события реального времени.
- **События жизненного цикла** – подтверждается, что для всех типов событий (`RUNTIME_STATUS_CHANGED`, `SYNC_STARTED`, `MEDIA_UPLOAD_REQUESTED` и т.д.) в `event_bus` определена константа, а провайдерный обработчик способен их генерировать. Функция `sanitize_event_payload` удаляет ключи `session_key`, `access_token`, `password`, `raw_body`.
- **Live‑smoke‑свидетельства** – скрипт‑валидатор `scripts/whatsapp-live-smoke-evidence.mjs` требует префиксы типа `'raw_record:'`, `'event_log:'`, `'command:'` и т.д. Для outbound‑команд обязательно наличие `command:` плюс provider‑observed‑событий (event_log / signal_hub). Документация (`live-smoke-checklist.md`, `status.md`, `whatsapp-live-smoke-readiness.mjs`) явно фиксирует этот ужесточённый контракт.
- **Коллектор свидетельств** – скрипт `scripts/whatsapp-live-smoke-collect-evidence.mjs` нормализует наблюдения, проверяет через валидатор и не создаёт синтетических прохождений; gate остаются pending без настоящих sanitized‑refs.
- **Путь обновления Native MD** – скрипт `scripts/whatsapp-native-md-sdk-gap-readiness.mjs` делает обновление Rust/wa‑rs исполняемым доказательством (проверка `cargo info`, `MAKOSH_WA_RS_CRATES_IO_PROBE=1`). Файлы `Cargo.toml` и `Cargo.lock` зафиксированы с версиями `wa-rs = 0.2.0`, `wa-rs-core = 0.2.0`.

## `yandex_telemost_calendar_matching.rs`

Тест проекции созвонов Yandex Telemost на участников события календаря.

- Создаётся событие календаря с `conference_url` (yandex‑telemost) и провайдером `"yandex_telemost"`.
- Формируется событие проекции с типом `yandex_telemost_event_types::COHOSTS_OBSERVED`, содержащее список `cohosts` (email‑адреса, в том числе дубли в разном регистре).
- После вызова `project_yandex_telemost_calendar_matching` в таблице `event_participants` появляются два уникальных участника с `role = "attendee"` и `source = "yandex_telemost_cohost_observed"`, адреса – в нижнем регистре, без дублей.

## `zoom_calendar_matching.rs`

Тест связывания встреч Zoom с событиями календаря.

- Создаётся событие календаря с `conference_url` (zoom‑ссылка), провайдером `"zoom"` и типом `"meeting"`.
- Через `ZoomStore::setup_fixture_account` и `observe_meeting` создаётся аккаунт и наблюдение встречи.
- Функция `CalendarEventStore::find_zoom_conference_match` находит соответствующее событие календаря по `join_url`, `meeting_id` и временным рамкам.
- После вызова `project_zoom_calendar_matching` в `event_relations` появляется связь с `entity_type = "call"`, `relation_type = ZOOM_CALENDAR_RELATION_TYPE`, `source = "zoom.meeting.observed"`.
- Повторная проекция не создаёт дубликатов.

## `zoom_participant_identity.rs`

Тест идентификации участников Zoom и ревью‑кандидатов.

- Создаётся персона с `display_name` и email.
- В наблюдение встречи добавляется участник с таким же `display_name` и новым email.
- Проекция `project_zoom_participant_identity` создаёт запись в `person_identity_candidates` с `candidate_kind = "attach_email_address"`, `review_state = "suggested"`, `evidence_summary` содержит email, а также событие `person_identity.candidate.detected`.
- Последующий вызов `project_person_identity_review_event` помещает кандидата в `review_items` с `item_kind = "identity_candidate"` и соответствующими метаданными.

## `zoom_provider_foundation.rs`

Базовая проверка провайдера Zoom и его жизненного цикла (фрагмент обрезан после 12000 символов).

- **Provider‑ и secret‑kind** – `CommunicationProviderKind::try_from("zoom_user")` → `ZoomUser`, аналогично для `"zoom_server_to_server"`. Метод `is_zoom()` истинен, `is_email()` и `is_telegram()` ложны.
  `ProviderAccountSecretPurpose::ZoomOauthToken` принимает `OauthToken`, `ZoomClientSecret` – `ApiToken`, `ZoomWebhookSecret` – `ApiToken`; другие сочетания отвергаются.
- **Capabilities** – эндпоинт `/api/v1/integrations/zoom/capabilities` возвращает массив `capabilities`, включающий как минимум `token_maintenance.scheduler`, `provider_sync.recordings.scheduler`, `recording_imports.remove`, `retention.cleanup`, `retention.cleanup.scheduler`, `auth.token_rotation_policy`, `calendar_event_matching`, `meeting_participant_identity_resolution` – все со статусом `"available"`. Планируемые фичи (`planned_features`) не содержат `calendar_event_matching` и `meeting_participant_identity_resolution` (то есть они уже реализованы).
- **Жизненный цикл** – аккаунт создаётся через `/api/v1/integrations/zoom/fixtures/accounts`, после чего:
  - Статус runtime (`/runtime/status`) возвращает `status: "stopped"`, `healthy: true`.
  - Запуск (`/runtime/start`) переводит в `"running"`.
  - Остановка (`/runtime/stop`) возвращает `"stopped"`.
  - Удаление (`/runtime/remove`) даёт `removed: true`.
  - Список активных аккаунтов (`/accounts`) пуст, а с флагом `include_removed=true` содержит аккаунт с `lifecycle_state: "removed"`.
- **Фильтрация звонков** – маршрут `/api/v1/calls?account_id=…&provider=zoom` возвращает только zoom‑связанные встречи, игнорируя generic‑звонки с другим провайдером.
- **Live‑регистрация** – тест `zoom_live_account_registration_is_blocked_and_uses_secret_bindings` начат, но детали обрезаны.

## `zoom_signal_detection.rs`

Тест отображения встреч Zoom в систему сигналов (Signal Hub).

- При проекции через `project_zoom_signal_detection` создаются события:
  - `signal.raw.zoom.meeting.observed` – `subject` содержит `source_code = "zoom"`, `account_id`, `entity_id` (call_id), `meeting_id`; `provenance.source = "zoom_signal_detection"`.
  - `signal.accepted.zoom.meeting` – `provenance.signal_hub.decision = "accepted"`, ссылается на `raw_event_id`.
- При активном профиле `"testing"` (через `SignalHubProfileService::apply_profile`) вместо принятого сигнала генерируется `signal.muted.zoom.meeting` с решением `"muted"` и причиной `"testing profile mutes Zoom signals"`. Принятые сигналы отсутствуют.
```

## Покрытие источников

| Source File | Covered Facts |
|-------------|---------------|
| `backend/tests/v1_communications_templates.rs` | Сохранение, список, рендеринг, mail‑merge preview и удаление rich‑шаблонов через API v1; проверка полей `placeholder_variables`, `undeclared_variables`, `unused_variables`, `malformed_placeholders`; проверка рендеринга с подстановкой переменных; предпросмотр рассылки с `ready`/`blocked` и `missing_variables`; удаление шаблона и его отсутствие в последующем списке. |
| `backend/tests/v1_workflow_actions.rs` (частично, первые 12000 символов) | Эндпоинт workflow‑actions без БД возвращает `503`; смена состояния сообщения через PUT с сохранением `observation_links` и `origin_kind = "manual"`; идемпотентное создание задачи с проверкой `tasks`, `observation_links` (`task_create`, `workflow_action_projection`), `event_log` без утечки определённой строки; создание контакта с генерацией persona‑ и identity‑observation‑link; наличие теста создания заметки (детали обрезаны). |
| `backend/tests/v2_domain_api.rs` | Проверка обязательности `x-makosh-secret` (403) и требования БД (503); GET‑tasks возвращает поля задачи с `source_type = "observation"`, `confidentiality = "private_local"`; GET‑person‑health возвращает одиночный результат с `health_status`, `communication_gap_days` и без массива `items`. |
| `backend/tests/whatsapp.rs` (частично, первые 12000 символов) | `CommunicationProviderKind` для `whatsapp_web` / `whatsapp_business_cloud`, `is_whatsapp`, `is_email`, `is_telegram`; `ProviderAccountSecretPurpose::WhatsappWebSessionKey` принимает/отклоняет типы секретов; фикстура WhatsApp Business Cloud возвращает `session.companion_runtime = "api_credentials"` и соответствующие метаданные; фикстура Native MD проверяет `provider_shape`, появление в агрегатных маршрутах сессий и сообщений. |
| `backend/tests/whatsapp_signal_hub.rs` (частично, первые 12000 символов) | Проверка маппинга в `signal_hub/whatsapp.rs` и наличия меток в матрице фикстур; проверка констант событий event‑bus и их генерации обработчиком; санитизация `sanitize_event_payload` удаляет секретные ключи; требования к live‑smoke‑evidence (валидатор с префиксами, обязательность `command:` + provider‑observed‑refs); коллектор как не‑bypass‑путь; проверка обновления Native MD через `cargo info` и зафиксированные версии `wa-rs = 0.2.0`. |
| `backend/tests/yandex_telemost_calendar_matching.rs` | Проекция cohosts‑события в участников календаря: два уникальных участника с `role = "attendee"` и `source = "yandex_telemost_cohost_observed"`, дедупликация email. |
| `backend/tests/zoom_calendar_matching.rs` | Создание встречи Zoom, поиск соответствия календарному событию через `find_zoom_conference_match`; проекция `project_zoom_calendar_matching` создаёт `event_relations` с `entity_type = "call"` и `relation_type = ZOOM_CALENDAR_RELATION_TYPE`; идемпотентность повторной проекции. |
| `backend/tests/zoom_participant_identity.rs` | Создание персонажа и участника с совпадающим `display_name`; проекция `project_zoom_participant_identity` порождает кандидата `identity_candidate` типа `attach_email_address` и событие `person_identity.candidate.detected`; последующая проекция в `review_items`. |
| `backend/tests/zoom_provider_foundation.rs` (частично, первые 12000 символов) | `CommunicationProviderKind` для `zoom_user`/`zoom_server_to_server`, методы `is_zoom`/`is_email`/`is_telegram`; `ProviderAccountSecretPurpose` и принимаемые типы секретов; capabilities‑эндпоинт и его состав; жизненный цикл аккаунта (создание, старт/стоп/удаление, фильтрация); фильтрация звонков по провайдеру. |
| `backend/tests/zoom_signal_detection.rs` | Проекция сигналов Zoom: события `signal.raw.zoom.meeting.observed` и `signal.accepted.zoom.meeting`; при тестовом профиле – `signal.muted.zoom.meeting` с причиной, отсутствие `accepted`. |

## Исходные файлы

- [`backend/tests/v1_communications_templates.rs`](../../../../backend/tests/v1_communications_templates.rs)
- [`backend/tests/v1_workflow_actions.rs`](../../../../backend/tests/v1_workflow_actions.rs)
- [`backend/tests/v2_domain_api.rs`](../../../../backend/tests/v2_domain_api.rs)
- [`backend/tests/whatsapp.rs`](../../../../backend/tests/whatsapp.rs)
- [`backend/tests/whatsapp_signal_hub.rs`](../../../../backend/tests/whatsapp_signal_hub.rs)
- [`backend/tests/yandex_telemost_calendar_matching.rs`](../../../../backend/tests/yandex_telemost_calendar_matching.rs)
- [`backend/tests/zoom_calendar_matching.rs`](../../../../backend/tests/zoom_calendar_matching.rs)
- [`backend/tests/zoom_participant_identity.rs`](../../../../backend/tests/zoom_participant_identity.rs)
- [`backend/tests/zoom_provider_foundation.rs`](../../../../backend/tests/zoom_provider_foundation.rs)
- [`backend/tests/zoom_signal_detection.rs`](../../../../backend/tests/zoom_signal_detection.rs)

## Кандидаты на drift

Из предоставленного контекста явных расхождений (code/docs/ADR drift) не видно. Однако можно отметить следующие потенциальные риски, которые не подтверждены, но могут возникнуть при несинхронизированных изменениях:

- Тесты `whatsapp_signal_hub.rs` содержат множество проверок на точные строковые вхождения в исходных файлах (например, `signal_hub_whatsapp.contains(...)`). Если соответствующие исходные файлы будут реорганизованы или переименованы константы, эти тесты станут маркером drift‑а.
- Тест `v1_workflow_actions.rs::workflow_action_create_task_is_idempotent_and_records_safe_event` проверяет отсутствие строки `"Body for local trash API"` в payload event лога. Если эта строка изменится в коде домена, тест перестанет ловить намеренное исключение и может потребоваться актуализация.
- В обрезанных частях `whatsapp.rs` и `zoom_provider_foundation.rs` могут находиться тесты, которые уже не соответствуют текущей реализации API, но из видимого контекста это не подтверждается.
- Ожидаемый результат `tasks_endpoint` проверяет `source_type = "observation"`, что может отражать внутреннюю конверсию (создаётся задача с `source_type = "manual"`, но API возвращает другое значение). Если конверсия изменится, тест укажет на drift.
