---
chunk_id: 122-adr-docs-part-003
batch_id: batch-20260628T214902
group: docs
role: adr
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 122-adr-docs-part-003 — docs/adr

- Target index: [[decisions/adr-index]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Необходимо создать или обновить страницу русского индекса архитектурных решений (`decisions/adr-index.md`) в Obsidian-совместимой wiki проекта Макошь. Чанк содержит документы ADR-0051–ADR-0076 (кроме отсутствующего ADR-0075). Индекс должен перечислить все решения из этого набора с их статусами и краткими описаниями, опираясь только на встроенный в чанк исходный текст. Страница должна быть написана на русском языке, использовать синтаксис Obsidian (внутренние ссылки) и не добавлять внешних фактов, не подтверждённых предоставленными файлами ADR.

## Предложенные страницы

#### `decisions/adr-index.md`

````markdown
# Индекс архитектурных решений (ADR‑Index)

В этом документе перечислены архитектурные решения (ADR) проекта Макошь, начиная с ADR-0051. Более ранние решения описаны в других частях индекса.

## ADR-0051 – ADR-0060

- **``ADR-0051` (`ADR-0051-v5-whatsapp-web-companion-boundary`)`** – V5 WhatsApp Web Companion Boundary (`Proposed`)
  - Явная граница провайдера `whatsapp_web` для личного WhatsApp. Первый путь – фикстуры/ручное состояние; живая сессия заблокирована до появления видимого десктоп-рантайма. Секреты не хранятся в PostgreSQL, учётные данные резолвятся по `account_id + secret_purpose`, raw-записи – append‑only. Отправка в реальном времени не входит в V5-основание.

- **``ADR-0052` (`ADR-0052-capability-runtime-and-action-confirmation-policy`)`** – Capability Runtime and Action Confirmation Policy (`Proposed`)
  - Централизованная проверка разрешений на уровне backend‑приложения. Классификация действий: `read`, `local_write`, `provider_write`, `destructive`, `export`, `secret_access`, `automation`. Автоматизация требует ограниченных политик с привязкой к аккаунту, шаблону, получателю, лимитам. Временные заголовки `Authorization: Bearer <MAKOSH_LOCAL_API_TOKEN>` и `X-Макошь-Actor-Id` остаются до замены на capability‑рантайм. Аудит высокорисковых действий обязателен, секреты в аудит не попадают. Плагины ненадёжны по умолчанию, активируются через декларативные манифесты и ограниченные представления данных.

- **``ADR-0053` (`ADR-0053-database-backed-encrypted-secret-vault`)`** – Database-Backed Encrypted Secret Vault (`Superseded by ADR-0076`)
  - Предлагал хранить зашифрованные секреты провайдеров в PostgreSQL в таблице `encrypted_secret_vault_entries` с использованием AES‑256‑GCM и Argon2id, оставляя ключ `MAKOSH_SECRET_VAULT_KEY` вне базы. Заменён решением ADR‑0076, которое выносит новые шифрованные полезные нагрузки из PostgreSQL в хост‑хранилище.

- **``ADR-0054` (`ADR-0054-application-settings-store`)`** – Application Settings Store (`Proposed`)
  - Хранение пользовательских несекретных настроек в PostgreSQL в таблице `application_settings`. Значения – типизированный JSONB (`boolean`, `integer`, `string`, `json`). Ключи не должны содержать `secret`, `password`, `token`, `credential` или `private_key`. Провайдерские аккаунты остаются в собственных таблицах. Стартап‑проверка (repair) создаёт недостающие настройки и сбрасывает некорректные до дефолтов.

- **``ADR-0055` (`ADR-0055-full-email-provider-networking`)`** – Full Email Provider Networking (Read + Write) (`Accepted`, заменяет ADR-0043)
  - Снимает ограничение только на чтение для email‑провайдеров. Добавлены SMTP‑отправка, IMAP‑мутации флагов и ящиков, Gmail‑мутации и работа с черновиками. SMTP‑пароли хранятся как отдельные секреты с purpose `smtp_password`. Автоматические интеграционные тесты должны использовать только пути чтения.

- **``ADR-0056` (`ADR-0056-local-api-simplified-auth`)`** – Local API – Simplified Auth (`Accepted`, заменяет ADR‑0037, 0038, 0040)
  - Единая проверка разделяемого секрета через `tower::layer` на уровне роутера. Удалены `MAKOSH_LOCAL_API_TOKEN`, `x-makosh-actor-id`, `verify_local_api_capability()` и связанные структуры. Все аудит‑записи используют актора `"makosh-frontend"`.

- **``ADR-0057` (`ADR-0057-person-memory-and-provenance`)`** – Person Memory and Provenance System (`Proposed`)
  - Факты, карточки памяти, предпочтения и экспертиза хранятся в доменных таблицах с обязательными колонками `source`, `confidence`, `last_verified_at`. Движок обогащения пишет через эти таблицы, никогда не изменяя профиль напрямую. Предусмотрено устаревание памяти и обнаружение конфликтующих фактов.

- **``ADR-0058` (`ADR-0058-person-enrichment-engine`)`** – Person Enrichment Engine Boundary (`Proposed`)
  - Выделенный `EnrichmentEngine` с подключаемыми провайдерами (GitHub, LinkedIn, публичные источники). Результаты проходят через `enrichment_results` со статусами `pending`/`applied`/`rejected`/`conflict`. Автоматическое применение разрешено только для высокодоверительных фактов из проверенных источников.

- **``ADR-0059` (`ADR-0059-person-communication-dna`)`** – Person Communication DNA and Personas (`Superseded by ADR-0084`)
  - Хранил атрибуты коммуникационного стиля (`communication_style`, `verbosity`, `technical_depth`, `call_preference` и др.) прямо в таблице `persons`, а персоны – в `person_personas`. Заменён ADR‑0084, который делает Persona корневой сущностью домена.

- **``ADR-0060` (`ADR-0060-person-timeline-and-graph-integration`)`** – Person Timeline and Graph Integration (`Proposed`)
  - События отношений хранятся в `relationship_events` с возможностью перестроения. Графовые связи интегрированы через существующие таблицы `graph_nodes`/`graph_edges` с новыми типами отношений (`person_has_identity`, `person_works_at_organization` и т. д.).

## ADR-0061 – ADR-0070

- **``ADR-0061` (`ADR-0061-organization-as-first-class-entity`)`** – Organization as First-Class Domain Entity (`Proposed`)
  - Организации становятся самостоятельными доменными сущностями с ID `org:v1:{nanos}`. Связь с персонами – через `organization_contact_links` (many‑to‑many, с role/department/primary‑flag). Поле `organization_reference` на персоне остаётся как кешированное значение.

- **``ADR-0062` (`ADR-0062-organization-identity-and-resolution`)`** – Organization Identity and Resolution (`Proposed`)
  - Идентификаторы организаций (домены, VAT/CIF/NIF, GitHub‑орги, LinkedIn-страницы и пр.) хранятся в `organization_identities` с provenance. Разрешение коллизий основано на совпадении доменов, VAT, имён и общих контактов. Слияние обратимо.

- **``ADR-0063` (`ADR-0063-organization-passive-osint-boundary`)`** – Organization Passive OSINT Boundary (`Proposed`)
  - Обогащение из публичных источников (веб-сайт, GitHub, LinkedIn, VIES, публичные реестры) только через пассивные наблюдения. Запрещено активное сканирование, брутфорс, обход контроля доступа.

- **``ADR-0064` (`ADR-0064-organization-memory-and-provenance`)`** – Organization Memory and Provenance (`Proposed`)
  - Факты, карточки памяти, предпочтения и события организаций – в отдельных таблицах с обязательными `source`, `confidence`, `last_verified_at`. Снапшоты (`organization_snapshots`) дают diff истории.

- **``ADR-0065` (`ADR-0065-organization-portals-procedures-playbooks`)`** – Organization Portals, Procedures, and Playbooks (`Proposed`)
  - Порты (`organization_portals`), процедуры (`organization_procedures` в JSONB), сценарии (`organization_playbooks` – триггеры/шаги/режим подтверждения) и шаблоны (`organization_templates`). В v1 сценарии хранятся как данные, автоматического исполнения нет.

- **``ADR-0066` (`ADR-0066-organization-graph-integration`)`** – Organization Graph Integration (`Proposed`)
  - Организации участвуют в существующих `graph_nodes`/`graph_edges` с новыми типами (`org_has_domain`, `org_has_contact`, `org_has_document`, `org_involved_in_project`, `org_parent_of`). Дополнительно используется прямой справочник `related_organizations`.

- **``ADR-0067` (`ADR-0067-calendar-multi-provider-architecture`)`** – Calendar as First-Class Domain with Multi-Provider Architecture (`Proposed`)
  - Календарь – самостоятельный домен с `calendar_account_id = cal:v1:{uuid}`. Мультипровайдерная архитектура с моделью возможностей (read/write/delete/recurring/attendees/…). События несут полную информацию об источнике. Провайдер-синхронизация отложена на будущее; домен пригоден для локальных событий сразу.

- **``ADR-0068` (`ADR-0068-calendar-event-as-graph-node`)`** – Calendar Event as Knowledge Graph Node (`Proposed`)
  - События включаются в граф знаний через `event_relations` (связи с персоной, организацией, проектом, документом, задачей, письмом и т.д.) и `event_participants`. Контекст‑пак (`event_context_packs`) – материализованный JSONB‑снапшот связанных данных для быстрого доступа.

- **``ADR-0069` (`ADR-0069-calendar-intelligence-heuristic-fallbacks`)`** – Calendar Intelligence Layer with Heuristic Fallbacks (`Proposed`)
  - Классификация событий, оценка важности, готовности, рисков, брифинги, генерация повестки и поиск – на детерминированных эвристиках, без обязательного Ollama. Ollama – опциональное улучшение без изменения API.

- **``ADR-0070` (`ADR-0070-tasks-first-class-domain`)`** – Tasks as First-Class Domain with Local Overlay (`Proposed`)
  - Задачи – первый в домене с `task_id = task:v1:{nanos_hex}`. Локальный оверлей (AI‑summary, приватные заметки, контекст) никогда не синхронизируется с внешними трекерами. Мультипровайдерная архитектура готова в схеме, но синхронизация (Jira/YouTrack/GitHub) отложена.

## ADR-0071 – ADR-0076

- **``ADR-0071` (`ADR-0071-task-context-evidence-provenance`)`** – Task Context and Evidence Provenance (`Proposed`)
  - Контекст‑пак задачи – материализованный JSONB‑снапшот (сводка, открытые вопросы, блокеры, риски, следующее действие). Для AI‑извлечённых задач хранится Evidence с `source_type`, `source_id`, `quote`, `confidence`. Задачи с низким confidence попадают в suggested inbox.

- **``ADR-0072` (`ADR-0072-task-intelligence-heuristic-fallbacks`)`** – Task Intelligence with Heuristic Fallbacks (`Proposed`)
  - Приоритет, риск, готовность, детекция пропущенного контекста и подсказка следующего действия – на эвристиках без обязательного Ollama. Оценки хранятся непосредственно в строке задачи для сортировки и фильтрации.

- **``ADR-0073` (`ADR-0073-backend-module-organization`)`** – Backend Module Organization (`Accepted`)
  - Семислойная структура backend‑крейта: `app/`, `domains/`, `engines/`, `integrations/`, `ai/`, `workflows/`, `platform/`. Домены не импортируют друг друга напрямую – только через events или контракты. Порог размера файла – 700 строк, с обязательным header‑комментарием при превышении.

- **``ADR-0074` (`ADR-0074-person-multi-channel-identity-model`)`** – Person Multi-Channel Identity Model (`Accepted`)
  - Сохранён стабильный текстовый ID персоны (`person:v1:email:...`). Мультиканальные идентификаторы (email, Telegram, WhatsApp, phone, GitHub, LinkedIn) добавляются в `person_identities`. Резолюция слияния/разделения работает с текущими текстовыми ID. Переход к opaque UUID‑идентификаторам требует отдельного ADR.

- **ADR-0075 отсутствует в данном контексте.**

- **``ADR-0076` (`ADR-0076-host-vault-on-macos`)`** – Host Vault on macOS (`Accepted`, заменяет ADR‑0044, 0053)
  - Выделенное хост‑хранилище `~/.makosh/vault` с SQLite‑базой `vault.db` для зашифрованных секретов. PostgreSQL содержит только несекретные метаданные и `secret_references` (`store_kind = host_vault`). Мастер‑ключ – на macOS Keychain, в development‑режиме – через `MAKOSH_DEV_KEY_PATH`. Криптография: XChaCha20‑Poly1305 с AAD, OS‑энтропия, mouse‑entropy при onboarding. Восстановление обязательно, биометрия/Keychain только для разблокировки.
````

## Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `ADR-0051-v5-whatsapp-web-companion-boundary.md` | Граница `whatsapp_web`, состояние фикстуры/блокировки, вне PostgreSQL секретов, `account_id + secret_purpose`, raw‑записи append‑only, отсутствие live‑отправки в V5 |
| `ADR-0052-capability-runtime-and-action-confirmation-policy.md` | Централизованная граница capability‑проверок, классификация действий (7 видов), scoped‑разрешения, временные заголовки, аудит высокорисковых действий, плагины по умолчанию не доверены |
| `ADR-0053-database-backed-encrypted-secret-vault.md` | Статус Superseded, предложение хранить шифрованные секреты в PostgreSQL, AES‑256‑GCM+Argon2id, `MAKOSH_SECRET_VAULT_KEY` вне базы |
| `ADR-0054-application-settings-store.md` | Таблица `application_settings`, типизированный JSONB, запрет секретоподобных ключей, стартап‑repair, провайдерские аккаунты отдельно |
| `ADR-0055-full-email-provider-networking.md` | Полный email‑доступ (read+write), SMTP, IMAP/Gmail мутации, работы с черновиками, секреты по `account_id + secret_purpose`, тесты только на чтение |
| `ADR-0056-local-api-simplified-auth.md` | Единый `tower::layer` с разделяемым секретом, удаление `MAKOSH_LOCAL_API_TOKEN`, `x-makosh-actor-id`, актор `"makosh-frontend"` |
| `ADR-0057-person-memory-and-provenance.md` | Доменные таблицы фактов/карточек памяти/предпочтений/экспертизы, обязательные `source`/`confidence`/`last_verified_at`, decay памяти, обнаружение конфликтов |
| `ADR-0058-person-enrichment-engine.md` | `EnrichmentEngine`, подключаемые провайдеры, статусы результатов, пользовательское подтверждение для результатов с низким/средним confidence |
| `ADR-0059-person-communication-dna.md` | Статус Superseded, атрибуты DNA в `persons`, `person_personas`; заменён ADR‑0084 |
| `ADR-0060-person-timeline-and-graph-integration.md` | `relationship_events`, перестраиваемая проекция, использование `graph_nodes`/`graph_edges` с новыми типами отношений |
| `ADR-0061-organization-as-first-class-entity.md` | Организация – сущность с `org:v1:{nanos}`, `organization_contact_links` (many‑to‑many), кешированное `organization_reference` на персоне |
| `ADR-0062-organization-identity-and-resolution.md` | `organization_identities` со статусами, разрешение по доменам/VAT/именам/контактам, обратимое слияние |
| `ADR-0063-organization-passive-osint-boundary.md` | Пассивное обогащение из публичных API (website, GitHub, LinkedIn, VIES, реестры), запрет активного сканирования |
| `ADR-0064-organization-memory-and-provenance.md` | Таблицы фактов/памяти/предпочтений/событий организации, снапшоты и diff истории |
| `ADR-0065-organization-portals-procedures-playbooks.md` | Порты, процедуры (JSONB), сценарии (триггеры/шаги/подтверждение), шаблоны; сценарии только как данные в v1 |
| `ADR-0066-organization-graph-integration.md` | Участие в `graph_nodes`/`graph_edges`, новые типы рёбер, `related_organizations` таблица |
| `ADR-0067-calendar-multi-provider-architecture.md` | `calendar_account_id = cal:v1:{uuid}`, мультипровайдерная модель возможностей, source‑identity событий, провайдер‑синхронизация отложена |
| `ADR-0068-calendar-event-as-graph-node.md` | `event_relations`, `event_participants`, контекст‑пак (JSONB), интеграция с графом |
| `ADR-0069-calendar-intelligence-heuristic-fallbacks.md` | Эвристики для классификации/important/ readiness/risk/brief/поиска, Ollama опционален |
| `ADR-0070-tasks-first-class-domain.md` | `task_id = task:v1:{nanos_hex}`, локальный оверлей не синхронизируется, мультипровайдерная схема готова, синхронизация отложена |
| `ADR-0071-task-context-evidence-provenance.md` | Evidence с `source_type`/`source_id`/`quote`/`confidence`, suggested inbox для низкодоверительных задач |
| `ADR-0072-task-intelligence-heuristic-fallbacks.md` | Приоритет/риск/готовность/детекция пропусков/следующее действие – эвристики; Ollama опционален |
| `ADR-0073-backend-module-organization.md` | Семислойная структура `app`, `domains`, `engines`, `integrations`, `ai`, `workflows`, `platform`, domain‑isolation, порог 700 строк |
| `ADR-0074-person-multi-channel-identity-model.md` | Текстовый `person_id`, `person_identities` для мультиканальности, запрет на переход к opaque ID без отдельного ADR |
| `ADR-0076-host-vault-on-macos.md` | Хост‑хранилище `~/.makosh/vault`, SQLite `vault.db`, `store_kind = host_vault`, macOS Keychain для мастер‑ключа, XChaCha20‑Poly1305, обязательное восстановление |

## Исходные файлы

- [`docs/adr/ADR-0051-v5-whatsapp-web-companion-boundary.md`](../../../adr/ADR-0051-v5-whatsapp-web-companion-boundary.md)
- [`docs/adr/ADR-0052-capability-runtime-and-action-confirmation-policy.md`](../../../adr/ADR-0052-capability-runtime-and-action-confirmation-policy.md)
- [`docs/adr/ADR-0053-database-backed-encrypted-secret-vault.md`](../../../adr/ADR-0053-database-backed-encrypted-secret-vault.md)
- [`docs/adr/ADR-0054-application-settings-store.md`](../../../adr/ADR-0054-application-settings-store.md)
- [`docs/adr/ADR-0055-full-email-provider-networking.md`](../../../adr/ADR-0055-full-email-provider-networking.md)
- [`docs/adr/ADR-0056-local-api-simplified-auth.md`](../../../adr/ADR-0056-local-api-simplified-auth.md)
- [`docs/adr/ADR-0057-person-memory-and-provenance.md`](../../../adr/ADR-0057-person-memory-and-provenance.md)
- [`docs/adr/ADR-0058-person-enrichment-engine.md`](../../../adr/ADR-0058-person-enrichment-engine.md)
- [`docs/adr/ADR-0059-person-communication-dna.md`](../../../adr/ADR-0059-person-communication-dna.md)
- [`docs/adr/ADR-0060-person-timeline-and-graph-integration.md`](../../../adr/ADR-0060-person-timeline-and-graph-integration.md)
- [`docs/adr/ADR-0061-organization-as-first-class-entity.md`](../../../adr/ADR-0061-organization-as-first-class-entity.md)
- [`docs/adr/ADR-0062-organization-identity-and-resolution.md`](../../../adr/ADR-0062-organization-identity-and-resolution.md)
- [`docs/adr/ADR-0063-organization-passive-osint-boundary.md`](../../../adr/ADR-0063-organization-passive-osint-boundary.md)
- [`docs/adr/ADR-0064-organization-memory-and-provenance.md`](../../../adr/ADR-0064-organization-memory-and-provenance.md)
- [`docs/adr/ADR-0065-organization-portals-procedures-playbooks.md`](../../../adr/ADR-0065-organization-portals-procedures-playbooks.md)
- [`docs/adr/ADR-0066-organization-graph-integration.md`](../../../adr/ADR-0066-organization-graph-integration.md)
- [`docs/adr/ADR-0067-calendar-multi-provider-architecture.md`](../../../adr/ADR-0067-calendar-multi-provider-architecture.md)
- [`docs/adr/ADR-0068-calendar-event-as-graph-node.md`](../../../adr/ADR-0068-calendar-event-as-graph-node.md)
- [`docs/adr/ADR-0069-calendar-intelligence-heuristic-fallbacks.md`](../../../adr/ADR-0069-calendar-intelligence-heuristic-fallbacks.md)
- [`docs/adr/ADR-0070-tasks-first-class-domain.md`](../../../adr/ADR-0070-tasks-first-class-domain.md)
- [`docs/adr/ADR-0071-task-context-evidence-provenance.md`](../../../adr/ADR-0071-task-context-evidence-provenance.md)
- [`docs/adr/ADR-0072-task-intelligence-heuristic-fallbacks.md`](../../../adr/ADR-0072-task-intelligence-heuristic-fallbacks.md)
- [`docs/adr/ADR-0073-backend-module-organization.md`](../../../adr/ADR-0073-backend-module-organization.md)
- [`docs/adr/ADR-0074-person-multi-channel-identity-model.md`](../../../adr/ADR-0074-person-multi-channel-identity-model.md)
- [`docs/adr/ADR-0076-host-vault-on-macos.md`](../../../adr/ADR-0076-host-vault-on-macos.md)

## Кандидаты на drift

На основании предоставленного контекста видны следующие потенциальные расхождения:

1. **ADR-0053 (Database‑Backed Encrypted Secret Vault) объявлен устаревшим (Superseded by ADR-0076).** Если в кодовой базе или документации до сих пор присутствует активное использование таблицы `encrypted_secret_vault_entries` для новых провайдерских учётных данных, либо старые ADR (0016, 0042, 0044) ссылаются на ADR‑0053 как на действующий стандарт, это drift. ADR‑0076 требует, чтобы новые секреты размещались в хост‑хранилище, а `encrypted_secret_vault_entries` оставался только для наследия/миграции.

2. **ADR-0059 (Person Communication DNA and Personas) заменён решением ADR-0084.** Поскольку ADR‑0084 не входит в данный чанк, невозможно подтвердить актуальность структуры `person_personas` и DNA‑колонок в `persons`. Любое использование этих таблиц как целевой модели домена (вместо миграции к модели Persona как корневой сущности) будет drift.

3. **ADR-0055 (Full Email) заменяет ADR-0043.** Если где-либо сохраняется упоминание read‑only ограничения email‑адаптеров как актуального политики, это drift.

4. **ADR-0056 (Local API Simplified Auth) заменяет ADR-0037, 0038, 0040.** Присутствие в коде `MAKOSH_LOCAL_API_TOKEN`, `x-makosh-actor-id`, `verify_local_api_capability()` и связанных структур, не удалённых в пользу единого router‑уровня `tower::layer`, является прямым drift.

5. **ADR-0076 (Host Vault) заменяет ADR-0044 и ADR-0053.** Если конфигурация всё ещё использует `MAKOSH_SECRET_VAULT_PATH` как первичный путь записи, или файловое хранилище `encrypted_vault` (из ADR‑0044) рассматривается как основной механизм, это расходится с решением о переносе в хост‑хранилище с `vault.db` и `store_kind = host_vault`.

Других расхождений в предоставленном наборе документов не обнаружено.
