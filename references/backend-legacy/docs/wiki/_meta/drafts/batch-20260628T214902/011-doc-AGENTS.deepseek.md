### Summary

Страница `operations/documentation-map.md` должна предоставлять навигационную карту документации проекта Макошь на основе правил из `AGENTS.md`. В текущем контексте доступен только файл `AGENTS.md`, поэтому предлагаемая страница извлекает из него все упомянутые документационные пути, списки ADR, архитектурные ограничения и инструменты валидации. Страница пишется на русском и сохраняет Obsidian-совместимый Markdown.

### Proposed pages

#### `operations/documentation-map.md`

```markdown
# Карта документации

Карта документации проекта Макошь, основанная на правилах из [`AGENTS.md`](../AGENTS.md).
Используется как справочник для агентов и разработчиков при навигации по документации.

## Источники истины

При конфликтах используется следующий порядок источников (из `AGENTS.md`, раздел 2):

1. Текущий запрос пользователя.
2. `AGENTS.md` (корень репозитория).
3. ADR в `docs/adr/`.
4. Каноническая продуктовая и foundation-документация:
   - `docs/product/master-spec.md`
   - `docs/foundation/`
   - `docs/domains/`
   - `docs/engines/`
   - `docs/workflows/`
5. Архитектурная документация:
   - `docs/architecture/`
   - `docs/ai/agents/`
   - `docs/ui/`
6. Текущие файлы реализации (источник фактов о том, что уже реально реализовано).
7. Внешняя документация (только проверенная).

## Структура документации

### ADR (Architecture Decision Records)

- **Путь:** `docs/adr/`
- **Файлы:** `ADR-*.md`

Ключевые ADR (по состоянию из `AGENTS.md`, раздел 3):

| ADR | Название |
|---|---|
| `ADR-0001` | event sourcing is system spine |
| `ADR-0002` | Rust backend |
| `ADR-0004` | Tauri desktop shell |
| `ADR-0093` | Vue 3 frontend (supersedes ADR-0003) |
| `ADR-0005` | PostgreSQL primary store |
| `ADR-0006` | Tantivy full text search |
| `ADR-0008` | knowledge graph first |
| `ADR-0009` | local AI through Ollama |
| `ADR-0022` | no fine-tuning on private data |
| `ADR-0026` | desktop-first responsive UI |
| `ADR-0031` | temporary desktop-only UI scope |
| `ADR-0032` | Docker Compose development environment under `docker/` |
| `ADR-0041` | email provider ingestion foundation (Gmail, iCloud, IMAP) |
| `ADR-0042` | provider credential secret references and resolver boundary |
| `ADR-0046` | persistent dev mail cache and blob storage |
| `ADR-0054` | application settings store; user settings separate from provider accounts |
| `ADR-0055` | full email provider networking (read+write), supersedes ADR-0043 |
| `ADR-0056` | local API simplified auth with router-level `X-Макошь-Secret` |
| `ADR-0076` | host vault on macOS, supersedes ADR-0044 и ADR-0053 |
| `ADR-0077` | i18n с русским и английским интерфейсом (JSON словари, Svelte stores) |
| `ADR-0084` | Persona Intelligence System (supersedes Contact/Person CRM framing) |
| `ADR-0085` | Communication spine и Consistency/Contradiction Engine |
| `ADR-0086` | first-class Relationship persistence |
| `ADR-0087` | contradiction observation persistence |
| `ADR-0088` | obligation persistence |
| `ADR-0089` | decision persistence |

Устаревшие (Superseded) ADR сохраняются как исторические записи, но их требования **не** применяются в текущей реализации:
- `ADR-0003` (заменён через ADR-0093)
- `ADR-0043` (заменён через ADR-0055)
- `ADR-0044`, `ADR-0053` (заменены через ADR-0076)

### Продуктовая спецификация

- `docs/product/master-spec.md` — каноническая продуктовая спецификация Макошь как Personal Memory System.

### Foundation и домены

- `docs/foundation/` — foundation-документация.
- `docs/domains/` — документация по доменам (владеют долговременными сущностями).
- `docs/engines/` — документация по движкам (переиспользуемые механизмы: Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk, Polygraph).
- `docs/workflows/` — документация по рабочим процессам.

### Архитектура и UI

- `docs/architecture/` — общая архитектурная документация.
- `docs/ai/agents/` — документация по AI-агентам.
- `docs/ui/` — документация по пользовательскому интерфейсу.

### Выравнивание реализации

- `docs/refactoring/implementation-alignment-plan.md` — план выравнивания между документацией и кодом. Используется для фиксации расхождений, когда код использует устаревшие имена сущностей (например, `persons`, `person_id`, `contacts`) вместо канонических имён из `AGENTS.md`.

## Каноническая продуктовая модель (основные понятия)

Из `AGENTS.md`, раздел 3.1:

- **Persona** (персона), не контакт. Типы: `human`, `ai_agent`, `organization_proxy`, `system`.
- Одна **Owner Persona** с `is_self = true`.
- Первичные сущности: Communications, Knowledge, Memory, Relationships, Projects, Documents, Decisions, Obligations, Context.
- Коммуникационный позвоночник (Communication spine):
  ```
  Communication → Source Evidence → Extracted Knowledge → Memory → Context
  ```
- Движки (Engines) — переиспользуемые механизмы, порождающие производные представления, оценки, кандидаты и наблюдения. Включают Polygraph (Consistency/Contradiction Engine), который обнаруживает поддерживаемые доказательствами противоречия, но не перезаписывает память автоматически.

## Архитектурные ограничения

Из `AGENTS.md`, раздел 8:

- **Local-first** обязателен.
- **Knowledge graph** и **events** — первичные архитектурные концепции.
- Поисковые индексы и эмбеддинги — производное, перестраиваемое состояние.
- Вывод AI **не является** источником истины.
- Приватные данные **не используются** для дообучения моделей (fine-tuning).
- Провайдерские адаптеры должны сохранять **raw source provenance**.
- Полное read-write сетевое взаимодействие с провайдерами разрешено (ADR-0055); ограничение «только чтение» действует только для автоматических интеграционных тестов.
- Агенты и плагины используют **capability-based permissions**.
- **Мобильный UI вне скоупа** до замены ADR-0031.
- Docker-инфраструктура разработки размещается строго в `docker/` (ADR-0032).
- Защищённые локальные API-эндпоинты используют **router-level shared secret guard** из ADR-0056: `MAKOSH_LOCAL_API_SECRET` + заголовок `X-Макошь-Secret`.
- События локального API записываются в append-only `api_audit_log` (ADR-0039); токены и секреты **не сохраняются** в аудит.
- Учётные записи email-провайдеров: `gmail`, `icloud`, `imap` (ADR-0041). Конфигурация **не хранит** OAuth-токены, пароли приложений или пароли почтовых ящиков напрямую.

## Инструменты и конфигурация (из правил валидации)

Файлы и директории, упоминаемые в `AGENTS.md`, раздел 7:

- `docker/` — Docker Compose окружение разработки (ADR-0032).
- `Makefile` — цели валидации (`make validate`, `make backend-validate`, `make backend-test`).
- `package.json` — скрипты фронтенд-валидации (линтер, проверка типов, тесты, сборка).
- `crates/testkit/` — тестовый харнесс `makosh_test_session`, переиспользующий сессию PostgreSQL.

*Данная карта основана исключительно на содержимом `AGENTS.md` (по состоянию на момент генерации) и не включает файлы, которые могут существовать в репозитории, но не упомянуты в этом документе.*
```

### Source coverage

Из `AGENTS.md` в предложенной странице покрыты следующие факты:

- **Источники истины** (раздел 2) — полный список и порядок приоритетов.
- **Перечень ключевых ADR** (раздел 3) — все перечисленные ADR, включая отметки о superseded.
- **Каноническая продуктовая модель** (раздел 3.1) — описание Persona, Communication spine, движков, запрет на описание системы как CRM/почтового клиента и т.п.
- **Фаза реализации** (раздел 4) — допустимые и недопустимые действия (косвенно отражены в архитектурных ограничениях и карте).
- **Правила валидации** (раздел 7) — упоминание `Makefile`, `docker/`, `package.json`, `crates/testkit/`, команд валидации.
- **Архитектурные ограничения** (раздел 8) — все перечисленные ограничения (local-first, knowledge graph, email providers, local API auth, audit log, etc.).
- **Файл выравнивания** `docs/refactoring/implementation-alignment-plan.md` (раздел 3.1) — упоминание как места фиксации расхождений.

### Drift candidates

Потенциальные расхождения, видимые из контекста:

- `AGENTS.md` явно указывает, что текущая реализация может использовать устаревшие имена сущностей (`persons`, `person_id`, `contacts`) как «compatibility labels», тогда как каноническая модель требует `Persona` и связанные концепции. Для фиксации таких расхождений предусмотрен файл `docs/refactoring/implementation-alignment-plan.md`. Без доступа к текущему состоянию репозитория подтвердить или опровергнуть наличие конкретных расхождений невозможно.
- Иных кандидатов на drift из предоставленного контекста не выявлено.
