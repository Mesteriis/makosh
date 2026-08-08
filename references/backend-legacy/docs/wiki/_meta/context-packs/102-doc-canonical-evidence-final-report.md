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

- Chunk ID / ID чанка: `102-doc-canonical-evidence-final-report`
- Group / Группа: `canonical-evidence-final-report`
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

### `canonical-evidence-final-report.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/canonical-evidence-final-report.md`
- Size bytes / Размер в байтах: `243072`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```markdown
# Canonical Evidence / Review Inbox / Actions

Дата: 2026-06-19
Источник архитектуры: `docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`
Целевая схема: `External Systems -> Integrations -> Vault -> Observation Platform -> Ingestion -> Domains -> Knowledge -> Review -> Actions`

## 1. Честный статус

Refactor завершен и доказан текущим состоянием кода, активной документации,
guard-ов и validation gate.

Последний remaining gap был уже не в runtime-owner переписывании, а в
финальной синхронизации документации, терминологии и completion proof. Этот
gap в текущем периоде закрыт.

Этот файл является единственным каноническим отчетом текущего периода в корне проекта.

Текущая оценка готовности всего refactor:

- выполнено: `100%`;
- осталось: `0%`;
- текущий статус подтвержден completion audit matrix ниже и финальным validation gate.

## 1.2. Completion Audit Verdict

| Требование | Доказательство | Статус |
|---|---|---|
| `External Systems -> Integrations -> Vault -> Observation Platform -> Ingestion -> Domains -> Knowledge -> Review -> Actions` зафиксировано как целевая архитектура | `docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`, `docs/architecture/architecture-overview.md`, `docs/foundation/domain-map.md` | Доказано |
| Observation Platform = canonical evidence store, не часть Vault | `backend/src/platform/observations/**`, миграция `backend/migrations/0094_create_canonical_evidence_review_context.sql`, guard `scripts/check-architecture.mjs` запрещает `backend/src/vault/observations` | Доказано |
| Observation = evidence, not truth | `ADR-0096`, `docs/foundation/world-model.md`, owner flows идут через observation capture + review/domain promotion | Доказано |
| Observations append-only; provider deletion создает новую observation | live test `backend/tests/observations.rs::observations_are_append_only_and_survive_provider_deletion_against_postgres` | Доказано |
| Manual note / voice memo / browser capture создают observations без Vault | live tests в `backend/tests/observations.rs` для manual note, voice recording, browser capture | Доказано |
| Review существует как отдельный inbox domain | `backend/src/domains/review/**`, review routes в `backend/src/app/router/routes/review.rs` | Доказано |
| Review lifecycle `new/in_review/approved/promoted/dismissed/archived` | schema в миграции `0094...`, live tests `backend/tests/review_inbox.rs` | Доказано |
| Review promotion покрывает Personas, Organizations, Tasks, Decisions, Obligations, Relationships, Projects, Knowledge/Documents | `backend/tests/review_inbox.rs::review_can_materialize_promotions_for_core_target_domains_against_postgres` | Доказано |
| Tasks обязаны иметь provenance | guards в `backend/migrations/0104_add_task_provenance_reference_guard.sql`, checks/tests в `backend/tests/tasks.rs` | Доказано |
| Context Packs живут в `engines/context_packs/` как derived/rebuildable engine output | `backend/src/engines/context_packs/**`, runtime guard на legacy table access, active docs updated | Доказано |
| `engines/identity_resolution/` и `engines/relationships/` выделены отдельно | `backend/src/engines/identity_resolution/**`, `backend/src/engines/relationships/**`, `backend/src/engines/mod.rs` | Доказано |
| Запрещены `domains/signals`, `domains/events`, `domains/attention`, `domains/evidence`, Vault-owned observations | `scripts/check-architecture.mjs` + green run `node scripts/check-architecture.mjs` | Доказано |
| Ключевые workflow actions reuse observation evidence и materialize projection links | live suite `backend/tests/v1_workflow_actions.rs` | Доказано |
| Полный validation gate зеленый | `make validate` passed; внутри: backend tests green, `vitest` 156 files / 492 tests passed, `vue-tsc --noEmit && vite build` passed | Доказано |

## 1.1. Финальная валидация

Финальный validation pass завершен успешно.

- `make backend-validate` — passed
- `make validate` — passed
- frontend checks inside `make validate`:
  - `vitest run` — `156` test files passed, `492` tests passed
  - `vue-tsc --noEmit && vite build` — passed

Отдельно по ходу финального прохода были закрыты последние реальные блокеры:

- разбиение oversized `persons_api` tests, чтобы пройти architecture line-limit guard;
- разбиение oversized Telegram owner/runtime файлов, чтобы пройти `telegram_architecture`;
- cleanup stale architecture-boundary baseline и удаление test-only domain import debt в Telegram reactions tests.

## 2. Что сделано / что осталось / что дальше

Этот раздел ниже сохраняет исторический ledger slices текущего периода.
Колонка `Статус` в нем отражает состояние конкретного среза на момент его
выполнения, а не текущий итоговый verdict проекта. Актуальный итоговый статус
зафиксирован только в разделах `1` и `1.2`.

| Срез | Что сделано | Что осталось | Следующий шаг | Статус |
|---|---|---|---|---|
| Архитектурный baseline | Принят `ADR-0096`; зафиксированы инварианты `Observation = evidence`, `append-only`, `Vault does not own observations`, `Review = inbox domain`, `Context Packs = derived engine`. | Дочистить docs и guard baseline от legacy vocabulary и старых ownership трактовок. | Довести architecture guard и документацию до одной терминологии. | Частично |
| Observation Platform | Добавлен `backend/src/platform/observations/`; заведены `observations`, `observation_kind_definitions`, `observation_links`, `observation_ingestion_runs`; миграции `0094..0116`; добавлен `CONTRADICTION_OBSERVATION`. | Не все реальные write entrypoints уже проходят через canonical observation capture; часть покрыта только store/runtime слоями. | Системно пройти remaining manual/provider write surfaces по bounded context. | Частично |
| Vault boundary | `calendar_accounts`, `calendar_sources`, `task_provider_accounts` и теперь SQL ownership для `communication_provider_accounts` и `communication_provider_account_secret_refs` вынесены в `backend/src/vault/provider_accounts.rs`; mail `CommunicationIngestionStore` оставлен как compatibility façade, а email account-management, Gmail/IMAP setup, Telegram account lookup/lifecycle/capabilities, WhatsApp fixture account setup, outbox provider lookup, background sync account lookup и vault reconciliation уже ходят через `crate::vault::*Store`. | Remaining capability/session/config wiring, часть runtime/service call sites и compatibility routes еще местами размазаны по доменам и integrations. | Добить provider-account ownership, чтобы домены больше не были обязательным owner-ом account/config CRUD. | Частично |
| Vault-linked calendar account evidence | Linked calendar accounts, создаваемые из Gmail/iCloud setup и из host-vault reconciliation, теперь не просто upsert-ятся в `calendar_accounts`, а materialize observation trail через `vault::CalendarAccountStore`: account setup пишет `ObservationOriginKind::LocalRuntime`, reconciliation пишет `ObservationOriginKind::VaultSource`, обе ветки оставляют `observation_link` c `relationship_kind = linked_provider_upsert`. | Другие derived provider/config records в соседних bounded contexts еще нужно пройти тем же правилом. | Следующий ownership pass: derived provider/config/runtime records outside calendar/mail linked-account path. | Частично |
| Vault provider account and secret binding evidence | `communication_provider_accounts` и `communication_provider_account_secret_refs` теперь тоже materialize observation trail в owner-store `vault::provider_accounts`: обычные setup/runtime upsert/bind идут как `ObservationOriginKind::LocalRuntime`, host-vault reconciliation идет как `ObservationOriginKind::VaultSource`, observation links пишутся в `vault` domain для `communication_provider_account` и `communication_provider_secret_binding`. | Остались другие technical owner records, в том числе `task_provider_accounts` и возможные compatibility CRUD paths, которые еще нужно проверить и перевести по тому же контракту. | Следующий ownership pass: `task_provider_accounts` и оставшиеся non-domain technical records с direct durable writes. | Частично |
| Task provider owner cleanup | `task_provider_accounts` теперь создаются через owner-store `vault::TaskProviderStore` с evidence trail; старый write-capable compatibility store из `domains/tasks/core/providers.rs` убран, в domain layer оставлен только тип `TaskProviderAccount`. | Нужен live DB прогон нового observation assertion и затем такой же разбор remaining compatibility CRUD paths в других technical stores. | Следующий ownership pass: remaining compatibility CRUD stores that still hold direct durable writes. | Частично |
| Calendar account/source duplicate owner cleanup | Старые compatibility CRUD stores `domains/calendar/events/accounts.rs` и orphan `domains/calendar/events/sources.rs` удалены; `domains/calendar/events.rs` теперь оставляет единственный owner path через re-export `crate::vault::{CalendarAccountStore, CalendarSourceStore}`. | Дальше остаются другие technical stores вне calendar/tasks/mail provider metadata, где еще есть direct durable writes и может быть размазанное ownership. | Следующий ownership pass: remaining technical config/queue stores after `ai/control_center`. | Частично |
| AI control center evidence trail | `ai/control_center` mutation paths `create_provider`, `update_provider`, `record_consent`, `bind_api_key_secret` и `put_model_route` теперь работают в transaction-bound evidence contract: добавлены registry-backed observation kinds `AI_PROVIDER_ACCOUNT`, `AI_PROVIDER_SECRET_BINDING`, `AI_MODEL_ROUTE`, каждая durable mutation пишет canonical observation + `observation_link` в `domain = ai`. PostgreSQL regression test подтверждает полный trail для provider lifecycle, secret binding и model route. Architecture guard теперь также запрещает direct `ai_model_routes` mutations вне owner file. | Остались соседние technical queues/runtime states, например provider command queues и часть integration-owned write logs, которые еще живут без такого же canonical evidence trail. | Следующий technical pass: Telegram provider write commands и другие remaining queue/state owners с direct durable writes. | Частично |
| Telegram provider command queue evidence trail | `telegram_provider_write_commands` теперь materialize canonical evidence trail через owner-file `integrations/telegram/client/commands.rs`: добавлены observation kinds `TELEGRAM_PROVIDER_WRITE_COMMAND` и `TELEGRAM_PROVIDER_WRITE_COMMAND_STATUS`; enqueue, claim, retry, dead-letter, awaiting-provider, manual retry, stale recovery, reconciled и mismatch transitions пишут append-only observations и `observation_link` на `telegram/provider_write_command`. Reconciliation paths из `chat_reconciliation`, `chat_state`, `participants`, `reactions`, `lifecycle/provider_reconciliation` и `runtime/manager/topic_events` сведены к централизованным helpers вместо прямых SQL updates. Architecture guard теперь запрещает direct `INSERT/UPDATE telegram_provider_write_commands` вне owner-файла. | Еще остаются другие technical queue/state owners за пределами Telegram command queue и broader runtime/projection mutation surfaces, которые не все уже покрыты таким же contract. | Следующий technical pass: remaining queue/state tables outside Telegram command queue, затем новые guard rules на соседние runtime mutation owners. | Частично |
| AI agent run evidence trail | `ai_agent_runs` теперь тоже пишут canonical evidence trail: `AiRunStore::start_run`, `complete_run`, `fail_run` materialize observations `AI_AGENT_RUN` и `AI_AGENT_RUN_STATUS` с link на `ai/agent_run`. Existing integration test на AI answer run расширен assertion-ом на presence of canonical observations для run lifecycle. | Live verification этого пути зависит от `MAKOSH_TEST_DATABASE_URL`; без него test only compiles and skips. Остаются другие execution/job owners вне AI run store. | Следующий technical pass: `document_processing_jobs` и `communication_mail_sync_runs`. | Частично |
| Document processing job evidence trail | `document_processing_jobs` теперь пишут canonical evidence tr
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._
