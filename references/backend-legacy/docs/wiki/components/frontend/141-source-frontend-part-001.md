---
chunk_id: 141-source-frontend-part-001
batch_id: batch-20260628T214902
group: frontend
role: source
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 141-source-frontend-part-001 — frontend/source

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Чанк предоставляет исходные файлы frontend-части репозитория Макошь. Необходимо обновить или создать страницу `components/frontend.md` в русской Obsidian-вики, отражающую структуру модуля, сборочные скрипты, архитектурные правила, клиентский код и интеграцию с Tauri. На основе встроенных исходников страница должна документировать обнаруженные факты и указать на выявленные расхождения с остальной кодовой базой (например, устаревший скрипт разделения CSS).

## Предложенные страницы

### `components/frontend.md`

```markdown
# Frontend модуль

## Обзор

Модуль `frontend` является десктопным клиентом Макошь на базе Vue 3 и Tauri. Он включает пользовательский интерфейс с несколькими рабочими областями, взаимодействует с backend-сервером через REST API и предоставляет Tauri-команды для интеграции с WhatsApp Web и Яндекс.Телемост.

## Технологический стек

- **PostCSS** с плагинами `tailwindcss` и `autoprefixer` (файл `frontend/postcss.config.js`).
- **TypeScript**, **Vue 3** (composition API), **Pinia** (управление состоянием), **Vue Router** (маршрутизация с `createWebHashHistory`) и **@tanstack/vue-query** (получение данных с кэшированием).
- **Tauri** (Rust-часть в `frontend/src-tauri`) с подключением плагинов `tauri-plugin-shell` и `tauri-plugin-log`.

## Сборка и инструменты

### Скрипты (`frontend/scripts/`)

| Скрипт | Назначение |
|--------|-----------|
| `capture-layout-screenshots.mjs` | Создаёт скриншоты всех представлений (Home, Communications, Mail, Telegram, WhatsApp, Calls, Timeline, Persons, Projects, Tasks, Calendar, Documents, Notes, Knowledge Graph, AI Agents, Settings) для нескольких вьюпортов (800×600, 1024×768, 1366×768, 1920×1080) с помощью Playwright. Фиксирует layout-метрики, ошибки консоли, выравнивание виджетов и мультиколоночные раскладки. Использует фикстуру `layoutStatusFixture` (версия 'layout-capture', поверхности `messages`, `persons`, `search`, `documents`, `account_setup`). Режимы: `baseline`, `after`; по умолчанию подключается к `http://localhost:5174/`. Результаты сохраняются во временную директорию `makosh-layout-{mode}-{timestamp}`. |
| `check-component-lines.mjs` | Проверяет количество строк в продуктовых файлах с расширениями `.ts`, `.tsx`, `.vue`. Исключает тесты (файлы в `__tests__/` и с суффиксами `.test.*`/`.spec.*`) и сгенерированный код (`src/gen/`). Пороги: warning ≥ 500 строк, failure ≥ 700 строк, critical ≥ 1000 строк. При наличии failure-файлов завершается с кодом 1. Может использоваться как CLI или импортироваться. |
| `check-no-inline-styles.mjs` | Запрещает атрибуты `style`, `:style` и `v-bind:style` в шаблонах `.vue` и `.html`. Разрешён динамический `:style` только для файлов из `dynamicLayoutStyleAllowlist` (например, `AttachmentSearchPanel.vue`, `CommunicationList.vue`, `KnowledgeGraphCanvas.vue`, `PersonsList.vue` и др.). Сканирует только секции `<template>` в `.vue`. При нарушениях завершается с кодом 1. |
| `generate-proto.mjs` | Генерирует TypeScript-код из protobuf-определений через `protoc` с плагином `protoc-gen-es`. Входные `.proto` файлы: `contracts/proto/makosh/common/v1/common.proto`, `makosh/events/v1/event_envelope.proto`, `makosh/signal_hub/v1/signal_hub.proto`, `makosh/communications/v1/communications.proto`. Результат помещается в `frontend/src/gen` с опцией `target=ts`. |
| `split-css.py` | Парсит плоский CSS, разбивает на блоки правил и распределяет по тематическим группам на основе префиксов классов. Группы: `vault` (`.vault-`), `sidebar`, `topbar`, `notifications`, `panels`, `pages`. Для каждой группы генерируется отдельный CSS-файл, и определяется инструкция импорта. Обрабатывает вложенные `@media`. **Примечание: скрипт ссылается на Svelte-компоненты и пути (`$lib`, `.svelte`), что не соответствует текущему Vue-коду (см. раздел «Кандидаты на drift»).** |

### Проверки на уровне архитектуры

- Ограничение размера компонентов: скрипт `check-component-lines.mjs` следит, чтобы продуктовые файлы не превышали 700 строк (failure) и тем более 1000 строк (critical). Это встроено в CI.
- Запрет инлайн-стилей: `check-no-inline-styles.mjs` гарантирует, что стили вынесены в CSS-файлы, за исключением разрешённых примитивов динамического layout.

## Tauri-слой (`frontend/src-tauri/`)

- `build.rs` объявляет список Tauri-команд: `open_whatsapp_web_companion`, `whatsapp_web_companion_manifest`, `whatsapp_web_companion_relay_observation`, `open_yandex_telemost_companion` и другие.
- `src/lib.rs` — основная сборка приложения:
  - Подключает плагин `tauri-plugin-shell`.
  - Регистрирует все команды из `whatsapp_companion` и `yandex_telemost_companion`.
  - В режиме отладки включает логирование через `tauri-plugin-log` на уровне `Info`.
  - Создаёт `BackendSidecar` для запуска backend-процесса (sidecar `makosh-backend`) с переменными окружения: `MAKOSH_HTTP_ADDR=127.0.0.1:8080`, `MAKOSH_LOCAL_API_SECRET`, а также опционально пробрасывает `DATABASE_URL`, `MAKOSH_SECRET_VAULT_KEY`, параметры Ollama, Google OAuth, Telegram API ID/Hash, путь к tdlib. Запуск sidecar можно отключить переменной `MAKOSH_DISABLE_BACKEND_SIDECAR`.
  - Пытается найти bundled `tdlib` (под разные платформы macOS arm64/x64, universal) и Google OAuth конфиг (`client_secret.json`).
- `src/main.rs` — точка входа, вызывает `app_lib::run()`, предотвращает дополнительное окно консоли на Windows в релизе.
- `src/whatsapp_companion.rs` — интеграция с WhatsApp Web:
  - Команда `open_whatsapp_web_companion` открывает WebView-окно на `https://web.whatsapp.com/` с предварительным скриптом инициализации и ограничением навигации доменом `web.whatsapp.com`.
  - `whatsapp_web_companion_manifest` возвращает манифест с детальным контрактом: разрешённые наблюдения (например, `runtime_lifecycle_metadata`, `message_identity_metadata`), запрещённые чтения (cookies, web storage, session material), event-поток (`visible_webview_companion -> protected_runtime_bridge -> ...`) и bridge-маршруты.
  - `whatsapp_web_companion_relay_observation` принимает наблюдения, проверяет окно по `account_id`, формирует `runtime_event` и отправляет через HTTP на backend runtime bridge (`/api/v1/integrations/whatsapp/runtime-bridge/runtime-events`).
- `src/yandex_telemost_companion.rs` — интеграция с Яндекс.Телемост:
  - Открытие WebView-окна по join-ссылке, разрешённые хосты: `telemost.yandex.ru`, `telemost.yandex.com`.
  - Управление аудиозаписью через `ffmpeg`: подготовка аудиоустройства, старт/стоп записи. Запись требует явного согласия (`consent_attested: true`).
  - `TelemostLocalRecorder` хранит активные сессии записи, при дропе убивает дочерние процессы.
  - `yandex_telemost_speaker_timeline_append` добавляет записи о говорящих в текстовый и JSONL файлы.

## Маршрутизация (`frontend/src/app/router.ts`)

Используется `createWebHashHistory`. Определены пути:

- `/` → редирект на `/home`
- `/home` → `HomeView.vue`
- `/communications` → `CommunicationsView.vue`
- `/timeline` → `TimelineView.vue`
- `/persons` → `PersonsView.vue`
- `/projects` → `ProjectsView.vue`
- `/tasks` → `TasksView.vue`
- `/calendar` → `CalendarView.vue`
- `/documents` → `DocumentsView.vue`
- `/notes` → `NotesView.vue`
- `/knowledge` → `KnowledgeView.vue`
- `/review` → `ReviewView.vue`
- `/event-tracing` → `EventTracingView.vue`
- `/settings` → `SettingsView.vue`
- `/agents` → `AgentsView.vue`
- `/organizations` → `OrganizationsView.vue`

## Доменная архитектура (`frontend/src/domains/`)

Каждый домен содержит подкаталоги `api`, `queries`, `stores`, `types`, компоненты и, при необходимости, `__tests__`.

### Agents (агенты AI)

- **API** (`domains/agents/api/agents.ts`): запросы на `/api/v1/ai/status`, `/api/v1/ai/agents`, `/api/v1/ai/runs`, `/api/v1/ai/runs/{run_id}`, `/api/v1/persons/owner`, `/api/v1/ai/answers`, `/api/v1/ai/meeting-prep`, `/api/v1/ai/task-candidates/refresh`.
- **Queries** (`domains/agents/queries/useAgentsQuery.ts`): `useAiWorkspaceQuery` (объединяет агентов, runs и owner persona, пытается получить статус AI), `useAiRunsQuery`. Используют `@tanstack/vue-query` с `staleTime=30_000` и `10_000`.
- **Store** (`domains/agents/stores/agents.ts`): Pinia-хранилище `useAgentsStore` управляет выбором агента, отправкой вопроса (`submitAiAnswer`), подготовкой брифа (`prepareAiBrief`) и обновлением кандидатов задач (`refreshTasksFromAi`). Public POST-операции принимают `AiHubRequestAcceptedResponse`, затем store ждёт completion через polling `fetchAiRun(run_id)` и преобразует итоговый `AiRun` в UI-friendly answer / briefing / task-refresh result. Включает утилиты `agentCardView`, `aiAgentPersonaEmail`, `agentVisual`, `aiRuntimeSummary`, `runStatusLabel`, `formatDuration`, `formatDateTime`, `safeCitations`.
- **Types** (`domains/agents/types/agents.ts`): интерфейсы `AiStatus`, `AiAgent`, `AiRun`, `AiHubRequestAcceptedResponse`, `OwnerPersona`, запросы/ответы для AI-операций, `AgentCard` для отображения.

### Calendar (календарь)

- **API** (`domains/calendar/api/calendar.ts`): методы для работы с аккаунтами календаря, источниками, событиями, участниками, контекстными пакетами, повесткой, заметками встреч, исходами, дедлайнами, watchtower, еженедельным брифингом и поиском.
- **Queries** (`domains/calendar/queries/useCalendarEventsQuery.ts`): хуки `useCalendarAccountsQuery` и `useCalendarEventsQuery(limit=200)`.
- **Store** (`domains/calendar/stores/calendar.ts`): `useCalendarStore` управляет режимом просмотра (`day`, `week`, `month`, `agenda`), поиском, формами создания события, выбранным событием, брифингом и повесткой. Экспортируются утилиты форматирования дат (`formatEventDate`, `formatEventTime`, `formatRelativeTime`), тюна типа события (`eventTypeTone`), дни недели и фильтрация событий по неделе.
- **Types** (`domains/calendar/types/calendar.ts`): полные типы для `CalendarAccount`, `CalendarSource`, `CalendarEvent`, `EventParticipant`, `EventContextPack`, `EventAgenda`, `MeetingNote`, `MeetingOutcome`, `DeadlineEvent`, а также `WeeklyBrief` и параметры запросов.

## API-клиент (`frontend/src/platform/api/ApiClient`)

Синглтон `ApiClient`, инициализируемый через `ApiClient.init(baseUrl, secret)`. Особенности (подтверждённые тестами `frontend/src/__tests__/apiClient.test.ts`):

- Отклоняет пустой secret.
- Убирает концевой слеш у `baseUrl`.
- Отправляет HTTP-запросы (`GET`, `POST`, `DELETE`) с заголовком `X-Макошь-Secret`, в POST-запросах — `Content-Type: application/json`.
- Для ответа 204 возвращает `undefined`.
- При не-OK статусе выбрасывает ошибку с полями `message` (текст ответа или fallback) и `status`.

## Безопасность контента

В модуле присутствует санитайзер HTML-сообщений (`frontend/src/__tests__/sanitizeEmailHtml.test.ts` подтверждает поведение):

- Функция `sanitizeEmailHtml` удаляет: теги `<script>`, `<form>`, `<svg>`, атрибуты-обработчики событий (`onclick`, `onmouseover`, `onerror`), атрибут `style`, ссылки с `javascript:`, `mailto:`.
- Сохраняет теги форматирования (`<b>` → `<strong>`, `<i>` → `<em>`, `<a>`, `<blockquote>`, `<table>`), но удаляет небезопасные атрибуты.
- Безопасные ссылки оборачиваются с `target="_blank" rel="noreferrer noopener"`.
- `renderMessageBody` обрабатывает тела сообщений: HTML пропускает через санитайзер, plain text экранирует и заменяет переводы строк на `<br>`.

## Тесты

Тесты используют `vitest`. Примеры наборов:

- `frontend/src/__tests__/apiClient.test.ts` — проверяет инициализацию, заголовки, методы, обработку ответов и ошибок ApiClient.
- `frontend/src/__tests__/sanitizeEmailHtml.test.ts` — проверяет удаление опасного контента и сохранение безопасных элементов.
- `frontend/src/domains/communications/api/aiState.test.ts` — тестирует получение и обновление `ai_state` сообщений.
- `frontend/scripts/check-component-lines.test.mjs` — юнит-тесты для политик проверки строк.
```

## Покрытие источников

- `frontend/postcss.config.js` — конфигурация PostCSS (tailwindcss, autoprefixer).
- `frontend/scripts/capture-layout-screenshots.mjs` — скриншот-скрипт: список представлений, вьюпорты, режимы, метрики, взаимодействие с навигацией, фикстура, временная директория.
- `frontend/scripts/check-component-lines.mjs` — проверка строк: расширения, исключения, пороги, классификация, CLI.
- `frontend/scripts/check-component-lines.test.mjs` — тесты политики проверки строк.
- `frontend/scripts/check-no-inline-styles.mjs` — запрет инлайн-стилей: шаблоны, паттерны, allowlist, сканирование.
- `frontend/scripts/generate-proto.mjs` — генерация из proto: protoc, proto-файлы, выходная директория.
- `frontend/scripts/split-css.py` — разделение CSS: группы (vault, sidebar, topbar, notifications, panels, pages), префиксы, выходные файлы, импорты, ссылки на Svelte-компоненты.
- `frontend/src-tauri/build.rs` — объявление Tauri-команд.
- `frontend/src-tauri/src/lib.rs` — инициализация Tauri-приложения, плагины, боковой процесс backend, переменные окружения, bundled ресурсы.
- `frontend/src-tauri/src/main.rs` — точка входа, атрибут windows subsystem.
- `frontend/src-tauri/src/whatsapp_companion.rs` — WhatsApp-компаньон: команды, манифест, relay, контракты, bridge-маршруты.
- `frontend/src-tauri/src/yandex_telemost_companion.rs` — Телемост-компаньон: открытие окна, аудиоустройства, запись, speaker timeline.
- `frontend/src/__tests__/apiClient.test.ts` — тесты ApiClient: init, secret, заголовки, методы, обработка ответов.
- `frontend/src/__tests__/sanitizeEmailHtml.test.ts` — тесты санитайзера HTML и рендеринга plain text.
- `frontend/src/app/router.ts` — конфигурация Vue Router: hash history, маршруты.
- `frontend/src/config/index.ts` — загрузка frontend-конфигурации.
- `frontend/src/domains/agents/api/agents.ts` — API-функции для AI-агентов.
- `frontend/src/domains/agents/queries/useAgentsQuery.ts` — хуки Vue Query для AI.
- `frontend/src/domains/agents/stores/agents.ts` — Pinia-хранилище AI, утилиты.
- `frontend/src/domains/agents/types/agents.ts` — типы для AI.
- `frontend/src/domains/calendar/api/calendar.ts` — API-функции календаря.
- `frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts` — хуки Vue Query для календаря.
- `frontend/src/domains/calendar/stores/calendar.ts` — Pinia-хранилище календаря, утилиты форматирования.
- `frontend/src/domains/calendar/types/calendar.ts` — типы для календаря.
- `frontend/src/domains/communications/api/aiState.test.ts` — тесты API ai_state для сообщений.

## Исходные файлы

- [`frontend/postcss.config.js`](../../../../frontend/postcss.config.js)
- [`frontend/scripts/capture-layout-screenshots.mjs`](../../../../frontend/scripts/capture-layout-screenshots.mjs)
- [`frontend/scripts/check-component-lines.mjs`](../../../../frontend/scripts/check-component-lines.mjs)
- [`frontend/scripts/check-component-lines.test.mjs`](../../../../frontend/scripts/check-component-lines.test.mjs)
- [`frontend/scripts/check-no-inline-styles.mjs`](../../../../frontend/scripts/check-no-inline-styles.mjs)
- [`frontend/scripts/generate-proto.mjs`](../../../../frontend/scripts/generate-proto.mjs)
- [`frontend/scripts/split-css.py`](../../../../frontend/scripts/split-css.py)
- [`frontend/src-tauri/build.rs`](../../../../frontend/src-tauri/build.rs)
- [`frontend/src-tauri/src/lib.rs`](../../../../frontend/src-tauri/src/lib.rs)
- [`frontend/src-tauri/src/main.rs`](../../../../frontend/src-tauri/src/main.rs)
- [`frontend/src-tauri/src/whatsapp_companion.rs`](../../../../frontend/src-tauri/src/whatsapp_companion.rs)
- [`frontend/src-tauri/src/yandex_telemost_companion.rs`](../../../../frontend/src-tauri/src/yandex_telemost_companion.rs)
- [`frontend/src/__tests__/apiClient.test.ts`](../../../../frontend/src/__tests__/apiClient.test.ts)
- [`frontend/src/__tests__/sanitizeEmailHtml.test.ts`](../../../../frontend/src/__tests__/sanitizeEmailHtml.test.ts)
- [`frontend/src/app/router.ts`](../../../../frontend/src/app/router.ts)
- [`frontend/src/config/index.ts`](../../../../frontend/src/config/index.ts)
- [`frontend/src/domains/agents/api/agents.ts`](../../../../frontend/src/domains/agents/api/agents.ts)
- [`frontend/src/domains/agents/queries/useAgentsQuery.ts`](../../../../frontend/src/domains/agents/queries/useAgentsQuery.ts)
- [`frontend/src/domains/agents/stores/agents.ts`](../../../../frontend/src/domains/agents/stores/agents.ts)
- [`frontend/src/domains/agents/types/agents.ts`](../../../../frontend/src/domains/agents/types/agents.ts)
- [`frontend/src/domains/calendar/api/calendar.ts`](../../../../frontend/src/domains/calendar/api/calendar.ts)
- [`frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts`](../../../../frontend/src/domains/calendar/queries/useCalendarEventsQuery.ts)
- [`frontend/src/domains/calendar/stores/calendar.ts`](../../../../frontend/src/domains/calendar/stores/calendar.ts)
- [`frontend/src/domains/calendar/types/calendar.ts`](../../../../frontend/src/domains/calendar/types/calendar.ts)
- [`frontend/src/domains/communications/api/aiState.test.ts`](../../../../frontend/src/domains/communications/api/aiState.test.ts)

## Кандидаты на drift

1. **Скрипт `split-css.py` ссылается на Svelte-компоненты и пути, не соответствующие стеке Vue-приложения.** В словаре `IMPORTS` указаны такие импорты, как `import './vault.css';` в файлах `VaultOnboarding.svelte`, `Sidebar.svelte`, `Topbar.svelte`, `NotificationsDrawer.svelte`, `WidgetEditChrome.svelte`, а также путь `$lib/pages/pages.css` в `+layout.svelte`. Однако в исходниках фронтенда используются Vue-компоненты (`.vue`) и структура `src/app/`, `src/domains/`, без `src/lib` и Svelte-файлов. Это свидетельствует о том, что скрипт либо устарел, либо принадлежит предыдущей итерации на SvelteKit и не соответствует текущей кодовой базе.

2. **Список представлений в `capture-layout-screenshots.mjs` расходится с маршрутами в `router.ts`.** Скрипт захвата перечисляет: Home, Communications, Mail, Telegram, WhatsApp, Calls, Timeline, Persons, Projects, Tasks, Calendar, Documents, Notes, Knowledge Graph, AI Agents, Settings. При этом маршрутизатор содержит пути `/review`, `/event-tracing`, `/agents` (вместо "AI Agents"), `/organizations`, но не содержит отдельных подстраниц для Mail/Telegram/WhatsApp/Calls как самостоятельных маршрутов. Это может указывать на то, что скриншот-тесты не покрывают новые представления, либо интерфейс переработан.

3. **CSS-группа `pages` в `split-css.py` включает селекторы для `.review-` не отражено.** Конкретный префикс `.review-` не виден в предоставленных фрагментах (файл обрезан), но наличие в роутере `ReviewView` без соответствующей CSS-зоны может говорить о недостаточной детализации. Утверждать можно только в рамках обрезанного контекста, но это потенциальное несоответствие.

4. **Tauri-команда `whatsapp_web_companion_relay_observation` ожидает окно с меткой `whatsapp-companion-{account_id}`, однако в коде открытия окна используется `companion_window_label` с префиксом `WINDOW_LABEL_PREFIX = "whatsapp-companion"`. Расхождений не видно, но проверка целостности требует анализа полного файла (усечён). В предоставленной части конфликтов не обнаружено.
