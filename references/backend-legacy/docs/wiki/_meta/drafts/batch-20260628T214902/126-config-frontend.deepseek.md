### Summary / Резюме

Добавить страницу `operations/configuration.md` в русскую Obsidian wiki проекта Макошь. Страница документирует конфигурацию фронтенда на основе встроенных исходных файлов: скрипты сборки, TypeScript-настройки, обработку .env, конфигурацию Tauri-десктопного приложения, систему разрешений (capabilities/permissions) и интернационализацию. Информация извлекается только из предоставленного контекста, без внешних дополнений.

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

```markdown
# Конфигурация (Configuration)

Документ описывает конфигурацию фронтенда проекта Макошь: инструменты сборки и разработки, Tauri-десктопное приложение, систему разрешений, обработку переменных окружения и интернационализацию. Все факты взяты непосредственно из файлов исходного кода.

## Конфигурация сборки и инструментов разработки

### `package.json`

- Тип модуля: `"type": "module"`
- Менеджер пакетов: `pnpm@11.5.1` (зафиксирован в поле `packageManager`)
- Основные скрипты:
  - `dev` – запуск Vite dev-сервера: `vite`
  - `build` – продакшн-сборка с предварительной проверкой типов: `pnpm typecheck && vite build`
  - `typecheck` – проверка типов без эмиссии: `vue-tsc --noEmit`
  - `lint:ox` – линтинг через oxlint: `oxlint --vue-plugin .`
  - `lint:styles` – проверка отсутствия инлайн-стилей: `node scripts/check-no-inline-styles.mjs`
  - `lint:srp` – проверка длины компонентов: `node scripts/check-component-lines.mjs`
  - `lint` – агрегированный линтинг: `pnpm lint:ox && pnpm lint:styles && pnpm lint:srp`
  - `preview` – предпросмотр собранного приложения: `vite preview`
  - `test` / `test:unit` – юнит-тесты через vitest: `vitest run`
  - `validate` – полная проверка (lint + typecheck + test + build): `pnpm lint && pnpm typecheck && pnpm test:unit && pnpm build`
  - `proto:generate` – генерация кода из protobuf: `node scripts/generate-proto.mjs`
- Рантайм-зависимости: Vue 3.5, Pinia 3, vue-router 4, Tauri API 2, @tanstack/vue-query, @tanstack/vue-table, @tanstack/vue-virtual, @connectrpc/connect, @connectrpc/connect-web, @iconify/vue, @vee-validate/zod, @vue-flow/core, @vueuse/core, date-fns, motion-v, reka-ui, vee-validate, zod.
- Dev-зависимости: Vite 8, TypeScript 6, vue-tsc 2, oxlint 1, Tailwind CSS 3, PostCSS, autoprefixer, @tauri-apps/cli 2, vitest 4, @bufbuild/protoc-gen-es.

### `tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "ignoreDeprecations": "6.0",
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    },
    "types": ["node"]
  },
  "include": ["src/**/*.ts", "src/**/*.vue", "src/**/*.d.ts"],
  "exclude": ["node_modules", "dist", "src-svelte"]
}
```

- Алиас `@` указывает на `src/`.
- Из проверки исключены `node_modules`, `dist` и `src-svelte`.

### `pnpm-workspace.yaml`

```yaml
allowBuilds:
  vue-demi: true
```

Разрешена сборка пакета `vue-demi`.

## Переменные окружения (`frontend/.gitignore`)

Игнорируются все файлы `.env` и `.env.*`, кроме:

- `.env.example`
- `.env.test`

Также игнорируются временные файлы Vite:

- `vite.config.js.timestamp-*`
- `vite.config.ts.timestamp-*`

Остальные игнорируемые элементы: `node_modules`, `/dist`, `.DS_Store`, `Thumbs.db`.

## Tauri-десктопное приложение

### `tauri.conf.json`

```json
{
  "productName": "Макошь",
  "version": "0.1.0",
  "identifier": "dev.makosh.desktop",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://127.0.0.1:5173"
  },
  "app": {
    "windows": [
      {
        "title": "Макошь",
        "width": 1280,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "externalBin": ["binaries/makosh-backend"],
    "resources": {
      "resources/google-oauth/": "google-oauth",
      "resources/tdlib/": "tdlib"
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- Content Security Policy (`csp`) установлен в `null` — ограничения CSP не применяются.
- Внешний исполняемый файл: `binaries/makosh-backend`.
- Ресурсы: папки `google-oauth` и `tdlib`.
- Иконки для разных платформ.

### `Cargo.toml` (src-tauri)

```toml
[package]
name = "app"
version = "0.1.0"
description = "Макошь desktop shell"
authors = ["Макошь"]
edition = "2024"
rust-version = "1.89"

[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.6.2", features = [] }

[dependencies]
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
tauri = { version = "2.11.2", features = [] }
tauri-plugin-log = "2"
tauri-plugin-shell = "2"
ureq = { version = "3.3.0", default-features = false, features = ["json"] }
```

- Минимальная версия Rust: 1.89.
- Библиотека собирается как staticlib, cdylib, rlib.

### `.gitignore` (src-tauri)

```gitignore
/target/
/gen/schemas
```

Игнорируются артефакты сборки Rust и сгенерированные схемы.

## Система разрешений (Capabilities и Permissions)

Файлы в `src-tauri/capabilities/` определяют наборы разрешений для окон приложения.

### `capabilities/default.json` – главное окно `main`

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allow-open-whatsapp-web-companion",
    "allow-whatsapp-web-companion-manifest",
    "allow-yandex-telemost-companion",
    "allow-yandex-telemost-companion-manifest",
    "allow-yandex-telemost-prepare-audio-device",
    "allow-yandex-telemost-recording-start",
    "allow-yandex-telemost-recording-stop"
  ]
}
```

### `capabilities/whatsapp-companion-relay.json` – окна-компаньоны WhatsApp Web

```json
{
  "identifier": "whatsapp-companion-relay",
  "local": false,
  "remote": {
    "urls": ["https://web.whatsapp.com"]
  },
  "windows": ["whatsapp-companion-*"],
  "permissions": [
    "allow-whatsapp-web-companion-relay-observation"
  ]
}
```

- Права действуют только для удалённого контента с `https://web.whatsapp.com`.
- Окна с префиксом `whatsapp-companion-*`.

### `capabilities/yandex-telemost-companion-relay.json` – окна Яндекс.Телемост

```json
{
  "identifier": "yandex-telemost-companion-relay",
  "local": false,
  "remote": {
    "urls": [
      "https://telemost.yandex.ru",
      "https://telemost.yandex.com"
    ]
  },
  "windows": ["yandex-telemost-*"],
  "permissions": [
    "allow-yandex-telemost-speaker-timeline-append"
  ]
}
```

- Права действуют для доменов Яндекса.
- Окна с префиксом `yandex-telemost-*`.

### Автогенерированные файлы разрешений (`permissions/autogenerated/`)

Все файлы содержат пометку `# Automatically generated - DO NOT EDIT!`. Каждый файл определяет правило `allow-*` и `deny-*` для одной команды.

Список команд:

- `open_whatsapp_web_companion`
- `whatsapp_web_companion_manifest`
- `whatsapp_web_companion_relay_observation`
- `open_yandex_telemost_companion`
- `yandex_telemost_companion_manifest`
- `yandex_telemost_prepare_audio_device`
- `yandex_telemost_recording_start`
- `yandex_telemost_recording_stop`
- `yandex_telemost_speaker_timeline_append`

Пример одного из файлов (`open_whatsapp_web_companion.toml`):

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-open-whatsapp-web-companion"
description = "Enables the open_whatsapp_web_companion command without any pre-configured scope."
commands.allow = ["open_whatsapp_web_companion"]

[[permission]]
identifier = "deny-open-whatsapp-web-companion"
description = "Denies the open_whatsapp_web_companion command without any pre-configured scope."
commands.deny = ["open_whatsapp_web_companion"]
```

## Интернационализация (i18n)

Файлы переводов расположены в `frontend/src/platform/i18n/`:

- `en.json` – английский
- `ru.json` – русский

Страница настроек приложения использует следующие ключи (приведены видимые в контексте строки; файлы обрезаны):

| Ключ (en.json) | Перевод (ru.json) |
|---|---|
| `Application Settings` | Настройки приложения |
| `Settings Metrics` | Метрики настроек |
| `Loading settings...` | Загрузка настроек... |
| `No application settings are declared yet.` | Настройки приложения ещё не объявлены. |
| `Enabled` | Включено |
| `Disabled` | Отключено |
| `Runtime Source` | Источник runtime |
| `Backend bind` | Backend bind |
| `Frontend API` | Frontend API |
| `AI configuration` | AI-конфигурация |
| `Model routing` | Маршрутизация моделей |
| `Capability slots` | Capability slots |
| `Boundaries` | Границы |
| `Bootstrap` | Bootstrap |
| `Hidden` | Скрыто |
| `Secret boundary` | Граница секретов |
| `All non-secret settings except database connectivity; secret-like keys are rejected.` | Все несекретные настройки, кроме подключения к базе; ключи, похожие на секреты, отклоняются. |
| `PostgreSQL stores declared setting values` | PostgreSQL хранит значения объявленных настроек |
| `AI providers, models and routes live in AI Control Center` | AI-провайдеры, модели и маршруты живут в AI Control Center |
| `Domain tables` | Доменные таблицы |
| `Legacy ai.* settings are bootstrap fallback only` | Legacy ai.* настройки только bootstrap fallback |
| `Database URL stays outside the panel` | Database URL остаётся вне панели |
| `API token and vault key stay outside DB` | API token и vault key остаются вне БД |
| `Credentials stay in encrypted vault` | Учётные данные остаются в encrypted vault |
| `No secret values` | Без секретных значений |
| `Settings updates are audited` | Изменения настроек аудируются |
| `No values in audit` | Без значений в audit |
| `Accounts List` | Список аккаунтов |
| `Account Setup` | Настройка аккаунта |
| `Account Detail Status` | Статус аккаунта |
| `Security And Runtime Status` | Безопасность и состояние |

Из этих строк видно, что интерфейс настроек:

- Группирует настройки по разделам (Runtime Source, Backend bind, Frontend API, AI configuration, Model routing, Capability slots, Boundaries).
- Хранит значения в PostgreSQL.
- Не отображает секретные значения и ключи, похожие на секреты.
- Аудит изменений ведётся без записи самих значений.
- AI-конфигурация вынесена в отдельный центр управления (AI Control Center).
- Legacy-ключи `ai.*` используются только как bootstrap fallback.
- Database URL, API token и vault key остаются вне панели настроек.
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые в странице |
|---|---|
| `frontend/.gitignore` | Игнорируемые файлы: node_modules, dist, .DS_Store, Thumbs.db, .env/.env.* с исключениями .env.example и .env.test, временные метки Vite. |
| `frontend/package.json` | Тип модуля, менеджер пакетов, все скрипты, список runtime и dev зависимостей. |
| `frontend/pnpm-workspace.yaml` | Разрешение сборки vue-demi: `allowBuilds: vue-demi: true`. |
| `frontend/tsconfig.json` | Все compilerOptions, include, exclude, paths alias `@/*` -> `src/*`. |
| `frontend/src-tauri/.gitignore` | Игнорирование /target и /gen/schemas. |
| `frontend/src-tauri/tauri.conf.json` | Все поля: productName, version, identifier, build, app/windows, security/csp, bundle (externalBin, resources, icons). |
| `frontend/src-tauri/Cargo.toml` | Пакет app, версия, описание, Rust edition, rust-version, crate-type, зависимости (tauri, serde, log, плагины, ureq). |
| `frontend/src-tauri/capabilities/default.json` | Идентификатор default, окно main, список разрешений (core:default, 8 allow-*). |
| `frontend/src-tauri/capabilities/whatsapp-companion-relay.json` | Ограничение по URL `https://web.whatsapp.com`, окна `whatsapp-companion-*`, разрешение relay-observation. |
| `frontend/src-tauri/capabilities/yandex-telemost-companion-relay.json` | Ограничение по URL Яндекса, окна `yandex-telemost-*`, разрешение speaker-timeline-append. |
| `frontend/src-tauri/permissions/autogenerated/open_whatsapp_web_companion.toml` | Идентификаторы allow-/deny-open-whatsapp-web-companion, команда `open_whatsapp_web_companion`. |
| …`whatsapp_web_companion_manifest.toml` | Команда `whatsapp_web_companion_manifest`. |
| …`whatsapp_web_companion_relay_observation.toml` | Команда `whatsapp_web_companion_relay_observation`. |
| …`open_yandex_telemost_companion.toml` | Команда `open_yandex_telemost_companion`. |
| …`yandex_telemost_companion_manifest.toml` | Команда `yandex_telemost_companion_manifest`. |
| …`yandex_telemost_prepare_audio_device.toml` | Команда `yandex_telemost_prepare_audio_device`. |
| …`yandex_telemost_recording_start.toml` | Команда `yandex_telemost_recording_start`. |
| …`yandex_telemost_recording_stop.toml` | Команда `yandex_telemost_recording_stop`. |
| …`yandex_telemost_speaker_timeline_append.toml` | Команда `yandex_telemost_speaker_timeline_append`. |
| `frontend/src/platform/i18n/en.json` (truncated) | Ключи Application Settings, Settings Metrics, Loading settings..., No application settings..., Enabled, Disabled, Runtime Source, Backend bind, Frontend API, AI configuration, Model routing, Capability slots, Boundaries, Bootstrap, Hidden, Secret boundary, строки о PostgreSQL, AI Control Center, аудите, секретах, аккаунтах. |
| `frontend/src/platform/i18n/ru.json` (truncated) | Переводы всех вышеуказанных ключей на русский язык. |
| `frontend/pnpm-lock.yaml` (truncated) | Не покрыт напрямую — lock-файл не содержит декларативных конфигурационных фактов, кроме подтверждения версий зависимостей (что уже отражено в package.json). |
| `frontend/src-tauri/Cargo.lock` (truncated) | Не покрыт — аналогично, lock-файл Rust, не документируется как конфигурация. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения между кодом и документацией не видны. Сравниваемых артефактов (например, существующей wiki-страницы или ADR) в составе чанка нет, поэтому подтвердить или опровергнуть drift невозможно. Единственный потенциальный момент — `"csp": null` в `tauri.conf.json` может быть осознанным решением или упущением, но без дополнительных источников это не классифицируется как drift.
