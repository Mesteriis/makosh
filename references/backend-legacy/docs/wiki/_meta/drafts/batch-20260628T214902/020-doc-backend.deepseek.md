## Summary

Страница `operations/documentation-map.md` должна быть обновлена на основе
содержимого `backend/README.md`. Она предоставит структурированную карту
документации backend-компонента Макошь: обзор возможностей, перечень
команд разработки и тестирования, переменные окружения, HTTP API (health,
readiness, V1‑статус, хранилище, события, аудит, граф, интеграции почты),
защищённые Workflow API, описание дымовых тестов (smoke tests), операции с
email‑фикстурами, графовую проекцию, AI‑подсистему и перечень текущих
ограничений.

## Proposed pages

### `operations/documentation-map.md`

````markdown
# Карта документации backend

Карта построена на основе `backend/README.md` и описывает документацию,
доступную для backend‑компонента Макошь.

## Обзор

Backend реализован на Rust. Текущий состав включает:

- конфигурационный парсинг;
- Health‑ и Readiness‑эндпоинты;
- V1‑статусный API;
- каноническое событийное API (append/read) и хранение событий;
- аудит доступа к API;
- onboarding и unlock host‑vault, совместимость с устаревшим database‑vault;
- настройка учётных записей Gmail/iCloud/IMAP, метаданные секретных ссылок;
- ingestion‑хранилище коммуникаций, префлайт‑планирование синхронизации почты;
- сетевое взаимодействие с почтовыми провайдерами с явными границами
  read/write‑возможностей;
- импорт/экспорт email‑фикстур;
- локальное хранение почтовых блобов и метаданных вложений;
- границы проекции сообщений, Persona‑совместимых идентичностей и документов;
- граница полнотекстового поиска Tantivy;
- курсоры проекций и пакетная семантика projection‑runner;
- графовое ядро: проекция и API чтения;
- защищённые Workflow API для проектов, кандидатов задач, обзора
  Persona‑идентичностей и обработки документов;
- локальные AI‑workflow API на базе Ollama + семантический поиск через
  pgvector.

В данный момент **не реализованы**:

- полный парсинг MIME и извлечение вложений;
- редактирование графа и более глубокая графовая инференция;
- first‑class Polygraph‑наблюдения;
- автономный agent‑action runtime.

## Команды

### Make‑цели

Из корня репозитория доступны (выборочно):

- `make backend-run`, `make backend-run-dev` – запуск backend;
- `make backend-watch-dev` – автоперезапуск backend при изменениях;
- `make dev` – полный цикл разработки: поднимает PostgreSQL, запускает backend
  с автоперезапуском (требуется `watchexec` или `cargo-watch`) и фронтенд
  SvelteKit с Vite HMR;
- `make backend-smoke-dev`, `make backend-storage-smoke-dev`,
  `make backend-secrets-smoke-dev`, `make backend-event-log-smoke-dev`,
  `make backend-communication-smoke-dev`, … – дымовые тесты;
- `make backend-email-*` – работа с email‑фикстурами и кешем синхронизации;
- `make backend-graph-smoke-dev` – тесты графового хранилища, проекций и API
  чтения (запускает и останавливает Compose‑PostgreSQL);
- `make backend-workflow-smoke-dev` – интеграционные тесты Workflow API
  (проекты, кандидаты задач, Persona identity, документы); включена в
  `make validate`;
- `make backend-ai-smoke-dev` – тесты AI‑подсистемы с живым Ollama;
- `make backend-graph-project-dev` – проекция текущих V1‑данных в графовые
  таблицы (не подключается к почтовым провайдерам);
- `make backend-validate` – линтинг/проверки.

`backend-contacts-smoke-dev` – устаревшее имя; в данный момент запускает
набор интеграционных тестов `persons` и сохранено только для совместимости.

### Прямые Cargo‑команды

```sh
cargo run --manifest-path backend/Cargo.toml
cargo run --manifest-path backend/Cargo.toml --bin makosh-graph-project
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-fixture-export
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-fixture-dev
cargo run --manifest-path backend/Cargo.toml --bin makosh-email-sync-dev
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

## Переменные окружения

- `MAKOSH_HTTP_ADDR` – адрес прослушивания, по умолчанию `127.0.0.1:8080`.
- `MAKOSH_BACKEND_STARTUP_ATTEMPTS` / `MAKOSH_BACKEND_STARTUP_SLEEP_SECONDS` –
  управление опросом готовности backend для `make dev` (по умолчанию
  300 попыток, интервал 1 с).
- `MAKOSH_FRONTEND_STARTUP_ATTEMPTS` / `MAKOSH_FRONTEND_STARTUP_SLEEP_SECONDS` –
  аналогично для фронтенда (по умолчанию 120 попыток, интервал 1 с).
- `DATABASE_URL` – опциональный URL PostgreSQL. Эндпоинт `/healthz` не
  требует соединения с БД.
- `MAKOSH_LOCAL_API_SECRET` – локальный общий секрет, требуемый охранной
  защитой маршрутизатора для защищённых локальных API.
- `MAKOSH_VAULT_HOME` – директория host‑vault; по умолчанию локальный
  Макошь‑vault‑home.
- `MAKOSH_DEV_MODE` – при значении `true` включает отладочное поведение
  ключа разработки host‑vault.
- `MAKOSH_DEV_KEY_PATH` – путь к отладочному ключу разработки host‑vault.
- `MAKOSH_SECRET_VAULT_KEY` – устаревший мастер‑ключ зашифрованного
  database‑vault, **запрещено коммитить, логировать или сохранять в
  PostgreSQL**.
- `MAKOSH_OLLAMA_BASE_URL` – URL Ollama, по умолчанию `http://127.0.0.1:11434`.
- `MAKOSH_OLLAMA_CHAT_MODEL` – модель чата, по умолчанию `qwen3:4b`.
- `MAKOSH_OLLAMA_EMBED_MODEL` – модель эмбеддингов, по умолчанию
  `qwen3-embedding:4b`.
- `MAKOSH_OLLAMA_TIMEOUT_SECONDS` – таймаут запроса к Ollama, по умолчанию
  `120`.

## HTTP API

Все эндпоинты `/api/v1/*` требуют заголовок
`X-Макошь-Secret: <MAKOSH_LOCAL_API_SECRET>`, если не указано иное.

### Health и Readiness

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/healthz` | Возвращает статус health и имя сервиса. |
| `GET` | `/readyz` | Статус готовности; `503`, если PostgreSQL не настроен, недоступен или отсутствуют миграции SQLx. |

### V1‑статус и Vault

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/v1/status` | Включённые V1‑поверхности. |
| `GET` | `/api/v1/vault/status` | Статус инициализации/разблокировки host‑vault. |
| `POST` | `/api/v1/vault/collect-entropy` | Запись образцов энтропии для создания host‑vault. |
| `POST` | `/api/v1/vault/create` | Создание и разблокировка host‑vault. |
| `POST` | `/api/v1/vault/unlock` | Разблокировка существующего host‑vault. |
| `POST` | `/api/v1/vault/recovery/export` | Экспорт материалов восстановления для разблокированного host‑vault. |
| `POST` | `/api/v1/vault/recovery/import` | Импорт материалов восстановления. |

### События и Аудит

| Метод | Путь | Описание |
|-------|------|----------|
| `POST` | `/api/v1/events` | Добавление канонического события. Авторизованные вызовы записываются в `api_audit_log` с актором `makosh-frontend`. Значение API‑секрета никогда не сохраняется. |
| `GET` | `/api/v1/events/{event_id}` | Загрузка события по ID. |
| `GET` | `/api/v1/audit/events` | Записи аудита, параметры: `target_id`, `actor_id`, `after_audit_id`, `limit`. |

### Граф

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/v1/graph/summary` | Сводка количества узлов, рёбер и evidence. |
| `GET` | `/api/v1/graph/search` | Поиск узлов графа по параметру `q`, опционально `limit`. |
| `GET` | `/api/v1/graph/neighborhood` | Окрестность глубины 1 для `node_id`, включая соседние узлы, рёбра и evidence. |

### Интеграции почты

| Метод | Путь | Описание |
|-------|------|----------|
| `POST` | `/api/v1/integrations/mail/accounts/gmail/oauth/start` | Запуск OAuth‑настройки Gmail, возвращает PKCE‑URL авторизации. Требует локальные API‑заголовки, PostgreSQL и инициализированный/разблокированный host‑vault. |
| `GET` | `/api/v1/integrations/mail/accounts/gmail/oauth/callback` | Отображение OAuth‑кода/состояния для desktop‑setup. |
| `POST` | `/api/v1/integrations/mail/accounts/gmail/oauth/complete` | Обмен Gmail‑кода, сохранение учётных данных в host‑vault, создание привязки учётной записи провайдера. |
| `POST` | `/api/v1/integrations/mail/accounts/imap` | Создание метаданных учётной записи iCloud/IMAP, сохранение пароля/app‑password в host‑vault. |

### Workflow API (защищённые)

Все требуют `X-Макошь-Secret`.

| Метод | Путь | Описание |
|-------|------|----------|
| `GET` | `/api/v1/projects` | Список проектов с производными статистиками. |
| `GET` | `/api/v1/projects/{project_id}` | Детали проекта: хронология, сообщения, документы, люди. |
| `GET` | `/api/v1/projects/{project_id}/link-candidates` | Кандидаты на связывание сообщений/документов проекта. |
| `PUT` | `/api/v1/projects/{project_id}/link-reviews` | Запись состояния обзора связей как каноническое событие. |
| `GET` | `/api/v1/task-candidates` | Список кандидатов задач с источниками. |
| `PUT` | `/api/v1/task-candidates/{task_candidate_id}/review` | Запись обзора кандидата задачи (далее исходный `README.md` обрезан, полный перечень эндпоинтов не подтверждён текущим контекстом). |

## Дымовые тесты (smoke tests)

- `make backend-smoke-dev`, `make backend-storage-smoke-dev`,
  `make backend-secrets-smoke-dev`, `make backend-event-log-smoke-dev`,
  `make backend-communication-smoke-dev` – общие и по‑компонентные тесты.
- `make backend-graph-smoke-dev` – стартует локальный PostgreSQL, выполняет
  тесты графового хранилища, проекций и API чтения, затем останавливает
  Compose‑сервис. Не запускать, если используется та же Compose‑PostgreSQL
  для активной сессии разработки.
- `make backend-workflow-smoke-dev` – стартует локальный PostgreSQL, создаёт
  изолированные временные БД, последовательно прогоняет интеграционные
  наборы проектов, API проектов, обзора связей, кандидатов задач, Persona
  identity и обработки документов. Включена в `make validate`.
- `make backend-ai-smoke-dev` – стартует локальный PostgreSQL для
  pgvector/API‑тестов и выполняет живую проверку Ollama против
  `http://192.168.1.2:11434` по умолчанию. Переопределить эндпоинт можно
  переменной `MAKOSH_AI_SMOKE_OLLAMA_BASE_URL`.
- `backend-contacts-smoke-dev` – устаревшее имя, запускает тестовый набор
  `persons`.

## Работа с email‑фикстурами

- `make backend-email-fixture-export-icloud-dev` – экспорт redacted‑фикстуры
  из iCloud IMAP. Требует переменные `MAKOSH_IMAP_FIXTURE_USERNAME`,
  `MAKOSH_IMAP_FIXTURE_PASSWORD`, `MAKOSH_IMAP_FIXTURE_MAX_MESSAGES`,
  `MAKOSH_IMAP_FIXTURE_OUTPUT`. Использует IMAP‑команды `EXAMINE`,
  `UID SEARCH` и `BODY.PEEK[]`, не импортирует в PostgreSQL. Результат по
  умолчанию в `tmp/` (игнорируется Git).
- `make backend-email-fixture-import-dev` – импорт redacted‑фикстуры в
  локальную dev‑БД.
- `make backend-email-fixture-project-dev` – проекция импортированной
  фикстуры через канонические сообщения, Persona‑совместимые записи и
  графовую проекцию.
- `make backend-email-sync-cache-dev` – синхронизация почты iCloud/IMAP в
  локальный кеш (чтение‑только, сохранение сырых `.eml` в `docker/data/mail/`,
  метаданные в PostgreSQL). Gmail OAuth пока не поддерживается; для Gmail
  нужно предварительно получить токены через account‑setup.

## Графовая проекция

- `make backend-graph-project-dev` – стартует локальный PostgreSQL (если
  нужно), применяет миграции, запускает
  `GraphProjectionService::project_from_v1()` для текущей dev‑БД и выводит
  JSON‑сводку проекции. Не подключается к почтовым провайдерам.

## AI‑подсистема

AI Workflow API работают на базе Ollama и pgvector для семантического
поиска. Параметры задаются переменными окружения:

- `MAKOSH_OLLAMA_BASE_URL`, `MAKOSH_OLLAMA_CHAT_MODEL`,
  `MAKOSH_OLLAMA_EMBED_MODEL`, `MAKOSH_OLLAMA_TIMEOUT_SECONDS`.

Дымовой тест: `make backend-ai-smoke-dev`.

## Текущие ограничения (не реализовано)

- Полный MIME‑парсинг, извлечение вложений.
- Редактирование графа и более глубокая графовая инференция.
- First‑class Polygraph‑наблюдения.
- Автономный agent‑action runtime.
````

## Source coverage

Исходный файл: `backend/README.md` (обрезан на 12000 символов).

Факты, покрытые предложенной страницей:

- общий состав backend и список **не** реализованных возможностей;
- Make‑команды для запуска, разработки, тестирования, работы с
  email‑фикстурами, графовой проекцией и AI;
- прямые Cargo‑команды;
- переменные окружения с типами, значениями по умолчанию и
  комментариями безопасности (`MAKOSH_SECRET_VAULT_KEY`);
- полные пути и методы HTTP‑эндпоинтов (`/healthz`, `/readyz`,
  `/api/v1/*`, в том числе vault, events, audit, graph, интеграции
  почты и Workflow API), требования к заголовку `X-Макошь-Secret`;
- границы дымовых тестов (граф, workflow, AI, устаревший
  `contacts-smoke`) и особенности их запуска;
- операции экспорта/импорта email‑фикстур и синхронизации почтового
  кеша с указанием необходимых переменных окружения;
- описание графовой проекции через `make backend-graph-project-dev`;
- параметры AI‑подсистемы и smoke‑test;
- ограничение: обрезка исходника не позволяет подтвердить полный
  перечень Workflow API, что явно отмечено.

## Drift candidates

Из предоставленного контекста (только `backend/README.md`, без
сопоставления с исходным кодом, другими wiki‑страницами или ADR)
расхождения кода/документации не видны.
