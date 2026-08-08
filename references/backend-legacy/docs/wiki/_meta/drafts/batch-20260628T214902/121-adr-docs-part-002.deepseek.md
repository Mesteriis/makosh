### Summary / Резюме

Необходимо создать индексную страницу `decisions/adr-index.md` для русской Obsidian wiki на основе встроенных ADR-документов (ADR-0026 – ADR-0050). Страница должна содержать перечень всех предоставленных ADR с указанием статуса, названия и краткого описания на русском языке. Это обеспечит навигацию по проектным решениям.

### Proposed pages / Предлагаемые страницы

#### `decisions/adr-index.md`

```markdown
# Индекс архитектурных решений (ADR)

Этот индекс включает все ADR-документы, предоставленные в текущем чанке. Статус и описание основаны исключительно на тексте исходных файлов.

| ADR | Статус | Название | Краткое описание |
|-----|--------|----------|------------------|
| ADR-0026 | Proposed | Desktop First Responsive UI | Проектировать интерфейс с приоритетом настольных экранов (desktop-first) и адаптивной вёрсткой для разных размеров окон, не закладываясь на мобильное упрощение. |
| ADR-0027 | Proposed | Capability Based Permission Model | Использовать модель разрешений на основе возможностей (capabilities) для агентов, плагинов и внешних действий. |
| ADR-0028 | Proposed | Backup and Restore as Core Feature | Рассматривать резервное копирование и восстановление как часть продуктовой архитектуры, а не второстепенную операционную задачу. |
| ADR-0029 | Proposed | Explicit Schema Evolution | Использовать явное версионирование схем, миграции и проверки совместимости для долгоживущих данных. |
| ADR-0030 | Proposed | Documentation First Monorepo | Начать проект с монорепозитория, управляемого документацией, с каталогами для docs, backend, frontend, infrastructure, tools и examples. |
| ADR-0031 | Temporary | Temporary Desktop Only UI Scope | До пересмотра этого ADR не проектировать, не реализовывать и не валидировать мобильный UI; scope — только ПК/ноутбуки. |
| ADR-0032 | Proposed | Docker Compose Development Environment | Использовать Docker Compose для локальной инфраструктуры разработки, хранить Docker-файлы в `docker/`. |
| ADR-0033 | Proposed | Backend Managed Local Schema Migrations | Возложить на Rust-бэкенд применение локальных миграций PostgreSQL при старте; миграции в `backend/migrations/`. |
| ADR-0034 | Proposed | Event Replay and Projection Cursors | Использовать `event_log.position` как порядок воспроизведения и `projection_cursors` для сохранения прогресса проекций. |
| ADR-0035 | Proposed | Local Event API Command Boundary | Предоставить локальный API для добавления и чтения канонических событий: `POST /api/events`, `GET /api/events/{event_id}`. |
| ADR-0036 | Proposed | Projection Runner Checkpoint Semantics | Проекционные воркеры сохраняют курсор только после успешной обработки события; сбойные события не пропускаются. |
| ADR-0037 | Superseded by ADR-0038 | Local Write Capability Token | Был введён временный токен `MAKOSH_LOCAL_WRITE_TOKEN` для защиты мутирующих endpoint'ов; заменён на ADR-0038. |
| ADR-0038 | Temporary | Local Event API Capability Token | Временный локальный API-токен `MAKOSH_LOCAL_API_TOKEN` защищает и чтение, и запись событий; `MAKOSH_LOCAL_WRITE_TOKEN` сохранён как fallback. |
| ADR-0039 | Proposed | Local Event API Access Audit Log | Создана таблица `api_audit_log` для журналирования обращений к локальному API событий; аудит не влияет на поток доменных событий. |
| ADR-0040 | Superseded by ADR-0056 | Local API Actor Identity | Требовал заголовок `X-Макошь-Actor-Id` для идентификации клиента; заменён на более полную модель в ADR-0056. |
| ADR-0041 | Proposed | Email Provider Ingestion Foundation | Определён провайдер-нейтральный контракт хранения для Gmail, iCloud и IMAP: `communication_raw_records`, чекпоинты, отделение секретов. |
| ADR-0042 | Superseded by ADR-0053 | Secret References for Provider Credentials | Секреты представлены ссылками (`secret_references`) без хранения значений в PostgreSQL; заменён на vault-модель с зашифрованными пейлоадами. |
| ADR-0043 | Superseded by ADR-0055 | Read-Only Email Provider Networking | Ограничивал сетевое взаимодействие с почтовыми провайдерами только чтением; отменено для production, оставлено только для автотестов. |
| ADR-0044 | Superseded by ADR-0076 | Account Setup and Encrypted Secret Vault | Добавлял локальный зашифрованный vault для OAuth-токенов и паролей; заменён хранилищем `~/.makosh/vault` в ADR-0076. |
| ADR-0045 | Proposed | Graph Core Projection | Вводит реляционные таблицы графа (`graph_nodes`, `graph_edges`, `graph_evidence`) как перестраиваемую проекцию поверх контактов, сообщений и документов. |
| ADR-0046 | Proposed | Persistent Dev Mail Cache and Blob Storage | Разделяет хранение метаданных в PostgreSQL и «тяжёлых» почтовых blob'ов (`.eml`, вложения) в локальном blob-хранилище `docker/data/mail/`. |
| ADR-0047 | Proposed | Project Memory Spine | Добавляет модель проектов (`projects`) и проекционные связи с сообщениями, документами, персонами; связи выводятся по ключевым словам. |
| ADR-0048 | Proposed | Project Link Review Workflow | Реализует событийно-ориентированный рабочий процесс проверки связей проектов: подтверждение/отклонение через `project.link_review_state_changed`. |
| ADR-0049 | Proposed | V3 Local AI Runtime and Retrieval | Вводит тонкий локальный AI-рантайм на базе Ollama (`qwen3:4b`, `qwen3-embedding:4b`), pgvector `halfvec(2560)` и аудируемые AI-запуски. |
| ADR-0050 | Proposed | V4 Telegram Client, Policy Automation and Call Intelligence | Расширяет систему клиентом Telegram (TDLib/Bot API), контролируемой автоматизацией отправки сообщений и локальной транскрипцией звонков. |

*Примечание: перечислены только ADR, предоставленные в чанке `121-adr-docs-part-002`. ADR, на которые ссылаются как на заменяющие (0053, 0055, 0056, 0076, 0081), в данном контексте отсутствуют.*
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---------------|----------------|
| `docs/adr/ADR-0026-desktop-first-responsive-ui.md` | Название, статус Proposed, решение проектировать desktop-first с адаптивной вёрсткой. |
| `docs/adr/ADR-0027-capability-based-permission-model.md` | Название, статус Proposed, решение использовать capability-based модель разрешений. |
| `docs/adr/ADR-0028-backup-and-restore-as-core-feature.md` | Название, статус Proposed, решение считать бэкап/восстановление частью архитектуры. |
| `docs/adr/ADR-0029-explicit-schema-evolution.md` | Название, статус Proposed, решение о явном версионировании схем и миграциях. |
| `docs/adr/ADR-0030-documentation-first-monorepo.md` | Название, статус Proposed, решение начать с documentation-first монорепо. |
| `docs/adr/ADR-0031-temporary-desktop-only-ui-scope.md` | Название, статус Temporary, ограничение scope’а только десктопным UI. |
| `docs/adr/ADR-0032-docker-compose-development-environment.md` | Название, статус Proposed, решение использовать Docker Compose для dev-окружения. |
| `docs/adr/ADR-0033-backend-managed-local-schema-migrations.md` | Название, статус Proposed, решение о бэкенд-управляемых миграциях. |
| `docs/adr/ADR-0034-event-replay-and-projection-cursors.md` | Название, статус Proposed, решение об использовании `event_log.position` и `projection_cursors`. |
| `docs/adr/ADR-0035-local-event-api-command-boundary.md` | Название, статус Proposed, решение о локальном API событий (`POST /api/events`, `GET /api/events/{event_id}`). |
| `docs/adr/ADR-0036-projection-runner-checkpoint-semantics.md` | Название, статус Proposed, решение о семантике чекпоинтов проекционных воркеров. |
| `docs/adr/ADR-0037-local-write-capability-token.md` | Название, статус Superseded by ADR-0038, исходное решение о `MAKOSH_LOCAL_WRITE_TOKEN`. |
| `docs/adr/ADR-0038-local-event-api-capability-token.md` | Название, статус Temporary, решение о `MAKOSH_LOCAL_API_TOKEN` с fallback на write-токен. |
| `docs/adr/ADR-0039-local-event-api-access-audit-log.md` | Название, статус Proposed, решение о `api_audit_log` и правилах аудита. |
| `docs/adr/ADR-0040-local-api-actor-identity.md` | Название, статус Superseded by ADR-0056, требование `X-Макошь-Actor-Id`. |
| `docs/adr/ADR-0041-email-provider-ingestion-foundation.md` | Название, статус Proposed, контракт хранения для Gmail/iCloud/IMAP, `communication_raw_records`, чекпоинты. |
| `docs/adr/ADR-0042-secret-references-for-provider-credentials.md` | Название, статус Superseded by ADR-0053, решение о `secret_references`. |
| `docs/adr/ADR-0043-read-only-email-provider-networking.md` | Название, статус Superseded by ADR-0055, исходное ограничение read-only. |
| `docs/adr/ADR-0044-account-setup-and-encrypted-secret-vault.md` | Название, статус Superseded by ADR-0076, решение о локальном зашифрованном vault. |
| `docs/adr/ADR-0045-graph-core-projection.md` | Название, статус Proposed, решение о реляционном графе как проекции. |
| `docs/adr/ADR-0046-persistent-dev-mail-cache-and-blob-storage.md` | Название, статус Proposed, разделение хранения между PostgreSQL и blob-хранилищем. |
| `docs/adr/ADR-0047-project-memory-spine.md` | Название, статус Proposed, модель `projects` и проекционные связи. |
| `docs/adr/ADR-0048-project-link-review-workflow.md` | Название, статус Proposed, рабочий процесс проверки связей проектов. |
| `docs/adr/ADR-0049-v3-local-ai-runtime-and-retrieval.md` | Название, статус Proposed, AI-рантайм на Ollama, pgvector `halfvec(2560)`, аудит запусков. |
| `docs/adr/ADR-0050-v4-telegram-client-policy-and-call-intelligence.md` | Название, статус Proposed, Telegram-клиент, автоматизация по политикам, транскрипция звонков. |

### Drift candidates / Кандидаты на drift

- **Незавершённые ссылки на заменяющие ADR:** ADR-0037 заменён ADR-0038 (присутствует), однако ADR-0040 ссылается на ADR-0056, ADR-0042 — на ADR-0053, ADR-0043 — на ADR-0055, ADR-0044 — на ADR-0053 и ADR-0076, ADR-0049 упоминает поправку ADR-0081. Ни один из этих заменяющих/дополняющих ADR не входит в данный чанк. Индекс, построенный только на предоставленных документах, не отражает финальные актуальные решения — это создаёт риск расхождения (drift) между индексом и фактическим состоянием репозитория.
- **Временные ADR без видимого срока действия/замены:** ADR-0031 и ADR-0038 имеют статус Temporary, но в контексте чанка отсутствуют указания на условия их отмены или замены. Если со временем эти решения устареют без создания новых ADR, индекс может вводить в заблуждение.
- **Отсутствие ADR, предшествующих 0026:** Индекс начинается с ADR-0026; ADR c меньшими номерами (например, упомянутые в текстах ADR-0001, ADR-0008, ADR-0015, ADR-0016, ADR-0019, ADR-0022, ADR-0023) не предоставлены. Это ожидаемо для данного чанка, но страница `adr-index.md` не будет полным индексом всех решений проекта.
