# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `127-doc-frontend`
- Group / Группа: `frontend`
- Role / Роль: `doc`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/documentation-map.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `frontend/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/README.md`
- Size bytes / Размер в байтах: `5427`
- Included characters / Включено символов: `5399`
- Truncated / Обрезано: `no`

````markdown
# Frontend

Vue 3 + TypeScript desktop UI for Макошь, packaged by Tauri.

Current scope is a desktop/laptop shell for the local backend APIs with provider account setup wizards for Gmail, iCloud and raw IMAP, graph/project/task/Persona identity/document workflow surfaces, and local AI workflow surfaces. Mobile UI is out of scope while ADR-0031 is active.

## UI Styling Contract

The app-level CSS is loaded in this order from `src/App.vue`:

1. `src/assets/styles/tokens.css` defines design tokens and browser root defaults.
2. `src/assets/styles/app.css` defines global shell, view, state and responsive styles.

All components use scoped `<style>` blocks and Tailwind utility classes. No inline `style=` attributes in production components.

The supported desktop window minimum is `800 x 600`. At smaller widths or heights, a viewport guard is shown instead of the app. This is a desktop window constraint, not mobile UI support; ADR-0031 still keeps mobile design and validation out of scope.

## Scaffold

The frontend was scaffolded as a new Vue 3 + TypeScript + Vite project:

```sh
pnpm create vue@latest . -- --typescript --force
pnpm install
```

Tauri was initialized with:

```sh
pnpm tauri init --ci --app-name "Макошь" --window-title "Макошь" --frontend-dist "../dist" --dev-url "http://localhost:5173" --before-dev-command "pnpm dev" --before-build-command "pnpm build"
```

## Commands

```sh
make dev
make logs
make build
make migrate
make clean
make clean-vault
```

`make dev` is the supported desktop development loop. It starts PostgreSQL in
Docker, runs the backend via repo-local `bacon`, starts this Vue 3 + Vite
frontend natively at
`http://127.0.0.1:5174`.

The active session also exposes an aggregated plain-text log at:

```sh
make logs
```

`make build` is the supported release packaging entrypoint. It builds the
frontend, builds the backend release binary, prepares bundled Google OAuth,
TDLib and backend sidecar resources internally, and then runs `pnpm tauri
build`.

## Bundled TDLib Runtime

macOS release builds package the Telegram TDLib JSON runtime from
`frontend/src-tauri/resources/tdlib/`. Generated `libtdjson.dylib` files are not
committed; `make build` prepares the resource automatically before `tauri build`:

```sh
make build
```

The internal build step copies `libtdjson.dylib` from, in order:

1. `MAKOSH_TDJSON_SOURCE`
2. `MAKOSH_TDJSON_PATH`
3. Homebrew `tdlib`
4. `/opt/homebrew/lib/libtdjson.dylib`
5. `/usr/local/lib/libtdjson.dylib`

Release CI can build TDLib from source instead of relying on a system install:

```sh
MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make build
```

The backend still accepts `MAKOSH_TDJSON_PATH` as a development override, but a
packaged macOS app should resolve TDLib from the bundled Tauri resource path.
Linux is supported only as a development/container target and is not packaged as
a desktop TDLib bundle.

Telegram QR login also needs Telegram app credentials. Development runs can set
`MAKOSH_TELEGRAM_API_ID` and `MAKOSH_TELEGRAM_API_HASH` in the backend
environment. Packaged macOS builds can inject them into the Tauri launcher with
`MAKOSH_BUNDLED_TELEGRAM_API_ID` and `MAKOSH_BUNDLED_TELEGRAM_API_HASH`; the
launcher forwards those values to the backend sidecar as runtime
`MAKOSH_TELEGRAM_API_ID` and `MAKOSH_TELEGRAM_API_HASH`.

Google mail setup needs one project-owned OAuth Desktop app client. End users of
the packaged app should not create their own Google Cloud project. Release builds
copy the downloaded Desktop app JSON into the Tauri resource bundle:

```sh
make build
```

The internal build step reads `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` from
`docker/.env`, or `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` from the shell, and
copies the file to `frontend/src-tauri/resources/google-oauth/client_secret.json`.
That generated resource is ignored by Git. The packaged launcher passes the bundled resource path to the backend sidecar as
`MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.

## Bundled Backend Sidecar

macOS release builds package the Rust backend as a Tauri sidecar from
`frontend/src-tauri/binaries/`. Generated sidecar binaries are not committed;
`make build` prepares the current host binary before `tauri build`:

```sh
make build
```

## Architecture

The frontend is organized by domain under `src/domains/`:

- `src/domains/` — 14 domain modules (home, settings, personas, organizations, projects, tasks, calendar, documents, notes, knowledge, review, agents, timeline, communications, telegram, whatsapp)
- `src/shared/` — Shared UI primitives, stores, composables
- `src/platform/` — Platform abstractions (API client, SSE, routing, i18n, theming)
- `src/app/` — App shell, layout, view routing

Each domain follows a consistent structure:

```
domains/<name>/
  types/<name>.ts      — TypeScript interfaces
  api/<name>.ts        — API functions
  queries/<name>.ts    — TanStack Query hooks
  stores/<name>.ts     — Pinia store
  components/          — Vue 3 SFC components
  views/<name>.ts      — Page-level view component
```

Data flow: API → TanStack Query → Component (direct) or API → Pinia Store → Component.

Requests use `X-Макошь-Secret: <secret>` via the centralized `ApiClient` (see `src/platform/api/ApiClient.ts`).

Validate frontend packaging changes with:

```sh
make build
```

For the normal full-stack desktop workflow:

```sh
make dev
```
````

### `frontend/docs/code-review-2026-06-14.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/docs/code-review-2026-06-14.md`
- Size bytes / Размер в байтах: `68271`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```markdown
# Полное ревью фронтенда Макошь

**Дата:** 2026-06-14
**Объём:** 80+ файлов прочитано (конфигурация, API, типы, эндпоинты, сервисы, стор-файлы, страницы, тесты, i18n, стили)
**Вердикт:** Фундамент качественный, но есть критические проблемы, которые нужно исправить перед расширением.

---

## Дополнение: code review текущей Vue 3 миграции

**Дата проверки:** 2026-06-14 16:47 CEST
**Объём:** новый Vue 3 bootstrap, platform API/SSE/i18n, communications HTML rendering, Calendar/Tasks view data flow, frontend validation gate.
**ADR:** ADR-0093, ADR-0031, ADR-0056, ADR-0077.

### Critical: `ApiClient` не инициализируется перед использованием

**Статус:** Открыто.

**Файлы:** `frontend/src/main.ts`, `frontend/src/platform/api/ApiClient.ts`, `frontend/src/domains/*/api/*.ts`

Новый entrypoint создает Vue app, Pinia, Vue Query and router, но не вызывает `ApiClient.init(...)`. При этом `ApiClient.instance` бросает `ApiClient not initialized. Call ApiClient.init() first.`, а все domain API modules используют именно `ApiClient.instance`.

**Evidence:** `frontend/src/main.ts:8-14`, `frontend/src/platform/api/ApiClient.ts:72-83`, `rg "ApiClient\\.init" frontend/src` находит только сам метод.

**Impact:** любой экран, который загружает backend data через domain API, падает на runtime до network request. `vue-tsc` и `vite build` это не ловят, потому что контракт bootstrap singleton-а не покрыт тестом.

**Fix:** инициализировать `ApiClient` в bootstrap до mount/router-driven queries, используя validated config. Добавить regression test на bootstrap/config path: без secret fail-fast, с secret `ApiClient.instance` доступен.

### Critical: backend URL и secret env names расходятся с Makefile/backend contract

**Статус:** Открыто.

**Файлы:** `frontend/src/config/index.ts`, `Makefile`, `frontend/src-tauri/src/lib.rs`

`frontend-dev` передает `VITE_MAKOSH_API_BASE_URL` и `VITE_MAKOSH_LOCAL_API_SECRET`, backend sidecar стартует на `127.0.0.1:8080`, но новый config читает `VITE_API_BASE_URL` / `VITE_MAKOSH_API_SECRET` и default-ит API base URL в `http://localhost:3000`.

**Evidence:** `frontend/src/config/index.ts:4-13`, `Makefile:277-280`, `frontend/src-tauri/src/lib.rs:53`.

**Impact:** даже после исправления `ApiClient.init` frontend по умолчанию будет ходить не в Макошь backend и отправлять пустой `X-Макошь-Secret`. Protected endpoints по ADR-0056 будут недоступны, а dev loop через `make frontend-dev` не сможет поднять рабочий UI без ручного дублирования env vars.

**Fix:** вернуть единый public env contract: `VITE_MAKOSH_API_BASE_URL` with default `http://127.0.0.1:8080` and required `VITE_MAKOSH_LOCAL_API_SECRET`; пустой secret должен быть bootstrap error, а не silently accepted config.

### Critical: renderer писем снова вставляет непроверенный HTML через `innerHTML`

**Статус:** Открыто.

**Файл:** `frontend/src/domains/communications/components/MessageBodyTab.vue`

`sanitizeEmailHtml` удаляет несколько unsafe block tags, но не удаляет event-handler attributes (`onerror`, `onclick`), `javascript:` URLs, dangerous `src`/`href` schemes or inline CSS. Затем результат вставляется через `body.innerHTML`. Plain-text branch внутри shadow renderer также делает `bodyText.value.replace(/\n/g, '<br>')` and assigns it to `innerHTML` without escaping.

**Evidence:** `frontend/src/domains/communications/components/MessageBodyTab.vue:51-70`, `frontend/src/domains/communications/components/MessageBodyTab.vue:91-95`.

**Impact:** импортированное письмо является untrusted input. HTML вроде `<img src=x onerror=...>` или текстовое тело с HTML-like payload может выполнить script/event handler в основном renderer context, а не только в sandbox iframe. Это регрессия относительно mail rendering boundary, где message bodies должны быть sanitized/proxied before display.

**Fix:** не поддерживать sanitizer regex-цепочкой. Перенести проверенную allowlist sanitation из старого renderer или использовать DOM-based sanitizer boundary, удалить unsafe attributes/schemes, escape plain text before `innerHTML`, and cover with regression tests for event handlers, `javascript:` links and malformed HTML.

### Major: default rendered view для HTML-писем пустой

**Статус:** Открыто.

**Файл:** `frontend/src/domains/communications/components/MessageBodyTab.vue`

Для HTML email default path показывает `shadow-frame-wrapper`, но `renderShadowContent()` вызывается только в `onMounted`, когда `!isHtmlEmail`. Для HTML bodies renderer не вызывается ни при mount, ни при смене message, ни при переключении обратно с iframe view.

**Evidence:** `frontend/src/domains/communications/components/MessageBodyTab.vue:98-102`, `frontend/src/domains/communications/components/MessageBodyTab.vue:113-130`.

**Impact:** пользователь выбирает HTML-письмо и видит пустой rendered body до ручного переключения в `Original HTML`; кнопка `Rendered` не восстанавливает rendered content. Это ломает основную Communications surface.

**Fix:** render HTML shadow content on mount and whenever `message/bodyHtml/showOriginalFrame` changes, or remove dead shadow branch and use a single safe renderer path.

### Major: frontend validation gate заменен на сборку и один placeholder test

**Статус:** Открыто.

**Файлы:** `Makefile`, `frontend/package.json`, `frontend/src/__tests__/placeholder.test.ts`, удаленные `frontend/src/lib/**/*.test.ts`

`frontend-check` теперь запускает только `pnpm build`; `package.json` не содержит `check`, `lint:styles` or test-inclusive validation script. Старые API/service/store/layout tests удалены, а новая test suite состоит из одного placeholder test.

**Evidence:** `Makefile:282-292`, `frontend/package.json:7-14`, `frontend/src/__tests__/placeholder.test.ts:1-7`, `git diff --name-status -- frontend/src/lib` показывает удаление старых `.test.ts` files.

**Impact:** migration может пройти `make validate`/`frontend-check` без покрытия bootstrap API initialization, auth config, query invalidation, mail sanitizer, layout state or settings persistence. Это уже проявилось: `pnpm build` проходит при неинициализированном `ApiClient`.

**Fix:** восстановить frontend check как `lint + unit tests + build`; заменить placeholder на regression tests для bootstrap config, `ApiClient`, communications sanitizer/rendering, settings/sidebar persistence and critical query/mutation flows.

### Major: views обходят TanStack Query boundary из ADR-0093

**Статус:** Открыто.

**Файлы:** `frontend/src/domains/calendar/views/CalendarPage.vue`, `frontend/src/domains/tasks/views/TasksPage.vue`, `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`

ADR-0093 требует, чтобы server-derived state шел через TanStack Query, а direct API calls inside components были запрещены. Сейчас `CalendarPage.vue` напрямую вызывает `fetchCalendarSources`, `fetchWeeklyBrief`, `searchCalendarEvents`, `fetchEventContextPack`, `fetchEventBrief`, `fetchEventAgenda`, `createCalendarEvent`; `TasksPage.vue` напрямую вызывает decisions/obligations/task review APIs from the view.

**Evidence:** `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md:136-143`, `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md:350-353`, `frontend/src/domains/calendar/views/CalendarPage.vue:13-21`, `frontend/src/domains/calendar/views/CalendarPage.vue:56-127`, `frontend/src/domains/tasks/views/TasksPage.vue:6-7`, `frontend/src/domains/tasks/views/TasksPage.vue:27-95`.

**Impact:** server state is split between Query cache, local refs and Pinia. Invalidation becomes manual and easy to miss; loading/error state differs per screen; background refresh/stale-while-revalidate semantics promised by ADR-0093 do not hold.

**Fix:** move these calls into domain query/mutation composables with stable query keys and mutation invalidation. Views should consume composables and keep only transient UI state locally/Pinia.

### Warning: i18n contract drift from ADR-0077

**Статус:** Открыто.

**Файлы:** `frontend/src/platform/i18n/index.ts`, `frontend/src/platform/i18n/en.json`, `docs/adr/ADR-0077-i18n-russian-english.md`

ADR-0077 defines English strings as keys, `en.json` as empty identity fallback, and default locale `en`; the migrated implementation defaults to `ru` and ships a populated `en.json` identity dictionary.

**Evidence:** `docs/adr/ADR-0077-i18n-russian-english.md:16-23`, `frontend/src/platform/i18n/index.ts:10-18`, `wc -l frontend/src/platform/i18n/en.json frontend/src/platform/i18n/ru.json` reports 863 and 877 lines.

**Impact:** not a runtime blocker, but it changes the translation maintenance model without superseding ADR-0077. Future string additions now require two files instead of one English key plus Russian dictionary, and default locale behavior no longer matches the accepted ADR.

**Fix:** either restore ADR-0077 semantics in the Vue implementation or write a superseding ADR for the new default locale and dictionary model.

### Validation notes

- Ran: `pnpm lint:ts` from `frontend/`; result: passed (`vue-tsc --noEmit`).
- Ran: `pnpm test:unit` from `frontend/`; result: passed, but only 1 placeholder test executed.
- Ran: `pnpm build` from `frontend/`; result: passed. Build emitted Rolldown `INVALID_ANNOTATION` warnings from `@vueuse/core` and a chunk-size warning (`index-*.js` 641.10 kB / 186.44 kB gzip).

### Review note

Vue 3 migration itself is consistent with ADR-0093 as a platform decision. The blocking issues are not the framework choice; they are bootstrap/auth wiring, unsafe mail rendering, validation coverage loss and data-flow drift from the new ADR contract.

---

## Дополнение: ревью текущего AI Control Center diff

**Объём:** backend AI Control Center readiness guard, provider split, Testcontainers testkit retry, frontend AI Settings selector.
**ADR:** ADR-0082, ADR-0076.

### Major: API key можно сохранить для не-API provider

**Статус:** Исправлено в текущем diff. `api_key` теперь отклоняется для не-API providers на create/update boundary, а `bind_api_key_secret` дополнительно проверяет `provider_kind = 'api'` перед записью binding.

**Файлы:** `backend/src/ai/api/control_center.rs`, `backend/src/ai/control_center/providers/secrets.rs`, `backend/src/ai/control_center/vault.rs`

`post_ai_provider` и `patch_ai_provider` принимают `api_key` независимо от `provider_kind` и после create/update вызывают `store_api_key_in_host_vault(...)`. `bind_api_key_secret` проверяет только то, что `secret_ref` указывает на `host_vault` + `api_token`, но не проверяет, что целевой provider существует как `provider_kind = 'api'`.

**Impact:** `built_in` или `cli` provider может получить host-vault API-key secret и запись в `ai_provider_secret_refs`. Это нарушает ADR-0082 provider model, загрязняет host-vault metadata и создает misleading setup/status state.

**Fix:** reject `api_key` для `provider_kind != 'api'` в request/API boundary и продублировать invariant в `bind_api_key_secret` через загрузку provider перед записью binding.

### Warning: create-with-api-key оставляет частично созданный provider при сбое host vault

**Статус:** Исправлено в текущем diff. Create/patch с `api_key` теперь выполняют preflight host-vault unlock check до provider mutation; regression test проверяет, что locked/uninitialized vault возвращает `host_vault_error` и не оставляет provider row.

**Файлы:** `backend/src/ai/api/control_center.rs`, `backend/src/ai/control_center/vault.rs`

Provider создается до записи секрета в host vault. Затем `store_api_key_in_host_vault` сначала upsert-ит `secret_references`, потом пишет payload в host vault, потом bind-ит `ai_provider_secret_refs`. Если host vault write или binding падает, API request возвращает ошибку, но provider row уже остается; при повторном create с тем же `provider_kind/provider_key` возможен duplicate/unique conflict вместо идемпотентного восстановления setup flow.

**Impact:** UI может считать создание неуспешным, но backend уже содержит `needs_setup` provider и, в части failure modes, orphan `secret_references` metadata. Это операционный edge cas
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src-tauri/binaries/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/binaries/README.md`
- Size bytes / Размер в байтах: `339`
- Included characters / Включено символов: `339`
- Truncated / Обрезано: `no`

````markdown
# Tauri Sidecars

This directory holds generated Tauri sidecar binaries.

macOS release builds expect:

- `makosh-backend-aarch64-apple-darwin`
- `makosh-backend-x86_64-apple-darwin`

Generate the current host sidecar with:

```sh
make backend-sidecar-macos
```

Generated binaries are local build artifacts and are not committed.
````

### `frontend/src-tauri/resources/google-oauth/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/resources/google-oauth/README.md`
- Size bytes / Размер в байтах: `709`
- Included characters / Включено символов: `709`
- Truncated / Обрезано: `no`

````markdown
# Google OAuth Desktop Client Resource

This directory is packaged into the Tauri bundle as `$RESOURCES/google-oauth/`.

Release builds must place the Google OAuth Desktop app JSON at:

- `client_secret.json`

Use:

```sh
make google-oauth-resource
make frontend-tauri-build
```

`make google-oauth-resource` copies the JSON from
`MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` in `docker/.env`, or from
`MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` when set in the shell.

The generated `client_secret.json` is ignored by Git. It is intentionally a
bundle artifact, not a source-controlled credential file. Packaged builds pass
the bundled file path to the backend sidecar as
`MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.
````

### `frontend/src-tauri/resources/tdlib/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/resources/tdlib/README.md`
- Size bytes / Размер в байтах: `769`
- Included characters / Включено символов: `769`
- Truncated / Обрезано: `no`

```markdown
# TDLib Runtime Resources

This directory is packaged into the Tauri bundle as `$RESOURCES/tdlib/`.

Release builds for macOS must place `libtdjson.dylib` in one of these generated
directories before `tauri build`:

- `macos-arm64/libtdjson.dylib`
- `macos-x64/libtdjson.dylib`
- `macos-universal/libtdjson.dylib`

Use `make tdlib-macos-resource` to populate the matching directory from
`MAKOSH_TDJSON_SOURCE`, `MAKOSH_TDJSON_PATH`, or an installed Homebrew `tdlib`.
For release CI, `MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make tdlib-macos-resource`
can build TDLib from source before copying the generated dynamic library.
Generated dynamic libraries are ignored by Git.

Linux is development-container-only for this project and is not packaged as a
desktop TDLib resource.
```
