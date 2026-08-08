### Summary / Резюме

Создаётся страница `operations/documentation-map.md` — карта документации фронтенда Макошь. Она перечисляет имеющиеся документы по фронтенду, кратко описывает их содержание и указывает на известные расхождения между текущим состоянием кода и документацией/ADR, выявленные в ходе code review от 2026-06-14. Всё содержание страницы опирается исключительно на встроенные в этот context pack исходные файлы; ссылки на ADR, не включённые в пакет, документируются только как упоминания без раскрытия содержания.

### Proposed pages / Предлагаемые страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации фронтенда

Страница-оглавление по документации фронтенда Макошь. Описывает доступные файлы документации и основные факты из них.

## Документы

### `frontend/README.md`

Основная документация фронтенда.

- **Стек:** Vue 3 + TypeScript, сборка Tauri, Vite, Tailwind.
- **Область применения:** десктопная оболочка для локальных backend API; визарды настройки учётных записей Gmail, iCloud и IMAP; рабочие поверхности для графов, проектов, задач, идентичности Persona, документов и локальных AI-воркфлоу. Мобильный UI исключён текущим `ADR-0031`.
- **Стили:** глобальные стили подгружаются из `src/assets/styles/tokens.css` и `src/assets/styles/app.css`. Компоненты используют scoped `<style>` и классы Tailwind. Inline-стили в продакшен-компонентах запрещены. Минимальное окно — `800x600`, при меньших размерах показывается viewport guard (не мобильная поддержка).
- **Скэффолд:** проект создан командами `pnpm create vue@latest . -- --typescript --force` и `pnpm tauri init --ci --app-name "Макошь" ...`.
- **Команды разработки:**
  - `make dev` — запуск полного dev-стека (PostgreSQL в Docker, backend через `bacon`, фронтенд на `http://127.0.0.1:5174`).
  - `make logs` — агрегированный plain-text лог.
  - `make build` — продакшен-сборка: фронтенд, backend, ресурсы Google OAuth, TDLib, встраиваемый backend sidecar, затем `pnpm tauri build`.
  - `make migrate`, `make clean`, `make clean-vault`.
- **Встраиваемый TDLib runtime:** macOS-релиз пакует `libtdjson.dylib` из `frontend/src-tauri/resources/tdlib/`. Файлы не коммитятся; `make build` подготавливает ресурс, копируя из `MAKOSH_TDJSON_SOURCE`, `MAKOSH_TDJSON_PATH`, Homebrew `tdlib` или по фиксированным путям. Для CI можно собрать TDLib из исходников через `MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make build`. Linux — только контейнеры, без десктопного пакета TDLib.
- **Telegram QR-вход:** требует учётные данные приложения Telegram. Для разработки — переменные окружения `MAKOSH_TELEGRAM_API_ID` и `MAKOSH_TELEGRAM_API_HASH`. Для упакованных сборок — `MAKOSH_BUNDLED_TELEGRAM_API_ID` и `MAKOSH_BUNDLED_TELEGRAM_API_HASH`, которые лаунчер передаёт backend-sidecar как `MAKOSH_TELEGRAM_API_ID` и `MAKOSH_TELEGRAM_API_HASH`.
- **Google OAuth:** нужен один Desktop-клиент проекта. Релизная сборка копирует JSON-файл клиента в `frontend/src-tauri/resources/google-oauth/client_secret.json` (игнорируется Git). Читается из `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` (`docker/.env`) или `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` (shell). Лаунчер передаёт путь к backend-sidecar как `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.
- **Встраиваемый backend sidecar:** macOS-релиз пакует Rust-бэкенд как Tauri-sidecar из `frontend/src-tauri/binaries/`. Бинарники не коммитятся; `make build` подготавливает их перед `tauri build`.
- **Архитектура:** доменно-ориентированная структура в `src/domains/` (14 доменов: home, settings, personas, organizations, projects, tasks, calendar, documents, notes, knowledge, review, agents, timeline, communications, telegram, whatsapp). Общая структура домена: `types/`, `api/`, `queries/` (TanStack Query), `stores/` (Pinia), `components/`, `views/`. Поток данных: API → TanStack Query → компонент, или API → Pinia Store → компонент. Все запросы используют `X-Макошь-Secret` через централизованный `ApiClient` (`src/platform/api/ApiClient.ts`).
- **Валидация:** `make build`. Полный стек: `make dev`.

### `frontend/docs/code-review-2026-06-14.md`

Полное ревью фронтенда, датированное 2026-06-14. Охватывает более 80 файлов: конфигурацию, API, типы, эндопинты, сервисы, стор-файлы, страницы, тесты, i18n, стили. Также включает дополнение по текущей Vue‑3 миграции и AI Control Center diff.

**Ключевые выводы ревью:**

- Выявлены критические проблемы, обязательные к исправлению перед расширением:
  - `ApiClient` не инициализируется перед использованием в новом entrypoint.
  - Несоответствие имён переменных окружения для backend URL и секрета конфигурации из Makefile.
  - Некорректная отрисовка писем через `innerHTML` с недостаточной санацией (потенциальный XSS).
  - Пустой rendered view для HTML-писем при дефолтном отображении.
  - Валидационный гейт фронтенда сведён к сборке без тестов (один placeholder test).
  - Представления напрямую вызывают API-функции, обходя TanStack Query boundary, что нарушает `ADR-0093`.
  - Дрейф i18n контракта относительно `ADR-0077` (локаль по умолчанию и модель словарей).
- В AI Control Center diff зафиксированы и исправлены: некорректное сохранение API-ключа для non-API провайдера и частичное создание провайдера при сбое host vault.

**Примечание:** исходный файл был обрезан при встраивании в context pack, поэтому полный перечень результатов ревью недоступен.

В ревью имеются ссылки на следующие ADR (содержание которых не включено в данный контекст-пакет):

- `ADR-0093` (frontend platform migration to Vue 3)
- `ADR-0031` (mobile scope)
- `ADR-0056` (protected endpoints)
- `ADR-0077` (i18n Russian/English)
- `ADR-0082` (провайдерная модель AI)
- `ADR-0076` (упоминается в секции AI Control Center)

### `frontend/src-tauri/binaries/README.md`

- Описывает каталог генерируемых Tauri-sidecar бинарников.
- macOS-релиз ожидает файлы: `makosh-backend-aarch64-apple-darwin` и `makosh-backend-x86_64-apple-darwin`.
- Генерация выполняется командой `make backend-sidecar-macos`.
- Бинарники не коммитятся в репозиторий.

### `frontend/src-tauri/resources/google-oauth/README.md`

- Пакетируется в Tauri-бандл как `$RESOURCES/google-oauth/`.
- Для релиза в каталоге должен лежать `client_secret.json`.
- Используется команда `make google-oauth-resource`, затем сборка фронтенда.
- `make google-oauth-resource` копирует JSON из `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH` (`docker/.env`) или `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_SOURCE` (shell).
- Сгенерированный `client_secret.json` добавлен в `.gitignore`.
- Упакованный лаунчер передаёт путь к файлу в backend-sidecar как `MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH`.

### `frontend/src-tauri/resources/tdlib/README.md`

- Пакетируется в Tauri-бандл как `$RESOURCES/tdlib/`.
- Для macOS-релиза необходимо разместить `libtdjson.dylib` в одном из каталогов: `macos-arm64/`, `macos-x64/` или `macos-universal/`.
- Заполнение каталога выполняется через `make tdlib-macos-resource`, который ищет библиотеку по `MAKOSH_TDJSON_SOURCE`, `MAKOSH_TDJSON_PATH`, Homebrew `tdlib`.
- Для CI сборку TDLib из исходников можно включить флагом `MAKOSH_TDLIB_BUILD_FROM_SOURCE=1 make tdlib-macos-resource`.
- Сгенерированные динамические библиотеки не коммитятся.
- Linux — только dev-контейнеры; десктопный TDLib-ресурс не пакетируется.
```

### Source coverage / Покрытие источников

- **`frontend/README.md`** — полностью покрыт: стек, область применения, контракт стилей, минимальное окно, команды скэффолда и разработки (`make dev`, `make build`, прочие), встраиваемые ресурсы (TDLib, Telegram-учётные данные, Google OAuth, backend sidecar), архитектура (домены, структура, поток данных, `ApiClient`, `X-Макошь-Secret`), валидация.
- **`frontend/docs/code-review-2026-06-14.md`** (обрезан до 12000 символов) — покрыты: дата ревью, охват (80+ файлов, категории), критические и крупные проблемы (инициализация `ApiClient`, расхождение имён переменных окружения, некорректная отрисовка писем, пустой rendered view, утрата валидационного гейта, прямой вызов API из представлений, дрейф i18n), ссылки на ADR, данные валидационных прогонов. Часть об AI Control Center diff также зафиксирована в той мере, в какой она вошла в обрезанный фрагмент. Полный текст ревью остаётся за пределами контекст-пакета.
- **`frontend/src-tauri/binaries/README.md`** — полностью покрыт: ожидаемые имена бинарников для macOS, команда генерации, факт некоммитности.
- **`frontend/src-tauri/resources/google-oauth/README.md`** — полностью покрыт: целевой файл, команда подготовки ресурса, источники конфигурации (env), игнорирование `client_secret.json` Git-ом, способ передачи пути в backend-sidecar.
- **`frontend/src-tauri/resources/tdlib/README.md`** — полностью покрыт: структура каталогов для macOS, команда подготовки ресурса, источники библиотеки, опция сборки из исходников для CI, некоммитность, исключение Linux из десктопного пакета.

### Drift candidates / Кандидаты на drift

На основе предоставленного контекста выявлены следующие расхождения между кодом и документацией/ADR:

1. **Инициализация `ApiClient`** — `frontend/README.md` описывает использование `ApiClient` для авторизованных запросов, однако ревью от 2026-06-14 фиксирует отсутствие вызова `ApiClient.init()` в entrypoint, что приводит к падению рантайма (контракт `ApiClient` не выполняется).
2. **Имена переменных окружения для конфигурации** — `frontend/README.md` не задаёт конкретные имена, но ревью указывает, что текущий `config/index.ts` читает `VITE_API_BASE_URL` / `VITE_MAKOSH_API_SECRET` с дефолтом `http://localhost:3000`, тогда как Makefile использует `VITE_MAKOSH_API_BASE_URL` и `VITE_MAKOSH_LOCAL_API_SECRET`, а backend слушает `127.0.0.1:8080`. Наблюдается расхождение между READMED-описанными командами и фактической конфигурацией.
3. **Безопасность отрисовки писем** — README не описывает модель безопасности рендеринга, но ревью указывает на регресс: `innerHTML`-вставка недостаточно очищенного HTML (отсутствие удаление атрибутов-обработчиков, опасных URL) и неэкранированный plain text. Это несоответствие ожидаемому контракту обработки ненадёжного ввода.
4. **Валидационный гейт** — README предполагает `make build` как валидацию упаковки, но не описывает полную проверку. Ревью отмечает, что ранее существовавший `frontend-check` теперь заменён только на сборку, а тесты удалены (оставлен один placeholder). Это уменьшает покрытие по сравнению с предполагаемой зрелостью проекта.
5. **Нарушение `ADR-0093` (TanStack Query boundary)** — `frontend/README.md` описывает поток данных через TanStack Query, но ревью фиксирует прямые API-вызовы из представлений Calendar и Tasks, что прямо противоречит контракту, зафиксированному в ADR-0093 (содержание ADR не включено в пакет, но факт нарушения задокументирован ревью).
6. **Дрейф `ADR-0077` (i18n)** — README не специфицирует i18n модель, однако ревью указывает, что текущая реализация (дефолтная локаль `ru`, заполненный `en.json`) отличается от модели, описанной в ADR-0077 (дефолт `en`, `en.json` как пустой identity fallback). Без доступа к тексту ADR полный объём отклонения не подтверждён.
7. **Актуальность `frontend/README.md`** — часть зафиксированных в ревью проблем (пп. 1–3) напрямую ставит под вопрос соответствие README текущему состоянию кода; документ может требовать обновления после исправления выявленных дефектов.

Для пунктов, ссылающихся на полный текст ADR (0031, 0093, 0077, 0056, 0082, 0076), окончательный вердикт о drift не может быть вынесен без встраивания самих ADR-файлов в контекстный пакет.
