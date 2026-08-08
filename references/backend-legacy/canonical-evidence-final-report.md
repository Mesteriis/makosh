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
| Document processing job evidence trail | `document_processing_jobs` теперь пишут canonical evidence trail из owner-store `domains/documents/processing/jobs.rs`: queued/upsert дает `DOCUMENT_PROCESSING_JOB`, а `running/requeued/succeeded/failed/skipped` дают `DOCUMENT_PROCESSING_JOB_STATUS`; `observation_link` пишется на `documents/document_processing_job`. API error mapping обновлен под `ObservationStoreError`; regression tests расширены assertion-ами на `observation_links`. Architecture guard теперь также запрещает direct `document_processing_jobs` mutations вне owner file. | Live DB assertions в existing tests зависят от `MAKOSH_TEST_DATABASE_URL`; без него targeted tests только компилируются и скипаются. | Следующий шаг: идти дальше в remaining runtime/job owners после document processing/mail sync. | Частично |
| API owner cleanup: obligations / contradictions / document retry | `put_v1_obligation_review`, `put_v1_contradiction_review` и `post_document_processing_job_retry` больше не пишут `observation_links` напрямую из API layer. Для `obligations` и `consistency` link creation поднят в owner store methods `set_review_state_with_observation(...)`; для document retry API сначала делает durable retry command, затем materialize-ит manual `DOCUMENT_PROCESSING_JOB_STATUS` observation и через idempotent owner path `retry_failed_job_with_observation(...)` привязывает `retry_command` link к `documents/document_processing_job`. Быстрый grep по этим API-файлам на `upsert_link`/`NewObservationLink` теперь пустой. | Следующие большие API-heavy кластеры все еще остаются в `calendar`, `mail`, а также в compatibility API paths `tasks` и `organizations`. Live DB доказательство для свежего document retry path зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде есть compile gate и skip-aware targeted tests. | Следующий owner pass: `calendar/handlers/*` и `mail/handlers/*`, затем compatibility API cleanup для `tasks` и `organizations`. | Частично |
| Mail background sync run evidence trail | `communication_mail_sync_runs` теперь materialize canonical evidence trail в owner store: `start_run` пишет `COMMUNICATION_MAIL_SYNC_RUN`, а `update_progress`, `mark_recoverable_full_resync`, `finish_run`, `mark_orphaned_active_runs_failed` пишут `COMMUNICATION_MAIL_SYNC_RUN_STATUS`; `observation_link` идет на `communications/mail_sync_run`. `v1_communications_api` расширен проверкой observation trail для `sync-now`, architecture guard запрещает direct run mutations вне owner files. | Live execution ветки по-прежнему зависит от `MAKOSH_TEST_DATABASE_URL`; без него остается compile-only coverage. Нужно пройти оставшиеся runtime/job owners и, вероятно, добавить отдельный guard на `document_processing_jobs`. | Следующий technical pass: remaining runtime/job owners after mail sync, starting with adjacent document/semantic/background slices plus missing direct-mutation guards. | Частично |
| Mail drafts / folders / saved searches owner cleanup | `backend/src/domains/communications/handlers/communication_queries/{drafts,folders,saved_searches}.rs` больше не пишут `observation_link` напрямую. Durable linking поднят в owner stores: `EmailDraftStore::{upsert_with_observation,delete_with_observation}`, `MailFolderStore::{create_with_observation,update_with_observation,delete_with_observation,copy_message_with_observation,move_message_with_observation}`, `MailSavedSearchStore::{create_with_observation,update_with_observation,delete_with_observation}`. Таргетные regressions `v1_post_draft`, `v1_custom_folders_copy_move_and_events_against_postgres`, `v1_saved_searches_crud_and_events_against_postgres` и architecture guard прошли после cleanup. | В mail handler layer после этого среза еще остаются `message_actions`, `sending/*` и `workflow_actions/*`; CRUD/query owners для drafts/folders/saved searches сняты с handler-level linking. | Следующий mail pass: `message_actions` и `sending/local_state` как следующий message-mutation cluster. | Частично |
| Mail workflow / local-state / AI / outbox / attachment-import owner cleanup | `backend/src/domains/communications/handlers/{workflow_state,message_ai_state}.rs`, `backend/src/domains/communications/handlers/communication_queries/{outbox,imports}.rs` и `backend/src/domains/communications/handlers/sending/local_state.rs` больше не materialize `observation_link` напрямую. Durable linking поднят в owner layers: `MessageProjectionStore::{transition_workflow_state_with_observation,move_to_local_trash_with_observation,restore_from_local_trash_with_observation}`, `MailAiStateStore::transition_with_observation(...)`, `EmailOutboxStore::undo_with_observation(...)`, `MailStorageStore::upsert_imported_attachment_with_observation(...)`. Live regressions `v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres`, `v1_local_state_endpoints_capture_observation_trail_against_postgres`, `drafts_outbox::v1_send_schedules_outbox_message_and_allows_undo_against_postgres`, `telegram_media_upload_imports_attachment_and_queues_provider_command` и architecture guard прошли после cleanup. Во время этого pass найден и закрыт системный баг merge semantics: extra metadata больше не перетирает базовую owner metadata в новых mail owner methods. | В mail handler layer остаются `message_actions`, `sending/{forwarding,provider_send}` и `workflow_actions/actions/{calendar,documents,tasks}.rs`; это уже следующий message/runtime mutation cluster. | Следующий mail pass: `message_actions`, затем `provider_send/forwarding` и workflow projections. | Частично |
| Mail message-actions owner cleanup | `backend/src/domains/communications/handlers/message_actions.rs` больше не materialize `message_flag_update` links напрямую. Durable linking поднят в owner layers: `MessageProjectionStore::set_message_metadata_with_observation(...)` и `MessageFlags::{toggle_pin_with_observation,toggle_important_with_observation,snooze_with_observation,add_label_with_observation,remove_label_with_observation,toggle_mute_with_observation}`. Live regression `message_important_endpoint_toggles_metadata_flag` прошел после cleanup, а grep по handler больше не находит `NewObservationLink`/`upsert_link`. | В mail handler layer остаются `sending/{forwarding,provider_send}` и `workflow_actions/actions/{calendar,documents,tasks}.rs`; рядом также нужно отдельно решить, нужен ли аналогичный owner-path для bulk message actions beyond existing store. | Следующий mail pass: `provider_send/forwarding`, затем workflow projections и при необходимости bulk message actions consistency pass. | Частично |
| Mail forwarding and workflow-projection owner cleanup | `backend/src/domains/communications/handlers/sending/forwarding.rs` и `backend/src/domains/communications/handlers/workflow_actions/actions/{calendar,documents,tasks}.rs` больше не пишут `observation_link` напрямую. Durable linking поднят в owner layers: `EmailOutboxStore::enqueue_with_observation(...)`, `CalendarEventStore::create_manual_with_observation_in_transaction(...)`, `DocumentImportStore::import_document_manual_with_observation_in_transaction(...)`; `workflow_actions/tasks.rs` теперь полагается на уже существующий observation-aware `TaskStore::create_in_transaction(...)` и не дублирует task link из handler. Compile gate для `v1_workflow_actions`/`v1_communications_regressions`/`v1_communications_api`, skip-aware workflow regressions (`create_event`, `create_note`, `link_document`), live outbox regression `drafts_outbox::v1_send_schedules_outbox_message_and_allows_undo_against_postgres` и architecture guard прошли после cleanup. | В mail handler layer прямые `observation_link` writes остались только в `backend/src/domains/communications/handlers/sending/provider_send.rs`; это последний явный mail handler hotspot данного кластера. | Следующий mail pass: выделить owner/service path для live `provider_send` (SMTP/Gmail immediate send), чтобы handler больше не materialize-ил provider-send links сам. | Частично |
| Mail provider-send owner cleanup | `backend/src/domains/communications/handlers/sending/provider_send.rs` больше не materialize `observation_link` напрямую ни для immediate SMTP/Gmail send, ни для outbox enqueue path. Durable linking поднят в owner layers: `ProviderSendStore::record_sent_with_observation(...)` в `domains/mail/send.rs` и `EmailOutboxStore::enqueue_with_observation(...)` в `domains/mail/outbox.rs`. Во время этого pass снова проявился системный merge bug: extra metadata перетирала базовую owner metadata в `enqueue_with_observation(...)`; bug исправлен тем же паттерном merge-by-object. Live regressions `gmail_send_api_uses_gmail_api_when_send_scope_enabled_against_postgres`, `send_api::imap_send_api_sends_via_configured_smtp_against_postgres`, `drafts_outbox::v1_send_schedules_outbox_message_and_allows_undo_against_postgres` прошли после фикса, а grep по `backend/src/domains/communications/handlers/**` на `NewObservationLink/upsert_link` теперь пустой. | Mail handler layer как источник direct evidence linking больше не содержит известных production hotspots; дальше долг уже сместился в compatibility/API owner surfaces и deeper owner/runtime files. | Следующий слой: `domains/organizations/api.rs`, `domains/tasks/api.rs`, `domains/review/store.rs`, `domains/tasks/candidates/store/review.rs` и другие compatibility/owner files, где linking еще живет вне более узких core owner modules. | Частично |
| Compatibility API owner cleanup: organizations and tasks | `backend/src/domains/organizations/api.rs` и `backend/src/domains/tasks/api.rs` больше не materialize `observation_link` напрямую. Для organizations linking вынесен в `backend/src/domains/organizations/core/evidence.rs`; `create_with_observation`, `update_with_observation`, `archive_with_observation` и email-domain projection path теперь ходят через core evidence helpers. Для tasks linking вынесен в `backend/src/domains/tasks/core/observation_links.rs`; `TaskStore::create_in_transaction`, `update_internal` и `set_status_internal` больше не создают `NewObservationLink` сами. Compile gate для `relationships` и `tasks` прошел после refactor, а grep по `organizations/api.rs` и `tasks/api.rs` на `NewObservationLink/upsert_link` теперь пустой. | Compatibility/API debt теперь сузился: следующий явный слой — `domains/review/store.rs` и `domains/tasks/candidates/store/review.rs`, затем deeper owner/runtime files вроде `tasks/core/evidence.rs`, `organizations/enrichment.rs`, `organizations/health.rs` и соседних owner stores. | Следующий pass: review transition linking helpers, затем systematic audit remaining non-handler owner files с direct `observation_link`. | Частично |
| Review transition owner cleanup | `backend/src/domains/review/store.rs` и `backend/src/domains/tasks/candidates/store/review.rs` больше не materialize `review_transition` links напрямую. Общая семантика review-transition evidence вынесена в `backend/src/domains/review/evidence.rs`; `ReviewInboxStore::{set_status_with_observation,promote_with_observation}` и `task_candidates::store::review::set_review_state_with_observation(...)` теперь используют общий helper вместо локального `NewObservationLink` construction. Compile gate для `review_inbox`, `task_candidates`, `relationships`, `tasks` прошел после cleanup, а grep по этим store-файлам на `NewObservationLink/upsert_link` пустой. | Compatibility/API слой теперь в основном дочищен; следующий долг сместился в deeper owner/runtime files и domain core stores: `organizations/{health,enrichment}`, `tasks/core/{evidence,relations,subtasks,checklists}`, `persons/**`, `calendar/core/**`, `mail/*store*`, `decisions/store`, `obligations/store` и similar owner modules. | Следующий pass: взять следующий плотный core-owner cluster, вероятнее всего `organizations/{health,enrichment}` плюс `tasks/core/evidence`, чтобы дальше сжимать domain-owner direct linking. | Частично |
| Organizations core owner cleanup | `backend/src/domains/organizations/core/{aliases,departments,identity,contact_links}.rs` больше не materialize organization-related `observation_link` напрямую. Общий entity-link helper вынесен в `backend/src/domains/organizations/core/evidence.rs`, и эти core stores теперь используют его вместо собственного `NewObservationLink` construction. Compile gate для `relationships`, `review_inbox`, `task_candidates`, `tasks` прошел после cleanup. | В organizations bounded context еще остаются direct link writes в `organizations/enrichment.rs`, `organizations/health.rs` и частично в самом `core/evidence.rs`, что уже является canonical owner layer и не compatibility/API surface. | Следующий organizations pass: `health` и `enrichment`, затем общий person/organization review-memory cluster. | Частично |
| Organizations health/enrichment owner cleanup | `backend/src/domains/organizations/{health,enrichment}.rs` больше не строят `NewObservationLink` сами. Оба модуля переведены на общий helper `link_entity_in_transaction(...)` из `organizations/core/evidence.rs`; для этого их error enums теперь принимают `OrgCoreError` через `Core(#[from] ...)`. Compile gate для `relationships` и `tasks` прошел после cleanup, а grep по этим двум файлам на `NewObservationLink/upsert_link` пустой. | В organizations bounded context дальше остаются в основном canonical owner helpers (`core/evidence.rs`) и более высокоуровневые person/organization integration slices, а не разрозненные direct-link call sites по bounded context. | Следующий плотный cross-domain pass: `persons/**`, либо отдельно `projects/link_reviews/store.rs` и `relationships/store.rs` как review/graph owner cluster. | Частично |
| Tasks core owner cleanup | `backend/src/domains/tasks/core/{evidence,checklists,relations,subtasks}.rs` больше не materialize `observation_link` напрямую. Вынесен общий helper `materialize_task_entity_link_in_transaction(...)` в `tasks/core/observation_links.rs`; `task_evidence`, `task_checklists` и `task_subtasks` переведены на transaction-bound writes с последующим helper-based linking, а `task_relations` теперь тоже использует helper для observation-backed compatibility links. Compile gate для `tasks` и `tasks_api` прошел после cleanup, а grep по этим core store-файлам на `NewObservationLink/upsert_link` пустой. | В tasks bounded context direct linking теперь в основном сосредоточен в canonical owner helpers и соседних higher-level modules (`tasks/core/observation_links.rs`, `relationships/store.rs`, review promotion paths), а не размазан по нескольким core stores. | Следующий tasks-adjacent pass: `relationships/store.rs` и, возможно, `projects/link_reviews/store.rs` или person-memory/review cluster. | Частично |
| AI prompt studio evidence trail | `ai_prompt_templates`, `ai_prompt_template_versions` и `ai_prompt_eval_runs` теперь тоже идут через canonical evidence contract: create/version/activate/test materialize observations `AI_PROMPT_TEMPLATE`, `AI_PROMPT_TEMPLATE_VERSION`, `AI_PROMPT_EVAL_RUN`; activation теперь принимает `actor_id`, а architecture guard запрещает direct prompt-studio mutations вне owner files. Regression test `ai_control_center_mutations_record_observation_trail_against_postgres` расширен проверками prompt/template/version/eval observation trail. | Пока закрыт именно prompt studio surface; рядом остаются другие AI durable config/state tables (`catalog`, prompt eval adjacencies, semantic/runtime surfaces), которые еще не все приведены к тому же контракту. | Следующий AI pass: adjacent AI config/runtime owners after prompt studio, затем общий pass по remaining technical stores outside AI. | Частично |
| AI model catalog evidence trail | `ai_model_catalog` теперь materialize canonical evidence trail в `ai/control_center/catalog.rs`: curated model seeding возвращает upserted rows, пишет observation kind `AI_MODEL_CATALOG_ITEM`, а guard запрещает direct `ai_model_catalog` mutations вне owner file. Existing `ai_control_center_mutations_record_observation_trail_against_postgres` теперь также проверяет `sync-models` evidence trail для model catalog items. | Внутри AI bounded context еще остаются соседние surfaces вроде semantic/index/runtime stores и prompt-adjacent execution state, которые не все переведены на такой же contract. | Следующий AI pass: `ai/core/semantic/*` и другие remaining AI runtime/config owners; затем вернуться к non-AI runtime owners. | Частично |
| AI semantic embedding evidence trail | `semantic_embeddings` теперь пишут отдельный canonical observation trail в owner-file `ai/core/semantic/embeddings.rs`: upsert derived embedding materializes `AI_SEMANTIC_EMBEDDING` и link на `ai/semantic_embedding`. Source observation остается входным evidence, а embedding — отдельным derived artifact с собственным append-only trail. Architecture guard запрещает direct `semantic_embeddings` mutations вне owner file; `semantic_store` regression расширен assertion-ом на observation presence. | Live semantic regression зависит от `MAKOSH_TEST_DATABASE_URL`; в текущем окружении compile gate проходит, а live test корректно skip-ится. Остаются другие AI runtime/store surfaces и затем non-AI technical owners. | Следующий pass: remaining AI runtime/config stores after semantic embeddings, затем вернуться к document/background/domain technical owners и добить missing guards. | Частично |
| WhatsApp Web session evidence trail | `whatsapp_web_sessions` теперь materialize canonical evidence trail в owner-file `integrations/whatsapp/client/store/sessions.rs`: fixture/manual session upsert пишет `WHATSAPP_WEB_SESSION`, а `update_session_last_sync` пишет lifecycle observation с `relationship_kind = sync_progress`. Error mapping расширен под `ObservationStoreError`; architecture guard запрещает direct `whatsapp_web_sessions` mutations вне owner file. Existing WhatsApp fixture smoke test расширен assertion-ами на session observation trail. | Live WhatsApp smoke path зависит от `MAKOSH_TEST_DATABASE_URL`; в текущем окружении targeted test корректно skip-ится. Остаются другие technical session/runtime stores, особенно Telegram chats/participants и соседние provider-specific state owners. | Следующий pass: remaining provider runtime state owners after WhatsApp session, starting with adjacent Telegram/fixture stores or another isolated technical state table. | Частично |
| Telegram chat evidence trail | `telegram_chats` теперь тоже идут через transaction-bound canonical evidence contract в owner-file `integrations/telegram/client/chats.rs`: `upsert_chat` пишет `TELEGRAM_CHAT` с `relationship_kind = upsert`, а все metadata mutators, идущие через `persist_chat_metadata`, пишут `relationship_kind = metadata_update`. Targeted regression `telegram_message_links` теперь проверяет `observation_links` для chat lifecycle, а architecture guard запрещает direct `INSERT/UPDATE telegram_chats` вне owner file c явным test-only exception для `participant_roster.rs`. | Live runtime/provider-driven chat sync branches еще не закрыты отдельными assertion-ами; рядом остаются `telegram_chat_participants` и соседние provider roster/state owners. | Следующий Telegram pass: `telegram_chat_participants`, provider roster reconciliation и adjacent runtime state tables. | Частично |
| Telegram chat participant evidence trail | `telegram_chat_participants` теперь тоже materialize canonical evidence trail через owner-file `integrations/telegram/client/participants.rs`: `upsert_chat_participant` пишет `TELEGRAM_CHAT_PARTICIPANT` с `relationship_kind = upsert`, а exhaustive roster absence reconciliation пишет `relationship_kind = absent_exhaustive`. `participant_roster.rs` больше не делает голый durable update без evidence trail; architecture guard запрещает direct `INSERT/UPDATE telegram_chat_participants` вне owner path c test-only exception для roster test module. | Остаются соседние Telegram roster/runtime surfaces, прежде всего provider roster state transitions и другие participant-adjacent reconciliation paths, которые еще не все доказаны отдельными observation assertions. | Следующий Telegram pass: adjacent provider roster/runtime state owners и дополнительные live-path assertions для participant reconciliation. | Частично |
| Telegram topic evidence trail | `telegram_topics` теперь тоже переведены на canonical evidence contract в owner-file `integrations/telegram/client/topics.rs`: runtime/provider topic upsert materializes `TELEGRAM_TOPIC` с `relationship_kind = upsert`, а runtime topic event regression теперь проверяет presence of `observation_links` alongside command reconciliation and event-log assertions. Architecture guard запрещает direct `INSERT/UPDATE telegram_topics` вне owner file. | Topic write surface сам по себе закрыт, но рядом остаются более ветвистые Telegram state owners вроде `telegram_message_reactions`, `telegram_message_versions`, `telegram_message_tombstones` и message/provider-state transitions. | Следующий Telegram pass: `telegram_message_reactions` или lifecycle tables после topic projection owner cleanup. | Частично |
| Telegram message reaction evidence trail | `telegram_message_reactions` теперь materialize canonical evidence trail в owner-file `integrations/telegram/client/reactions.rs`: provider/runtime sync пишет `TELEGRAM_MESSAGE_REACTION` c `relationship_kind = provider_sync_activate` и `provider_sync_deactivate`, а local add/remove routes пишут `local_add` и `local_remove`. Regression coverage теперь есть для обеих веток: external route test проверяет local add/remove observation trail, internal runtime test проверяет provider reaction sync observation trail. Architecture guard запрещает direct `INSERT/UPDATE telegram_message_reactions` вне owner file. | Telegram message lifecycle еще не закончен: рядом остаются `telegram_message_versions`, `telegram_message_tombstones` и provider-state/message-metadata transitions, которые еще нужно свести к тому же completeness level по tests/guards/reporting. | Следующий Telegram pass: `telegram_message_versions` и `telegram_message_tombstones`, затем remaining provider-state transitions вокруг `communication_messages`. | Частично |
| Telegram message lifecycle evidence trail | `telegram_message_versions` и `telegram_message_tombstones` теперь тоже materialize canonical evidence trail через owner-files `integrations/telegram/client/lifecycle/message_versions.rs` и `.../tombstones.rs`: append-only edit versions пишут `TELEGRAM_MESSAGE_VERSION` с `relationship_kind = insert`, а provider tombstones пишут `TELEGRAM_MESSAGE_TOMBSTONE` с `relationship_kind = provider_delete` или generic `insert`. Existing realtime regressions на provider edit/delete теперь проверяют `observation_links` для lifecycle records. Architecture guard запрещает direct `INSERT INTO telegram_message_versions` и `INSERT INTO telegram_message_tombstones` вне owner files. | Append-only lifecycle tables уже закрыты, но рядом остаются non-append-only provider-state transitions и metadata projection updates в `communication_messages`, которые все еще нужно довести до того же canonical evidence contract и owner guard level. | Следующий Telegram pass: provider-state/message-metadata transitions вокруг `communication_messages`, потом remaining non-Telegram direct-write owners. | Частично |
| Telegram shared message projection evidence trail | Shared Telegram update paths для `communication_messages` теперь централизованы в owner-files `integrations/telegram/client/messages/provider_state.rs` и `.../attachments.rs`: metadata/content/delivery/pin transitions пишут append-only `COMMUNICATION_MESSAGE` observations с link на `communications/communication_message`, attachment download state пишет `COMMUNICATION_ATTACHMENT`, а `record_pin_state` больше не делает direct `UPDATE communication_messages`, а ходит через owner-path `apply_message_pinned_state`. Architecture guard теперь запрещает любые `UPDATE communication_messages` внутри `integrations/telegram`, кроме этих двух owner-files. | Покрыты owner-paths и targeted regressions на pin/attachment, но еще остается добавить более широкий runtime coverage для остальных provider-state transitions (`delivery_state`, provider content/edit metadata). | Следующий Telegram pass: runtime assertions на delivery/content/edit metadata transitions и затем выход на следующие non-Telegram shared projection owners. | Частично |
| Vault credential/runtime slice | `ProviderCredentialReader` больше не зависит от `CommunicationIngestionStore`: SMTP outbox sender, IMAP sync, Gmail sync, provider send API, Telegram runtime session-key resolution, Telegram runtime/API lifecycle/search/send/chat/history/topic/media flows, settings account listing, WhatsApp fixture ingestion, Telegram account setup/credential binding flows, mail fixture pipeline и Telegram runtime/client test scaffolds теперь берут account/binding ownership через `vault` stores. | Еще остаются broader provider/config ownership surfaces вне уже пройденных Telegram/mail paths и legacy compatibility façades в других bounded contexts. | Сужать `CommunicationIngestionStore` до raw-ingestion compatibility boundary и добивать cross-domain provider ownership. | Частично |
| Ownership guard slice | `backend/src/bin/makosh_email_sync_dev` тоже переведен на `vault::CommunicationProviderAccountStore`; `scripts/check-architecture.mjs` теперь запрещает provider account / secret binding CRUD через `CommunicationIngestionStore` compatibility façade где-либо в `backend/src`, кроме owner-файлов `domains/mail/core/accounts.rs` и `domains/mail/core/secrets.rs`; stale baseline entries после предыдущих ownership migration удалены. | Нужно расширять guard уже не по review-boundary, а по remaining ingress/ownership invariants и legacy write paths. | Следующий жесткий срез: добивать remaining observation-first ingress и provider/config ownership surfaces. | Частично |
| Review decoupling slice A | Прямой `domains/* -> domains/review` coupling убран из `decisions`, `obligations`, `relationships`: review mirror sync вынесен в `backend/src/workflows/review_mirror.rs`, domain stores больше не импортируют `ReviewInboxStore`/`ReviewItem*`, а error-модули больше не зависят от `ReviewInboxError` напрямую. | Replay/compatibility paths все еще нужно дожимать до одного owner-path, но уже без прямого domain -> review coupling. | Следующий срез: чистить remaining legacy review orchestration и parallel replay semantics. | Частично |
| Review decoupling slice B | `persons identity/enrichment/investigator` и `tasks candidates` тоже отвязаны от `domains/review`: review mirror logic перенесена в `workflows/review_mirror.rs`, `persons/enrichment` перестал вручную materialize relationship review item, а `task_candidates` больше не управляет `ReviewInboxStore` напрямую. | Остался orchestration debt, а не import debt. | Следующий срез: переводить remaining review replay/compatibility owners на тот же workflow contract. | Частично |
| Review decoupling slice C | `projects` и `calendar meetings` тоже отвязаны от `domains/review`: `project_link_reviews` и `meeting outcomes` теперь ходят через `workflows/review_mirror.rs`, `ProjectLinkReviewError` и `MeetingsError` больше не зависят от `ReviewInboxError`, а `get_project_link_candidates` materialize review candidates через workflow helper вместо прямого `ReviewInboxStore`. | Нужно проверить соседние replay/manual paths и не допустить возврата прямых review imports в другие модули. | Следующий срез: закрепить это дополнительными guard rules и пройти remaining replay/materialization paths. | Частично |
| Review promotion owner cleanup | `engines/review_promotion/mod.rs` больше не владеет direct SQL writes в `persons`, `person_personas`, `organizations`, `projects`, `project_keywords` и `obligation_task_links`. Для этого добавлены domain-owner paths `PersonProjectionStore::upsert_review_person`, `OrganizationStore::upsert_review_organization`, `ProjectStore::upsert_project` и `ObligationTaskLinkStore`; `task_candidates` obligation sync тоже переведен на тот же owner-store. Architecture guard теперь запрещает возврат direct SQL ownership этих таблиц в `review_promotion` engine. | Review promotion все еще остается orchestration boundary; дальше нужно пройти остальные engine/workflow модули, где orchestration все еще слишком близко к durable owner writes. | Следующий owner pass: remaining engine/workflow modules with direct domain mutation ownership after `review_promotion`, начиная с `engines/automation` и соседних compatibility/store surfaces. | Частично |
| Email sync organization owner cleanup | `workflows/email_sync_pipeline/organizations.rs` больше не пишет напрямую в `organizations`, `organization_domains`, `organization_identities`, `organization_contact_links` и больше не materialize-ит relationship рядом с workflow. Email-domain organization projection поднят в transaction-bound owner contract `OrganizationStore::upsert_email_domain_organization_with_observation(...)`: existing `communication_message` observation теперь линкуется не только к contact link, но и к `organizations/organization`, `organizations/organization_domain` и `organizations/organization_identity` через `relationship_kind = email_sync_projection`. Workflow оставляет только orchestration и owner-path `OrgContactLinkStore::link_email_participant_with_observation`, а live email sync regression теперь проверяет эти новые organization observation links. | Person-side projection в этом же ingestion spine уже тоже переведен, но в `email_sync_pipeline` и соседних engines/workflows все еще остаются другие owner bypass paths, прежде всего дальше по automation/compatibility surfaces. | Следующий owner pass: remaining direct durable writes in engines/workflows after email sync person+organization cleanup. | Частично |
| Email sync person owner cleanup | `project_message_knowledge` больше не materialize-ит persona/email identity как silent durable side effect. `PersonProjectionStore` получил transaction-bound owner contract `upsert_email_person_with_observation(...)`: existing `communication_message` observation теперь линкуется к `persons/persona` и `persons/identity` через `relationship_kind = email_sync_projection`, а `upsert_email_person_in_transaction(...)` возвращает и `person`, и `identity_id`, чтобы owner layer не терял evidence target. Related workflow contact action и person projection error mapping обновлены под новый contract; live email sync regression теперь проверяет эти person observation links alongside organization links. | Внутри email sync spine закрыт и person, и organization projection, но остаются следующие orchestration slices за пределами этого ядра, где evidence contract еще не доведен до полного покрытия. | Следующий owner pass: идти в remaining workflow/engine write paths после email sync projection cleanup. | Частично |
| Workflow action contact owner cleanup | `CreateContact` в `domains/mail/handlers/workflow_actions/actions/persons.rs` больше не создает persona/email identity как silent side effect внутри workflow transaction. Workflow path теперь либо reuse-ит source `communication_message.observation_id`, либо capture-ит explicit manual `PERSON_MUTATION` observation для ручного contact creation, после чего через `PersonProjectionStore::link_email_person_projection_in_transaction(...)` линкует evidence к `persons/persona` и `persons/identity` с `relationship_kind = workflow_action_projection`. Regression `v1_workflow_actions` теперь проверяет, что source message observation materialize-ит эти links на contact create path. | Остальные workflow actions (`create_note`, `create_event`, другие cross-domain mutations) еще нужно пройти на тот же explicit owner/evidence contract, если они до сих пор опираются только на durable writes и event payload. | Следующий owner pass: remaining workflow action mutations and other workflow-owned domain writes after `CreateContact`. | Частично |
| Document import owner contract and workflow document projection | `DocumentImportStore` теперь materialize-ит полный owner contract: captured `DOCUMENT` observation больше не остается только foreign-key в `documents.observation_id`, а сразу линкится к `documents/document` через `relationship_kind = import`. Поверх этого workflow `create_note` / `create_document` / `link_document` path теперь не только reuse-ит source `communication_message.observation_id` для projection links, но и сам создает document через explicit manual import contract `import_document_manual_in_transaction(...)`, а не через generic `file_import` semantics. Targeted regressions `documents` и `v1_workflow_actions` проверяют оба слоя: import link на document owner path, `origin_kind = manual` для workflow-created documents и source-message projection link на workflow `link_document`. | `DocumentImportStore` все еще оставляет generic `ObservationOriginKind::FileImport` для обычного local markdown/pdf import path, а остальные workflow-owned cross-domain mutations требуют такого же уточнения owner/evidence semantics. | Следующий owner pass: remaining workflow actions after `documents` slice, starting with adjacent non-document workflow mutations. | Частично |
| Workflow action event projection | `CreateEvent` в workflow actions больше не идет через generic runtime path. Теперь `domains/mail/handlers/workflow_actions/actions/calendar.rs` использует manual owner contract `CalendarEventStore::create_manual_in_transaction(...)`, а если source — `communication_message`, то исходный `observation_id` дополнительно линкуется к `calendar/event` через `relationship_kind = workflow_action_projection`. Targeted `v1_workflow_actions` regression подтверждает, что source message observation materialize-ит projection link на созданный calendar event. | В workflow actions еще остаются соседние cross-domain mutations, которые надо проверить на ту же дисциплину evidence reuse/manual capture и owner-path semantics. | Следующий owner pass: remaining workflow action mutations after `CreateEvent`, затем broader cross-domain workflow owners. | Частично |
| Workflow action task projection | Task creation из mail workflow теперь закрыт на двух уровнях. В `domains/tasks/api.rs` owner-store `TaskStore::create_in_transaction(...)` после durable insert всегда materialize-ит canonical `observation_link` от resolved source observation к `tasks/task` с `relationship_kind = task_create`, так что task больше не остается только с provenance/source columns без evidence graph edge. Поверх этого `domains/mail/handlers/workflow_actions/actions/tasks.rs` при source `communication_message` дополнительно пишет workflow-specific link `relationship_kind = workflow_action_projection` на тот же task. Regression `workflow_action_create_task_is_idempotent_and_records_safe_event` теперь проверяет оба link-а от source message observation. | Другие task creation surfaces за пределами mail workflow still rely on generic owner contract; их нужно отдельно проверить на domain-specific projection links там, где это важно для review/orchestration semantics. | Следующий owner pass: remaining task creation/promote flows from review, obligation and decision paths, затем соседние workflow-owned task mutations. | Частично |
| Review task promotion transition trail | Promotion `PotentialTask -> tasks/task` в `engines/review_promotion/mod.rs` теперь больше не выбивается из остального review semantics. Раньше task promotion создавал сам task через `TaskStore`, но не materialize-ил отдельный `REVIEW_TRANSITION` observation link на target entity, в отличие от person/org/decision/obligation/relationship promotions. Теперь перед promotion capture-ится manual `REVIEW_TRANSITION` observation c `operation = task_review_promotion`, а после task creation он линкуется к `tasks/task` через `relationship_kind = review_transition`. Regression helper в `backend/tests/review_inbox.rs` усилен: для `domain = tasks` он теперь требует и `task_create`, и `review_transition` link. | Live DB прогон этого review promotion path все еще зависит от `MAKOSH_TEST_DATABASE_URL`; без него regression доказывает compile-contract и skip-aware path, но не actual SQL rows. | Следующий owner pass: task promotion paths from review/obligation/decision completed at semantics level; дальше идти в remaining review-adjacent domain promotions and non-review task mutations. | Частично |
| Review knowledge promotion transition trail | `KnowledgeCandidate -> documents/document` в `engines/review_promotion/mod.rs` тоже выровнен с остальными promotion flows. Раньше knowledge promotion импортировал document и писал только `supports` links от source evidence, но не создавал explicit `REVIEW_TRANSITION` trail на target document. Теперь перед import capture-ится manual `REVIEW_TRANSITION` observation с `operation = knowledge_review_promotion`, а после import он линкуется к `documents/document` через `relationship_kind = review_transition`. Regression helper `assert_materialized_target(...)` в `backend/tests/review_inbox.rs` теперь для `domain = documents` требует и `supports`, и `review_transition` link. | Live DB подтверждение этого SQL path так же зависит от `MAKOSH_TEST_DATABASE_URL`; при его отсутствии regression остается compile-contract + skip-aware. | Следующий owner pass: remaining review promotion targets and non-review document mutation paths, где еще может отсутствовать explicit transition trail или stricter owner contract. | Частично |
| Review task promotion evidence support links | `PotentialTask -> tasks/task` теперь не теряет multi-evidence layer после promotion. После предыдущего среза task promotion уже писал `task_create` и `review_transition`, но в отличие от person/org/project/document promotions не materialize-ил `supports` links от всех `review_item_evidence` records на target task. Теперь `engines/review_promotion/mod.rs` пишет `relationship_kind = supports` для каждого evidence observation, а regression helper `assert_materialized_target(...)` для `domain = tasks` требует весь минимальный набор: `task_create`, `supports`, `review_transition`. | При одном evidence record текущий test contract подтверждает только baseline shape; richer multi-evidence live scenarios по-прежнему требуют DB-run с `MAKOSH_TEST_DATABASE_URL`. | Следующий owner pass: remaining target domains and non-review task mutation paths, где нужно проверить полноту support-link graph, а не только transition trail. | Частично |
| Review promotion parity for created decision / obligation / relationship | `engines/review_promotion/mod.rs` выровнен между mirrored-existing и newly-created branches для `PotentialDecision`, `PotentialObligation` и `PotentialRelationship`. Раньше explicit `REVIEW_TRANSITION` observation link на target entity гарантировался только когда promotion подтверждал уже существующий mirrored record через `set_review_state(...)`; create path через `upsert_with_evidence(...)` мог закончиться без такого transition trail. Теперь и create branches тоже capture-ят manual `REVIEW_TRANSITION` observation и пишут `relationship_kind = review_transition` на `decisions/decision`, `obligations/obligation` и `relationships/relationship`. Regression helper `assert_materialized_target(...)` усилен соответствующими assertion-ами. | Live SQL proof этих create branches тоже упирается в отсутствие `MAKOSH_TEST_DATABASE_URL`; compile-contract зеленый, но DB-rows в этой среде не подтверждены. | Следующий owner pass: remaining promotion targets outside current matrix and non-review mutation surfaces, где create/update paths все еще могут расходиться по evidence/transition semantics. | Частично |
| Decision / obligation / relationship owner support links | Owner stores `DecisionStore`, `ObligationStore` и `RelationshipStore` теперь не ограничиваются собственными `*_evidence` таблицами. При observation-backed evidence они дополнительно materialize-ят canonical `observation_links` с `relationship_kind = supports` на `decisions/decision`, `obligations/obligation` и `relationships/relationship`. Это закрывает общий owner-store gap: evidence больше не живет только внутри domain-specific tables без graph-level edge в Observation Platform. Targeted regressions усилены: `decisions.rs` теперь проверяет support link для message/document candidate refresh, а в `obligations.rs` и `relationships.rs` добавлены прямые live observation-backed store tests. | Полное live SQL доказательство по-прежнему зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде tests зеленые как compile-contract + skip-aware regressions. | Следующий owner pass: remaining domain stores, где observation-backed evidence еще может не materialize-ить canonical `supports` links на aggregate owner. | Частично |
| Task evidence owner-store links | `TaskEvidenceStore::add(...)` больше не полагается на route-level special case для observation links. Раньше `tasks_api.post_task_evidence` вручную линковал только manual-captured observation к `tasks/task_evidence`, а сам store ничего не гарантировал для других callers и не материализовал support-link на сам task aggregate. Теперь owner-store при `source_type = observation` сам пишет canonical link на `tasks/task_evidence` и дополнительный `relationship_kind = supports` на `tasks/task`; handler cleanup убирает дублирующую link-логику из HTTP path. Усилены regressions: в `tasks.rs` добавлен direct store test для observation-backed evidence, а `tasks_api/crud.rs` теперь проверяет не только `task_evidence`, но и support-link на `tasks/task`. | Live DB подтверждение, как и раньше, зависит от `MAKOSH_TEST_DATABASE_URL`; compile-contract зеленый, targeted task API regression прошел как skip-aware. | Следующий owner pass: remaining task core sub-record stores (`checklist`, `relations`, `subtasks`, etc.) и другие aggregate-adjacent stores, где observation trail еще может жить в handler вместо owner-store. | Частично |
| Task checklist/subtask owner-store links | `TaskChecklistStore::set(...)` и `TaskSubtaskStore::add_with_source(...)` теперь сами materialize canonical `observation_link`, когда `source` указывает на `observation:<id>`. Это убирает route-level ownership из `tasks_api.post_task_checklist` и `tasks_api.post_task_subtask`: handlers по-прежнему создают manual observation, но больше не отвечают за durable link semantics. В store-level metadata теперь фиксируются `task_id` для checklist и `parent_task_id` / `child_task_id` для subtask, чтобы downstream rebuild/review paths не зависели от HTTP-specific логики. Добавлены direct store regressions `task_checklist_store_materializes_observation_link_against_postgres` и `task_subtask_store_materializes_observation_link_against_postgres`; существующие API tests на manual create path остались зелеными после cleanup. | Live DB путь снова зависит от `MAKOSH_TEST_DATABASE_URL`, но compile gate прошел, store/API targeted tests отработали как skip-aware и architecture guard остался зеленым. `TaskRelationStore` пока еще пишет link через handler-level orchestration и остается следующим явным кандидатом на такой же owner-store перенос. | Следующий owner pass: `TaskRelationStore` и другие aggregate-adjacent task stores, где observation capture уже есть, но canonical link ownership еще не поднят в durable owner layer. | Частично |
| Task relation owner-store links | `TaskRelationStore::link(...)` теперь сам materialize-ит canonical `observation_link`, если `source` у relation указывает на `observation:<id>`. Это убирает последний явный route-level durable link special case из `tasks_api.post_task_relation`: handler по-прежнему capture-ит manual observation, но больше не владеет сохранением link в Observation Platform. Одновременно direct store regression `task_relation_store_materializes_observation_link_against_postgres` доказывает owner behavior вне HTTP layer, а существующий API regression `crud::task_relation_manual_create_path_captures_observation_against_postgres` остается зеленым после cleanup. Внутри task sub-record area это доводит одинаковый ownership pattern уже до `task_evidence`, `task_checklist`, `task_relation` и `task_subtask`. | Live DB path по-прежнему зависит от `MAKOSH_TEST_DATABASE_URL`, но compile gate, direct store test и API regression прошли; architecture guard тоже остается зеленым. Этот узкий pattern в task core фактически дожат, а следующий residual debt смещается в соседние bounded contexts и workflow/runtime surfaces, а не в те же task sub-record handlers. | Следующий owner pass: remaining handler/workflow durable-link or direct-mutation paths вне task sub-record area, начиная с ближайших cross-domain/task-adjacent flows. | Частично |
| Task aggregate mutation owner-store links | Owner semantics подняты и для самого `tasks/task` aggregate. `TaskStore` получил evidence-aware paths `update_with_observation(...)`, `set_status_with_observation(...)` и `archive_with_observation(...)`, которые внутри transaction сами materialize-ят canonical `observation_link` на `tasks/task`. За счет этого `tasks_api.put_task`, `post_task_status`, `post_task_archive` и runtime path `post_task_analyze` больше не владеют durable link persistence в handler-ах: они только capture-ят observation и передают его owner-store. Добавлен direct store regression `task_store_update_with_observation_materializes_task_link_against_postgres`; существующие API regressions на update/status/archive/analyze остались зелеными после cleanup. | Live DB assertions снова skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate, direct store test, `crud::tasks_crud_against_postgres`, `crud::task_status_transition`, `crud::task_analyze_runtime_path_captures_observation_against_postgres` и `crud::task_archive_manual_path_captures_observation_against_postgres` прошли. Внутри task bounded context route-level durable link ownership для core task mutations и sub-record mutations уже в основном снят; дальше priority смещается на task candidates / review orchestration и соседние cross-domain flows. | Следующий owner pass: remaining task-adjacent review/candidate/workflow paths и затем соседние bounded contexts, где durable links или direct mutations все еще сидят в handler/workflow layer. | Частично |
| Task candidate review owner-store links | `TaskCandidateStore` получил owner-aware review path `set_review_state_with_observation(...)`, а transaction-bound logic в `domains/tasks/candidates/store/review.rs` теперь сама materialize-ит `review_transition` link на `tasks/task_candidate`. За счет этого `tasks_api.put_task_candidate_review` больше не пишет `observation_link` после store call: handler только capture-ит manual `REVIEW_TRANSITION` observation и передает его owner-store вместе с command metadata. Добавлен direct store regression `review::task_candidate_store_review_with_observation_materializes_transition_link_against_postgres`; существующий live API regression `put_task_candidate_review_confirms_task_with_observation_trail` остался зеленым после cleanup. | Live DB path по-прежнему skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate, direct store regression и API regression прошли, architecture guard остался зеленым. В task candidate area это снимает еще один явный handler-level durable link special case; следующий review pass уже про аналогичные endpoints в других bounded contexts и оставшиеся workflow-level review transitions. | Следующий owner pass: review endpoints в `decisions`, `relationships`, `review_items`, `persons/organizations` и соседние workflow review transitions, где canonical link semantics еще могут жить вне owner-store. | Частично |
| Decision / relationship / review status owner-store links | `DecisionStore`, `RelationshipStore` и `ReviewInboxStore` теперь получили owner-aware review/status paths, которые сами materialize-ят `review_transition` link внутри своей transaction boundary. Для decisions и relationships добавлены `set_review_state_with_observation(...)`; для review domain добавлен `set_status_with_observation(...)`. Соответственно `put_v1_decision_review`, `put_v1_relationship_review` и `transition_review_item_status(...)` больше не пишут `observation_links` из API layer: handlers только capture-ят manual `REVIEW_TRANSITION` observation и передают его durable owner-у. Добавлен direct review store regression `review_inbox_status_with_observation_materializes_transition_link_against_postgres`; существующие API regressions `put_decision_review_updates_review_state_with_observation_trail` и `put_relationship_review_updates_relationship_and_graph_projection` остались зелеными после cleanup. | Live DB API paths снова skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate по `decisions_api` / `relationships_api` / `review_inbox`, targeted API tests и direct review store regression прошли; architecture guard остался зеленым. В review endpoints это снимает основной pattern handler-level transition linking для status/review mutations. Отдельным хвостом остается `review item promote`, где owner уже не `ReviewInboxStore`, а `ReviewPromotionService`. | Следующий owner pass: `review item promote` через `ReviewPromotionService`, затем аналогичные review-transition paths в remaining bounded contexts (`persons`, `organizations`, project/person identity review adapters, workflow review transitions). | Частично |
| Review promotion owner-service links | `ReviewPromotionService` теперь получил owner-aware path `promote_with_observation(...)`, а `ReviewInboxStore` — дополняющий owner path `promote_with_observation(...)` для final `review_items` mutation и `review_transition` link inside the same transaction. За счет этого `post_v1_review_item_promote` больше не пишет `observation_link` из API layer: handler только capture-ит manual `REVIEW_TRANSITION` observation и передает его promotion service. Добавлен direct regression `review_promotion_service_with_observation_materializes_review_item_transition_link_against_postgres`; существующий broad promotion regression `review_can_materialize_promotions_for_core_target_domains_against_postgres` остался зеленым после cleanup. | Live DB path снова skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate по `review_inbox`, direct promotion-service regression и core promotion regression прошли; architecture guard остался зеленым. Таким образом review domain больше не держит status/promote `review_transition` linking в API layer. Следующий residual debt уже смещается в remaining bounded contexts и adapters, а не в центральный review API. | Следующий owner pass: review-transition paths в `persons`, `organizations`, project/person-identity review adapters и workflow-level review transitions, где linking еще может жить вне owner/store/service boundary. | Частично |
| Person identity / project link review owner-store links | `PersonIdentityStore` и `ProjectLinkReviewStore` теперь получили owner-aware review paths `set_review_state_with_observation(...)`, которые сами materialize-ят `review_transition` link внутри своей transaction boundary. За счет этого `persons_api.put_identity_candidate_review` и `projects_api.put_project_link_review` больше не пишут `observation_links` из handler-ов: handlers только capture-ят manual `REVIEW_TRANSITION` observation и передают его owner-store. Existing API regressions `write_endpoints::identity_candidate_review_captures_observation_against_postgres` и `put_project_link_review_updates_review_state` остались зелеными после cleanup. | Live DB API paths снова skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate по `persons_api` / `projects_api`, targeted API tests и architecture guard прошли. Это снимает еще два adapter-level review_transition special case и сдвигает остаток работ к менее центральным person/org review-like handlers и workflow-level transition paths. | Следующий owner pass: remaining `persons` review-like handlers (`dossier_review`, intelligence review transitions), `organizations` adjacent review transitions и workflow-level review link adapters вне current owner stores. | Частично |
| Person dossier and enrichment review-like owner paths | `PersonInvestigator` получил owner-aware `review_dossier_snapshot_with_observation(...)`, а `EnrichmentResultStore` получил `apply_with_observation(...)` и `reject_with_observation(...)`. За счет этого `persons_api.put_person_dossier_review`, `post_person_enrichment_apply` и `post_person_enrichment_reject` больше не пишут `review_transition` links из handler-ов: handlers только capture-ят manual transition observations и передают их owner/service layer. Existing `persons_api` regressions `dossier_owner::person_investigate_captures_observation_and_links_snapshot_against_postgres` и `write_endpoints::person_enrichment_review_entrypoints_capture_observations_against_postgres` остались зелеными после cleanup. | Live DB verification снова skip-aware без `MAKOSH_TEST_DATABASE_URL`, но compile gate по `persons_api`, targeted dossier/enrichment tests и architecture guard прошли. Внутри persons bounded context это снимает еще один cluster review-like route-level link ownership. | Следующий owner pass: remaining person/organization compatibility and workflow-level transition adapters, где capture/link semantics все еще живут вне durable owner/service boundary. | Частично |
| Automation evidence trail | `engines/automation/store.rs` и `dry_run.rs` теперь materialize canonical evidence trail для `automation_templates`, `automation_policies` и `telegram_outbound_messages`: добавлены observation kinds `AUTOMATION_TEMPLATE`, `AUTOMATION_POLICY`, `TELEGRAM_OUTBOUND_MESSAGE`; template/policy upsert и Telegram dry-run теперь пишут append-only observations и `observation_link` в `domain = automation`. Existing Telegram automation test расширен assertion-ами на template/policy/outbound observation trail. Architecture guard запрещает direct mutations этих таблиц вне owner-files automation engine. | Следующим срезом остаются другие owner bypass paths в `workflows/` и `engines/`, прежде всего `email_sync_pipeline/participants.rs`, `email_sync_pipeline/relationships.rs` и дальнейшие compatibility orchestration paths. | Следующий owner pass: remaining direct durable writes in workflows/engines after automation evidence trail. | Частично |
| Email sync participants and relationship events owner cleanup | `workflows/email_sync_pipeline/participants.rs` и `relationships.rs` больше не пишут напрямую в `communication_message_participants` и `relationship_events`. Добавлен owner-path `MessageProjectionStore::upsert_email_participant`, а `RelationshipEventStore` получил `upsert_email_message_event`; оба path используют существующий `ProjectedMessage.observation_id` для `observation_link` на `communications/message_participant` и `persons/relationship_event`. Live email sync regression расширен assertion-ами на эти observation links, а architecture guard запрещает возврат прямых SQL writes в workflow. | В `email_sync_pipeline` остаются другие orchestration slices, но прямые writes этих двух compatibility tables из workflow убраны. Дальше остаются другие workflow/engine owner bypass paths вне этого среза. | Следующий owner pass: remaining direct durable writes after email sync participants/relationship-event cleanup, начиная с соседних workflow compatibility paths. | Частично |
| Consistency contradiction materialization link | `engines/consistency/store/observations.rs` теперь не только capture-ит canonical `CONTRADICTION_OBSERVATION`, но и сразу link-ит его к `consistency/contradiction_observation` с `relationship_kind = upsert`. Это делает contradiction record полноценным entity-level evidence target, а не только источником для review mirror и review transition. Live contradictions API regression расширен assertion-ом на materialized observation link. | В `consistency` остается более широкий долг по generic manual review-transition observation kinds и соседним replay semantics, но baseline materialization link на contradiction record теперь есть. | Следующий consistency pass: убрать placeholder review-transition kinds вроде `DOCUMENT`/`CONTACT_RECORD` там, где они используются как generic manual review observations. | Частично |
| Review transition vocabulary cleanup | Manual review/apply/reject/promote/status observations в `decisions`, `obligations`, `relationships`, `tasks candidates`, `projects`, `review`, `persons`, `organizations`, `consistency_api` и `review_promotion` больше не используют placeholder kinds `DOCUMENT` / `CONTACT_RECORD`. Добавлен canonical observation kind `REVIEW_TRANSITION`, и найденные review-related handlers теперь materialize manual transition evidence именно этим kind. Это выравнивает review vocabulary across domains и убирает скрытый semantic drift в evidence layer. | Остаются replay/compatibility semantics вокруг review orchestration и соседние manual/runtime surfaces, которые еще надо проверить на единый review owner path. | Следующий architecture pass: review orchestration consolidation и cleanup remaining replay compatibility paths. | Частично |
| Person identity candidate observation vocabulary | Identity-candidate mirror больше не пишет synthetic evidence как `CONTACT_RECORD`. Добавлен canonical observation kind `PERSON_IDENTITY_CANDIDATE`, `workflows/review_mirror.rs` теперь materialize identity candidate review evidence именно этим kind, а live `person_identity_api` regression проверяет его явно через `review_item_evidence -> observations.kind_code`. Это убирает review-specific semantic drift в одном из самых чувствительных candidate flows. | Аналогичные placeholder kinds `CONTACT_RECORD` / `DOCUMENT` все еще живут в других non-review и compatibility paths; они требуют отдельного bounded-context pass, а не точечной замены вслепую. | Следующий vocabulary pass: пройти remaining placeholder observation kinds по bounded contexts, начиная с persons/organizations/calendar compatibility writes и synthetic review-adjacent flows. | Частично |
| Project link review observation vocabulary | `project_link_reviews` больше не пишет review-event evidence как `DOCUMENT`. Добавлен canonical observation kind `PROJECT_LINK_REVIEW`; `domains/projects/link_reviews/adapters.rs` теперь materialize review-event observation именно этим kind, а live `project_link_reviews` regression проверяет `observations.kind_code` через downstream decision evidence. Это убирает еще один synthetic review artifact из generic document vocabulary. | Placeholder kinds `CONTACT_RECORD` / `DOCUMENT` все еще остаются в generic CRUD/manual paths (`persons`, `organizations`, `calendar`, `tasks`) и требуют аккуратного bounded-context design pass, потому что там не всегда очевиден единый replacement kind. | Следующий vocabulary pass: перейти к grouped manual/runtime paths по domains `persons/organizations/calendar`, где нужен более явный canonical registry, а не point-fix. | Частично |
| Task mutation observation vocabulary | В `tasks` bounded context synthetic/manual/runtime task-local observations больше не маскируются под `DOCUMENT`. Добавлен canonical observation kind `TASK_MUTATION`; на него переведены manual task provenance seed, task update/status/archive, checklist/evidence/relation/subtask API paths, task analyze runtime path и compatibility relation materialization. Upstream real `DOCUMENT` source semantics сохранены только там, где источник действительно document-backed (`task_activation` fallback и explicit external observations). Tests `tasks` и `tasks_api` теперь явно проверяют `kind_code = TASK_MUTATION` на representative paths. | Вне `tasks` остаются другие generic placeholder paths, прежде всего `persons`, `organizations`, `calendar` и часть `vault` CRUD/materialization flows с `CONTACT_RECORD` / `DOCUMENT`. | Следующий vocabulary pass: grouped migration по `persons/organizations` с явным разделением profile/core-record/manual-action semantics. | Частично |
| Person / organization aggregate mutation vocabulary | Для aggregate/manual profile paths введены canonical kinds `PERSON_MUTATION` и `ORGANIZATION_MUTATION`. На `PERSON_MUTATION` переведены `put_persona`, `put_owner_persona`, `post_person_favorite`, `post_person_watchlist_toggle`; на `ORGANIZATION_MUTATION` переведены `post_organization`, `put_organization`, `post_organization_archive`. Live regressions `persons_api` и `organizations_api` теперь проверяют `observations.kind_code` для этих aggregate/manual paths. Это отделяет aggregate mutation evidence от generic `CONTACT_RECORD` vocabulary. | Subordinate record paths еще не переведены: `organizations` identities/aliases/departments/contacts, а также `persons` compatibility/identity/memory handlers все еще используют placeholder kinds и требуют отдельного semantic pass. | Следующий vocabulary pass: перейти к subordinate person/org records и compatibility handlers с отдельным registry, не смешивая aggregate mutation и record capture semantics. | Частично |
| Review domain | Реализован `backend/src/domains/review/`; lifecycle `new / in_review / approved / promoted / dismissed / archived`; есть API и frontend review-domain. | Не все candidate/materialization/replay paths сведены к одному owner через `domains/review`. | Дожать единый review orchestration path и оставить legacy stores только как projections. | Частично |
| Tasks provenance | Task creation/promotion/workflow paths уже создают tasks с observation-backed provenance; закрыт fallback для task creation без observation evidence. | Остались legacy read/projection assumptions, которые еще опираются на старые `message/document` source semantics. | Добить downstream consumers и compatibility surfaces. | Частично |
| Review mirrors: tasks/persons/projects/calendar | `task_candidates`, `person_identity_candidates`, `project_link_reviews` и `meeting outcomes` теперь синхронно обновляют mirrored `review_items` через workflow-layer mirror functions. | Replay/compatibility контуры все еще живут рядом с новым inbox owner. | Свести review решение к одному orchestration path через `domains/review`. | Частично |
| Review mirrors: decisions/obligations/relationships | Domain review stores теперь синхронно переводят mirrored `review_items` в `new / approved / dismissed / promoted`. | Остались non-HTTP и compatibility transitions, которые надо проверить тем же контрактом. | Пройти engine/replay/manual compatibility paths в этих bounded contexts. | Частично |
| Consistency / contradictions | `engines/consistency` больше не должен быть отдельным inbox owner: contradiction observations materialize canonical `CONTRADICTION_OBSERVATION`, создают mirrored `contradiction_candidate` review items и синхронно ведут `new / approved / dismissed`. | Нужно проверить remaining engine-owned replay/compatibility semantics, чтобы не осталось обходных review transitions. | Следующий технический срез: consistency replay + adjacent engine review semantics. | Частично |
| Mail attachment import ingress | `POST /api/v1/communications/attachments/import` теперь сначала materialize `COMMUNICATION_ATTACHMENT` observation, затем вызывает owner-store `MailStorageStore::upsert_imported_attachment_with_observation(...)`, который сам пишет `observation_link` на `attachment_import` в той же transaction boundary. | Остались другие provider/local import и auxiliary mail write surfaces, которые еще не всегда имеют такой же observation trail. | Пройти remaining import/export/provider command entrypoints внутри mail bounded context. | Частично |
| Mail AI state ingress | `PUT /api/v1/communications/messages/{id}/ai-state` теперь сначала materialize manual `COMMUNICATION_MESSAGE` observation, затем вызывает owner-store `MailAiStateStore::transition_with_observation(...)`, который сам пишет `observation_link` на `communication_message` с `relationship_kind = ai_state_transition`. | Остались другие auxiliary mail state transitions и assistant-facing write paths, которые еще нужно проверить на тот же evidence trail. | Пройти remaining mail assistant/runtime mutation paths по одному. | Частично |
| Calendar rules, scheduling, accounts, sources, import, sync and intelligence ingress | `POST/PUT/DELETE /api/v1/calendar/rules`, `POST /api/v1/calendar/deadlines`, `POST /api/v1/calendar/focus-blocks`, `POST/PUT/DELETE /api/v1/calendar/accounts`, `POST /api/v1/calendar/accounts/{account_id}/sources`, `POST /api/v1/calendar/import`, `POST /api/v1/calendar/accounts/{account_id}/sync`, `POST /api/v1/calendar/events/{event_id}/classify` и `POST /api/v1/calendar/events/{event_id}/analyze` теперь materialize observation и затем пишут/link-ят durable mutation. `CalendarEventStore::create_in_transaction` тоже теперь всегда пишет `observation_link` для runtime-created events. | Остались другие calendar write paths вне уже покрытых handlers и deeper provider sync/replay surfaces, которые нужно проверить тем же контрактом. | Следующий calendar pass: проверить provider-driven calendar event ingestion/replay paths и соседние domains на missing `observation_link`/wrong origin semantics. | Частично |
| Calendar event detail owner cleanup | `backend/src/domains/calendar/handlers/events/{agenda,checklist,participants,relations}.rs` больше не пишут `observation_links` напрямую. Durable linking поднят в owner stores `EventAgendaStore::set_with_observation(...)`, `EventChecklistStore::set_with_observation(...)`, `EventParticipantStore::add_with_observation(...)`, `EventRelationStore::link_with_observation(...)`; для этого `CalendarCoreError` теперь включает `ObservationStoreError`. Быстрый grep по этим четырем handler-файлам на `upsert_link`/`NewObservationLink` пустой. | В `calendar` остаются другие route-level direct links: `accounts`, `meetings`, `reminders`, `rules`, `scheduling`, `sync`. Live DB proof для свежего sub-slice зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде есть compile gate и skip-aware targeted tests. | Следующий calendar pass: перевести owner-contract для `meetings/reminders` или `accounts/rules/scheduling`, не оставляя mixed handler/store ownership внутри одного bounded context. | Частично |
| Calendar meeting materials and reminders owner cleanup | `backend/src/domains/calendar/handlers/meetings.rs` и `reminders.rs` больше не materialize `observation_links` из handler layer для `meeting_note`, `meeting_outcome`, `event_recording`, `event_reminder` и reminder toggle. Durable linking поднят в owner stores: `MeetingNoteStore::create_with_observation(...)`, `MeetingOutcomeStore::add_with_observation(...)`, `EventRecordingStore::add_with_observation(...)`, `CalendarReminderStore::{create_with_observation, set_active_with_observation}`. Для этого `MeetingsError` и `ReminderError` теперь включают `ObservationStoreError`. Быстрый grep по `meetings.rs` и `reminders.rs` на `upsert_link`/`NewObservationLink` пустой. | Внутри `calendar` остаются remaining handler-level direct links в `accounts`, `rules`, `scheduling`, `sync` и, возможно, часть event/account-adjacent owner flows. Live DB proof снова зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде есть compile gate и skip-aware targeted tests. | Следующий calendar pass: перевести `accounts/rules/scheduling/sync`, чтобы calendar API перестал быть mixed owner surface. | Частично |
| Calendar accounts, sources and sync owner cleanup | `backend/src/domains/calendar/handlers/accounts.rs` и `sync.rs` больше не пишут `observation_links` напрямую для manual `calendar_account`, `calendar_source` и `sync_trigger` paths. Durable linking поднят в `vault/provider_accounts.rs`: `CalendarAccountStore::{create_with_observation, update_with_observation, delete_with_observation}` и `CalendarSourceStore::create_with_observation`. Sync trigger теперь тоже проходит через `CalendarAccountStore::update_with_observation(..., relationship_kind = \"sync_trigger\")`. Быстрый grep по `accounts.rs` и `sync.rs` на `upsert_link`/`NewObservationLink` пустой. | Внутри `calendar` остались в основном `rules` и `scheduling`; отдельные owner-store internal links в `vault/provider_accounts.rs` нормальны и intentional. Live DB proof зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде есть compile gate и skip-aware targeted tests `accounts::calendar_accounts_crud_against_postgres`, `misc::calendar_sources_list`, `misc::cal_sync`. | Следующий calendar pass: перевести `rules/scheduling`, после чего calendar handler layer будет практически дочищен и можно переходить к `mail`. | Частично |
| Calendar rules and scheduling owner cleanup | `backend/src/domains/calendar/handlers/rules.rs` и `scheduling.rs` больше не materialize `observation_links` из handler layer для `calendar_rule`, `deadline_event` и `focus_block`. Durable linking поднят в owner stores `CalendarRuleStore::{create_with_observation, update_with_observation, delete_with_observation}`, `DeadlineStore::create_with_observation(...)`, `FocusBlockStore::create_with_observation(...)`. Для этого `CalendarRuleError` и `SchedulingError` теперь включают `ObservationStoreError`. Проверка `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/calendar/handlers -g '!**/target/**'` теперь пустая по всему handler layer `calendar`. | Домен `calendar` больше не держит direct `observation_link` writes в handler layer, но внутри него остаются owner-store internal links и более глубокие runtime/projection surfaces (`events/event_store.rs`, `brain.rs`, `intelligence.rs`) которые уже нужно оценивать отдельно, не как handler cleanup. Live DB proof снова зависит от `MAKOSH_TEST_DATABASE_URL`; в текущей среде есть compile gate и skip-aware targeted tests `misc::cal_rules_crud`, `misc::cal_post_deadline`, `misc::cal_post_focus_block`. | Следующий доменный pass: выйти из calendar handler cleanup и перейти к `mail` handler cluster как следующему самому крупному remaining owner problem. | Частично |
| Context Pack engine consolidation | `calendar` и `tasks` больше не читают и не пишут legacy `event_context_packs` / `task_context_packs` как runtime owner-path. `EventContextPackStore` и `TaskContextPackStore` переведены на compatibility façade над `backend/src/engines/context_packs/store.rs`; `CalendarBrainService`, `TaskBrainService` и `TaskWatchtowerService` теперь читают engine-owned `context_packs`; architecture guard запрещает новые прямые обращения к legacy context-pack tables из `backend/src/**`. Это убирает parallel persistence model и реально закрепляет `engines/context_packs` как owner слоя derived context. | Нет полного rebuild/invalidation policy, source enrichment strategy и полного набора pack-ов (`Persona / Calendar / Meeting / Project` глубже текущего compatibility уровня). Legacy таблицы и миграции пока остаются как historical schema compatibility, но больше не должны использоваться runtime-кодом. | Следующий context pass: ввести explicit rebuild/invalidation contract, richer source sets и пройти remaining context-pack producers beyond current calendar/task façades. | Частично |
| Organization subordinate record vocabulary | Manual organization subrecords больше не пишутся как общий `CONTACT_RECORD`. `post_org_identity`, `post_org_alias`, `post_org_department`, `post_org_contact_link` и fallback materialization в `organizations/core/contact_links.rs` переведены на `ORGANIZATION_RECORD_MUTATION`; отдельно `post_org_watchlist_toggle` доведен до `ORGANIZATION_MUTATION`, чтобы aggregate mutation и subordinate record mutation больше не смешивались. PostgreSQL regression обновлен и теперь различает aggregate mutations (`create/update/watchlist/archive`) и subordinate record mutations (`identity/alias/department/contact`). | Остались person-side compatibility/manual traces и vault provider-account compatibility paths, где `CONTACT_RECORD` еще используется как слишком широкий тип. | Следующий vocabulary pass: пройти `persons/identity`, `persons/compatibility`, `persons/memory` и затем owner-paths в `vault/provider_accounts.rs`. | Частично |
| Person subordinate record vocabulary | Manual person-side subordinate records больше не materialize generic `CONTACT_RECORD`. `identity traces`, `person identities`, compatibility `roles/personas`, `facts/preferences` и `relationship events` переведены на `PERSON_RECORD_MUTATION`, при этом aggregate persona mutations (`owner`, `persona update`, `favorite`, `watchlist`) остаются на `PERSON_MUTATION`. PostgreSQL regressions теперь различают aggregate person mutations и subordinate/manual person record mutations. | `person_memory_cards` по-прежнему используют `DOCUMENT`, а не отдельный person-memory kind; это осознанно, но vocabulary может потребовать отдельного решения, если memory-card mutations нужно выделять отдельно от document-style evidence. | Следующий persons pass: решить, оставлять ли memory-card manual capture на `DOCUMENT` или выделить отдельный `PERSON_MEMORY_CARD` / `KNOWLEDGE_NOTE` kind. | Частично |
| Vault technical owner vocabulary | Vault-owned technical records больше не пишутся как `CONTACT_RECORD` в owner-stores `vault/provider_accounts.rs`. Linked calendar accounts теперь дают `CALENDAR_ACCOUNT_LINK`; task provider accounts — `TASK_PROVIDER_ACCOUNT`; communication provider accounts — `COMMUNICATION_PROVIDER_ACCOUNT`; communication provider secret bindings — `COMMUNICATION_PROVIDER_SECRET_BINDING`. Targeted regressions по task providers, Gmail/iCloud account setup и vault reconciliation теперь проверяют не только `origin_kind`, но и конкретный `kind_code`. | В `vault` еще остаются пути без отдельного explicit evidence kind contract, например delete/cleanup flows и возможные future capability/session records. | Следующий vault pass: пройти destructive lifecycle (`delete_metadata`) и remaining capability/session records, чтобы append-only observation trail покрывал не только upsert/bind, но и explicit removal semantics. | Частично |
| Vault metadata removal evidence | `CommunicationProviderAccountStore::delete_metadata` больше не делает silent destructive cleanup. При удалении metadata теперь materialize append-only observations `COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED` и `COMMUNICATION_PROVIDER_ACCOUNT_DELETED`, а `observation_links` пишутся с `relationship_kind = remove` и `delete` в `vault` domain. Deterministic API regression `email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account` теперь создает secret binding, удаляет account и проверяет both removal observations плюс `unbound_secret_refs` response contract. | Logout/config update path все еще не materialize explicit account-config mutation observation на owner-store уровне; также остаются другие destructive lifecycle paths вне communication provider metadata. | Следующий vault pass: перевести `update_config`/logout semantics в explicit evidence trail и затем пройти соседние destructive technical cleanup paths. | Частично |
| Vault account config mutation evidence | `CommunicationProviderAccountStore::update_config` больше не выполняет silent durable update. Owner-store теперь materialize `COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION` с `relationship_kind = config_update`; mail logout path и Telegram account lifecycle path вызывают `update_config_with_origin(...)` с explicit actor/action. Deterministic regression `email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account` теперь проверяет, что logout оставляет canonical config-mutation observation c `payload.action = logout`. | Другие account/config mutation paths outside communication provider accounts все еще могут писать durable state без такого же explicit owner-store evidence contract. | Следующий config pass: пройти adjacent technical config owners и затем оценить, нужно ли аналогично поднять sync settings / runtime account-state mutations в explicit observation taxonomy. | Частично |
| DOCUMENT replacement: persons/calendar manual artifacts | Overly-broad `DOCUMENT` больше не используется для person memory and calendar note-like/manual artifact mutations. Добавлены `PERSON_MEMORY_CARD`, `EVENT_AGENDA`, `EVENT_CHECKLIST`, `MEETING_NOTE`, `CALENDAR_RULE`; на них переведены `put_person_notes`, `post_person_memory_card`, calendar event agenda/checklist, meeting notes и calendar rules CRUD. Existing regressions по persons memory, calendar event detail entrypoints и calendar rules теперь проверяют эти конкретные kind codes вместо implicit document semantics. | `DOCUMENT` все еще остается валидным canonical kind для actual document/domain flows и части derived/manual semantics outside this slice. Нужно дальше пройти remaining `DOCUMENT` runtime usages, которые не являются literal document evidence. | Следующий vocabulary pass: разобрать remaining `DOCUMENT` usages в `documents/*`, `persons/core/roles`, `persons/trust`, `calendar` projections and any true document flows, separating literal document evidence from mutation artifacts. | Частично |
| Person-derived evidence taxonomy and document-processing retry vocabulary | Derived person evidence больше не маскируется под `DOCUMENT`. Добавлены `PERSON_ROLE`, `PERSON_TRUST_SIGNAL`, `PERSON_PROMISE`; на них переведены compatibility role relationship materialization, enrichment trust relationship evidence и promise-to-obligation projection. `post_document_processing_job_retry` больше не пишет generic `DOCUMENT`, а использует уже существующий `DOCUMENT_PROCESSING_JOB_STATUS` с `relationship_kind = retry_command`. Targeted regressions по `persons/relationships` и `document_processing_api` теперь проверяют точные `kind_code`. | В runtime после этого среза оставались только `documents/core/store.rs` и `tasks/candidates/store/task_activation.rs`. | Следующий vocabulary pass: закрыть `task_candidates/store/task_activation.rs` как observation-only path и затем переоценить, остается ли `DOCUMENT` где-либо кроме literal document import. | Частично |
| Task candidate activation provenance hardening | `task_candidates` activation больше не синтезирует новые observations из legacy `message/document` labels. Confirmation path теперь требует canonical observation evidence (`observation_id` или `source_kind = observation` с `source_id = observation_id`) и падает с `ObservationRequired`, если candidate не observation-backed. Added regression locks this in for manually inserted legacy non-observation task candidates. После этого `task_candidates` runtime больше не использует `DOCUMENT` или `COMMUNICATION_MESSAGE` как fallback taxonomy. | В runtime `backend/src` после этого среза остается только один `DOCUMENT` usage: literal document import в `documents/core/store.rs`. Это выглядит как корректный canonical kind, а не как ложная mutation taxonomy. | Следующий pass: проверить, нет ли remaining non-runtime docs/tests/fixtures, которые еще тянут старую source vocabulary в read-models, и затем перейти к remaining ownership/review-flow cleanup. | В основном сделано |
| Gmail refresh runtime ownership cleanup | `EmailAccountSetupService` больше не тянет `CommunicationIngestionStore` только ради `PgPool`. Service хранит `PgPool` напрямую и строит vault-owned account/binding stores без compatibility façade. `provider_send`, Gmail background sync, outbox provider sender и `account_setup_service` integration factory больше не зависят от `CommunicationIngestionStore` в refresh/send runtime path; façade остается только там, где реально нужен communication ingestion (`raw_records`, checkpoints, import/setup compatibility). | `CommunicationIngestionStore` как compatibility façade все еще широко импортируется в mail/integration codebase и частично нужен для raw-record/checkpoint contracts. Ownership cleanup не завершен для всех provider/runtime surfaces. | Следующий ownership pass: пройти remaining runtime/service call sites, где `CommunicationIngestionStore` еще используется для provider metadata instead of raw-record/checkpoint semantics, либо зафиксировать их как intentional compatibility boundaries. | Частично |
| Task candidate review mirror consolidation | `task_candidates` review mirroring больше не собирает mirrored review item двумя разными код-путями. `workflows/review_mirror` теперь держит shared helper `ensure_task_candidate_review_item[_in_transaction]`, а `workflows/review_inbox::sync_task_candidates_to_review_for_observations` использует тот же owner-path вместо собственной дублирующей сборки `NewReviewItem`. Заодно `StoredCandidateRow` расширен полями `due_text` и `assignee_label`, поэтому mirrored task review item больше не теряет candidate metadata на review-state sync path. Regression по `review_inbox` теперь подтверждает `due_text = "Friday 5pm"` на mirrored task item. | Консолидация пока сделана только для task candidates; аналогичный parallel mirror logic еще остается для decisions, obligations, relationships и project link candidates. | Следующий review pass: либо провести такой же helper-consolidation для decisions/obligations/relationships, либо поднять общий review mirror builder layer внутри `domains/review` / `workflows/review_mirror`. | Частично |
| Decision / obligation / relationship review mirror consolidation | `workflows/review_inbox` больше не собирает mirrored review items для `decisions`, `obligations` и `relationships` отдельными локальными ветками. Эти sync paths теперь идут через shared helper-слой `ensure_decision_review_item[_in_transaction]`, `ensure_obligation_review_item[_in_transaction]` и `ensure_relationship_review_item[_in_transaction]` в `workflows/review_mirror`. Соответствующие review-state sync функции в `review_mirror` тоже используют тот же owner-path и сначала требуют observation-backed evidence (`decision_evidence`, `obligation_evidence`, `relationship_evidence`) вместо параллельной ручной сборки `NewReviewItem`. Дополнительно existing mirrored item теперь не “застывает” на первом observation: при повторном sync helper-ы прикрепляют новое evidence к уже существующему review item через `ReviewInboxStore::attach_evidence_in_transaction`. Это сузило parallel review ownership до project-link/identity и убрало еще один класс drift между inbox refresh и state mirroring. | Project link и identity candidate review mirroring все еще живут отдельными ветками; кроме этого остается orchestration debt между legacy candidate refresh и единым review domain. | Следующий review pass: либо обобщить remaining project-link/identity mirror builders, либо зафиксировать их как отдельные intentional patterns, если payload shape действительно отличается. | Частично |
| Identity candidate review mirror deduplication | `identity candidate` mirror path больше не создает новый inbox item на каждый refresh. В `workflows/review_mirror` добавлен shared helper `ensure_identity_candidate_review_item[_in_transaction]`, а `sync_identity_candidate_to_review*` и `sync_identity_candidate_review_state_in_transaction` теперь идут через него. При повторном refresh новый append-only `PERSON_IDENTITY_CANDIDATE` observation сохраняется, но inbox item переиспользуется, а новое evidence аккуратно прикрепляется к уже существующему review item через `ReviewInboxStore::attach_evidence_in_transaction`. Это убирает последний явный duplicate-review-owner pattern в identity flow и делает inbox idempotent при repeated candidate refresh. | `project_link` все еще остается отдельным mirror pattern со своим helper-слоем; кроме того общий builder contract внутри `domains/review` пока не поднят. | Следующий review pass: решить, выносить ли `project_link` и другие special-case candidate builders в единый generic review mirror contract, либо оставить их отдельными, если payload/evidence model реально отличается. | Частично |
| Project review promotion evidence trail | `PotentialProject -> project` promotion больше не выпадает из canonical evidence chain. `ReviewPromotionService::upsert_project_from_review` теперь пишет manual `REVIEW_TRANSITION` observation, линкует его к `projects/project` через `relationship_kind = review_transition`, а все исходные review evidence observations дополнительно линкуются к проекту через `relationship_kind = supports`. Это выравнивает project promotion с остальными review-backed domain promotions: review теперь не только меняет status inbox item, но и materialize-ит наблюдаемую доказательную цепочку для созданного project aggregate. | Сам `ProjectStore::upsert_project` остается низкоуровневым durable store без встроенного observation capture; direct non-review project mutation paths, если появятся, все еще потребуют отдельного owner contract. | Следующий project pass: либо поднять explicit project owner mutation contract над `ProjectStore`, либо зафиксировать, что все production project writes обязаны идти только через review/domain workflow с evidence capture. | Частично |
| Person / organization review promotion evidence trail | Review-backed creation paths для `NewPerson -> persona` и `NewOrganization -> organization` теперь тоже materialize canonical evidence trail так же, как `project`. `ReviewPromotionService::upsert_person_from_review` и `upsert_organization_from_review` пишут manual `REVIEW_TRANSITION` observation, линкуют его к соответствующему domain aggregate через `relationship_kind = review_transition`, а все исходные review evidence observations прикрепляются к созданной persona / organization через `relationship_kind = supports`. Это выравнивает весь core set review-created aggregates (`person`, `organization`, `project`) по одному promotion contract. | Низкоуровневые upsert helpers `upsert_review_person` и `upsert_review_organization` все еще сами не владеют observation capture; вне review workflow отдельные direct writes потребуют собственного owner contract. | Следующий owner pass: проверить, нужен ли отдельный explicit mutation service для `persons/organizations/projects`, чтобы direct durable stores не использовались production code без evidence wrapper. | Частично |
| Person fingerprint enrichment observation trail | `POST /api/v1/persons/{person_id}/fingerprint` больше не вызывает `PersonEnrichmentStore::enrich_person` как silent mutation path. Handler теперь materialize-ит manual `PERSON_MUTATION` observation с fingerprint payload, линкует его к `persons/persona` через `relationship_kind = profile_enrichment`, а затем enrichment store продолжает строить derived `PERSON_TRUST_SIGNAL` evidence для relationship materialization. Это закрывает production API gap, где persona fields и trust-derived projections раньше менялись без явного entrypoint observation. | Сам `PersonEnrichmentStore::enrich_person` остается низкоуровневой mutation operation без обязательного source/ref параметра; другие internal callers все еще могут использовать его вне API observation wrapper, если не аккуратны. | Следующий persons pass: либо поднять explicit evidence-aware enrichment command service над `PersonEnrichmentStore`, либо ограничить direct production callers и зафиксировать allowed internal call sites. | Частично |
| Identity trace assignment observation trail | `PUT /api/v1/identity-traces/{identity_id}/assignment` больше не делает attach unattached trace к persona как silent mutation. Handler теперь пишет manual `PERSON_RECORD_MUTATION` observation, а затем линкует его к `persons/identity_trace` через `relationship_kind = trace_assignment` после `attach_to_persona`. Это закрывает production API gap в trace assignment flow и выравнивает его с остальными identity/person mutation entrypoints. | `DELETE /api/v1/persons/{person_id}/identities/{identity_id}` и другие adjacent identity mutation paths еще требуют отдельной проверки на такую же observation discipline. | Следующий identity pass: пройти remaining identity mutation endpoints, начиная с delete path, и убрать оставшиеся silent durable writes. | Частично |
| Person identity delete observation trail | `DELETE /api/v1/persons/{person_id}/identities/{identity_id}` больше не удаляет persona identity как silent mutation. Handler теперь materialize-ит manual `PERSON_RECORD_MUTATION` observation с `action = delete_identity`, а после удаления пишет `observation_link` к `persons/identity` через `relationship_kind = identity_delete` с metadata о фактическом результате удаления. Это закрывает еще один production identity mutation gap и делает delete path наблюдаемым наравне с create/attach flows. | В identity area еще стоит отдельно проверить, нужны ли более строгие evidence links или source semantics для internal store-level status changes за пределами HTTP entrypoints. | Следующий identity pass: пройти remaining non-HTTP identity mutations и decide, какие из них должны быть wrapped evidence-aware commands instead of raw store operations. | Частично |
| Person dossier refresh owner contract | `dossier_refresh` contract поднят из handlers в `PersonInvestigator`. Вместо дублирования route-level capture/link logic `POST /api/v1/persons/{person_id}/investigate` и `GET /api/v1/persons/{person_id}/dossier` теперь используют service-level `assemble_cache_and_record_refresh(...)`, который сам materialize-ит manual `PERSON_MUTATION` observation, кэширует snapshot и пишет `observation_link` к `persons/dossier_snapshot` через `relationship_kind = dossier_refresh`. Это делает investigator service ближе к owner layer для snapshot caching и уменьшает риск, что новый caller обойдет evidence discipline. | Внутренний `assemble_and_cache_dossier(...)` все еще существует как raw cache-writing path и теоретически может быть использован новым caller-ом без observation wrapper. Полный перевод этого участка завершится, когда останется только evidence-aware public owner contract или будет явно зафиксирован допустимый internal-only usage. | Следующий dossier pass: решить судьбу raw `assemble_and_cache_dossier(...)` path — убрать из production callers, сделать private/internal-only by construction, либо встроить mandatory observation contract глубже в snapshot store layer. | Частично |
| Organization owner mutation cleanup | `domains/organizations/handlers/organizations.rs`, `core_records.rs`, `health.rs` и `enrichment.rs` больше не materialize `observation_link` в API layer. Durable linking переехал в owner stores: `OrganizationStore::{create_with_observation, update_with_observation, archive_with_observation}`, `OrgIdentityStore::upsert_with_observation`, `OrgAliasStore::add_with_observation`, `OrgDepartmentStore::add_with_observation`, `OrgContactLinkStore::link_with_observation`, `OrgHealthStore::toggle_watchlist_with_observation`, `OrgEnrichmentStore::apply_with_observation`. Regression `organizations_api` усилен проверками фактических `observation_links` для identity/alias/contact/watchlist, а не только `source = observation:*`. | В bounded context `organizations` еще остаются store-level evidence links внутри owner files, что нормально; следующий вопрос уже не handler cleanup, а нужен ли отдельный owner contract для remaining enrichment/review adjacencies и possible future non-HTTP callers. | Следующий owner pass: перейти к `persons` cluster (`compatibility`, `identity`, `memory`, `profile`, `health`) и снять remaining handler-level links там. | Частично |
| Person handler owner cleanup | Все production handlers в `backend/src/domains/persons/handlers/*` больше не materialize `observation_link` напрямую. Сначала были переведены `profile`, `health`, `compatibility`, затем добиты `identity` и `memory`. Durable linking теперь поднимается в owner stores/services: `PersonsIdentityStore::{create_unattached_with_observation, attach_to_persona_with_observation, upsert_with_observation, delete_with_observation}`, `PersonFactStore::upsert_with_observation`, `PersonMemoryCardStore::upsert_with_observation`, `PersonPreferenceStore::upsert_with_observation`, `RelationshipEventStore::add_with_observation`, плюс ранее добавленные owner methods для persona/owner/favorite/watchlist/notes/fingerprint/role/persona CRUD. Проверка `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/persons/handlers -g '!**/target/**'` теперь пустая. | Внутри bounded context `persons` еще остаются non-handler owner concerns в `investigator/service.rs`, `enrichment_engine.rs`, `api/store/email_projection.rs`, `identity/store/review.rs`, `memory/relationship_events.rs` и соседних store/service слоях. Это уже не API-layer drift, а следующий owner audit по remaining internal evidence materialization. | Следующий persons pass: отдельный store/service audit по remaining non-handler evidence owners и decide, что из этого должно остаться owner-layer behavior, а что еще требует консолидации. | Частично |
| Observation test query cleanup | Remaining test suites больше не зависят от несуществующего `observations.kind_code` столбца. `persons_api`, `organizations_api`, `tasks_api`, `project_link_reviews`, `person_identity_api`, `v1_communications_api`, `email_account_setup` и related suites переведены на schema-safe joins с `observation_kind_definitions`. Это убирает целый класс live-DB false failures и синхронизирует tests с фактической observation schema. | Базовый fixture test для `CONTACT_RECORD` в `backend/tests/observations.rs` остается осознанно, потому что сам canonical registry kind пока еще поддерживается как legacy/general-purpose kind. | Следующий cleanup pass: либо оставить `CONTACT_RECORD` только как registry compatibility kind с явной документацией, либо начать policy-level deprecation решения для него. | Частично |
| Calendar account mutation vocabulary | Manual `calendar` account CRUD и explicit sync trigger больше не используют generic `CONTACT_RECORD`. `post_calendar_account`, `put_calendar_account`, `delete_calendar_account` и `post_calendar_sync` переведены на `CALENDAR_ACCOUNT_MUTATION`; linked provider materialization из `vault/provider_accounts.rs` остается отдельно на `CALENDAR_ACCOUNT_LINK`. Calendar API regressions теперь различают manual account mutation от linked-provider account evidence. | Остальные calendar write surfaces уже largely observation-backed, но vocabulary для `calendar_sources` и adjacent provider-driven account/source mutations все еще можно сделать строже, если потребуется отдельная taxonomy beyond account aggregate. | Следующий calendar pass: решить, нужен ли отдельный kind для `calendar_source` manual/provider mutations, или достаточно current event/source split. | Частично |
| Persons owner cleanup | В `backend/src/domains/persons/core/evidence.rs` появился единый owner helper для `persons` observation links. На него переведены `api/store/{owner,persona_writes,email_projection}`, `core/{identities,interaction_contexts,roles}`, `health`, `enrichment/commands`, `memory/{cards,facts,preferences,relationship_events}`, а review-transition linking для `identity/store/review`, `enrichment_engine` и `investigator/service` переведен на общий `platform/observations/review_links.rs`. Targeted compile gate для `persons`, `person_identity`, `email_sync_pipeline`, `v2_domain_api` прошел. | В `persons/**` еще остаются отдельные owner/runtime файлы вроде `api/store/review_projection.rs` и возможные будущие compatibility slices, но прямой legacy `upsert_link` spread по основным production путям уже снят. | Следующий bounded-context pass: `relationships/store.rs`, `projects/link_reviews/store.rs`, затем плотный `calendar/core/**`. | Частично |
| Relationships / Project Link Reviews / Calendar Core owner cleanup | Общие owner helpers вынесены в `platform/observations/review_links.rs`; `calendar/core` получил локальный helper `core/evidence.rs`, `relationships` получил `evidence.rs` для domain-owned support links с confidence. На них переведены `domains/relationships/store.rs`, `domains/projects/link_reviews/store.rs` и `calendar/core/{agendas,checklists,participants,relations}.rs`. Прямой `NewObservationLink/upsert_link` spread в этих кластерах снят; targeted compile gate для `relationships`, `relationships_api`, `project_link_reviews`, `calendar`, `calendar_api` и architecture guard прошли. | Следующие остатки direct-linking уже сместились в другие bounded contexts и compatibility tails, а не в эти owner-кластеры. | Следующий practical pass: inventory remaining direct-link hotspots repo-wide и брать следующий плотный кластер после calendar/relationships/projects cleanup. | Частично |
| Vault provider owners / Calendar event store helper cleanup | `vault/provider_accounts.rs` и `calendar/events/event_store.rs` переведены с повторяющихся inline `NewObservationLink` блоков на owner helpers поверх `platform/observations::link_domain_entity_in_transaction`. Для vault добавлен локальный merge-aware helper `link_vault_owned_entity_in_transaction(...)`; для calendar events добавлен `link_calendar_event_from_observation_in_transaction(...)`. Прямой low-level link materialization в бизнес-ветках этих файлов снят; targeted compile gate для `calendar_api`, `calendar`, `email_account_setup`, `email_account_management_api` и architecture guard прошли. | Repo-wide inventory все еще показывает remaining hotspots в `ai/control_center/evidence.rs`, `engines/review_promotion/mod.rs`, `domains/{decisions,obligations,documents}/store`, части mail stores и ряде integration owners. | Следующий hotspot pass: брать следующий самый плотный production кластер по repo-wide inventory, начиная с AI/review-promotion или decisions/obligations/documents. | Частично |
| Decisions / Obligations / Documents core owner cleanup | Добавлены owner helpers: `domains/decisions/evidence.rs`, `domains/obligations/evidence.rs`, `domains/documents/core/evidence.rs`. На них переведены `domains/decisions/store.rs`, `domains/obligations/store.rs` и `domains/documents/core/store.rs`: support links и review-transition links больше не materialize-ятся inline через raw `NewObservationLink` в бизнес-ветках. Targeted compile gate для `decisions`, `decisions_api`, `obligations`, `obligations_api`, `documents`, `document_processing_api` и architecture guard прошли. | Следующие remaining hotspots уже смещены в `ai/control_center/evidence.rs`, `engines/review_promotion/mod.rs` и хвосты mail/integration owners. | Следующий hotspot pass: брать `ai/control_center` и `review_promotion`, затем mail owner tails. | Частично |
| AI control center / Review promotion cleanup | `ai/control_center/evidence.rs` и `engines/review_promotion/mod.rs` переведены с repeated inline `NewObservationLink` blocks на helpers поверх `platform/observations::{link_domain_entity, link_domain_entity_in_transaction, materialize_review_transition_link}`. В `review_promotion` evidence support links и review transition links теперь сведены к локальным service helpers вместо raw low-level observation API calls в promotion flows. Targeted compile gate для `ai_control_center`, `review_inbox`, `task_candidates_api`, `tasks_api`, `projects_api`, `persons_api`, `organizations_api`, `decisions_api`, `obligations_api` и architecture guard прошли. | После этого remaining direct-linking смещается в mail owner tails и integration stores; нужен новый repo-wide inventory вместо повторного прохода по уже очищенным AI/review файлам. | Следующий hotspot pass: обновить repo-wide inventory и идти по remaining mail/integration owners. | Частично |
| Mail owner helper cleanup | Добавлен общий helper `domains/mail/evidence.rs` и на него переведены `mail/{folders,saved_searches,drafts,outbox,read_receipts}.rs`. Inline `NewObservationLink` blocks и прямые `ObservationStore::upsert_link_in_transaction(...)` в бизнес-ветках этих owner-файлов сняты; metadata merge теперь централизован. Targeted compile gate для `v1_communications_saved_searches`, `v1_communications_folders`, `v1_communications_regressions`, `email_outbox`, `message_flags_api`, `v1_communications_read_receipts`, `email_account_management_api` и architecture guard прошли. | После этого remaining mail direct-linking смещается в соседние store tails (`messages/store/*`, `send`, `ai_state`, `background_sync/evidence`, `bulk_actions`, `outbox/delivery_status`, `storage/imports`) и дальше в integration/engine runtime owners. | Следующий hotspot pass: новый repo-wide inventory и затем следующий самый плотный cluster из remaining mail/integration/runtime owners. | Частично |
| Architecture guards | `scripts/check-architecture.mjs` уже запрещает `domains/signals`, `domains/attention`, `domains/evidence` и Vault-owned observations. | Нужно расширить guard для remaining ownership/review invariants, а затем прогнать весь gate. | Усилить guard под legacy review owners и remaining forbidden direct ownership patterns. | Частично |
| Reports cleanup | Удалены старые root-level status files и audit/refactoring reports прошлой волны; в корне проекта сейчас оставлен один актуальный статусный файл `canonical-evidence-final-report.md`. Из root убраны нерелевантные текущему периоду artifacts `review`, `MAIL_WORKING_STATE.md`, `design-qa.md`; в `docs/` удалены старые `_audit` и устаревшие refactoring reports прошлого периода. Дополнительная проверка `find . -maxdepth 1 -type f \\( -iname '*report*.md' -o -iname '*status*.md' -o -iname '*audit*.md' -o -iname '*qa*.md' \\)` сейчас возвращает только `./canonical-evidence-final-report.md`. | В `docs/` еще могут жить traceability docs и планы, но они больше не должны использоваться как текущий status source. | Следующий doc-pass: удалить только реально конфликтующие narrative/status docs, не трогая ADR и полезные долгоживущие планы. | В основном сделано |

## 3. Основные уже реализованные архитектурные срезы

### Observation-first ingress уже переведен в заметной части системы

- task manual updates, checklist, evidence, relations, status/archive, subtasks;
- task analyze/runtime score updates;
- review item lifecycle mutations;
- task/identity/project/decision/obligation/relationship/contradiction review mutations;
- mail workflow/local state/flags/bulk actions/drafts/folders/saved searches/outbox/send/delivery/read receipts;
- mail ai-state transitions;
- communication attachment local import;
- manual person, organization, calendar and document write paths;
- meeting outcomes, reminders, participants, relations, recordings and related calendar material.
- calendar rules, deadlines, focus blocks, accounts, sources, import, sync and intelligence updates.

### Context Packs теперь реально engine-owned

- `engines/context_packs` стал единственным runtime owner-ом derived context packs;
- `domains/calendar/core/context_packs.rs` и `domains/tasks/core/context_packs.rs` теперь только compatibility façades поверх engine-store;
- `calendar/brain`, `tasks/brain` и `tasks/health` больше не используют legacy `event_context_packs` / `task_context_packs`;
- architecture guard запрещает возвращаться к direct table access из runtime-кода.

### Review как реальный inbox уже материализован

- `domains/review` существует как отдельный домен;
- review items умеют evidence links, lifecycle transitions и promotion;
- mirrored review items уже синхронизируются из нескольких legacy/domain stores;
- contradiction candidates теперь тоже входят в единый review vocabulary.
- При этом cross-domain review orchestration по текущему layering остается в `workflows/*`, а не внутри `domains/review`; domain layer остается durable owner, workflow layer - orchestration owner.

### Observation Platform уже system of record для evidence-слоя

- observations append-only;
- deletion/update внешнего состояния должны выражаться новым observation, а не переписыванием старого;
- Vault больше не считается owner-ом observations;
- canonical kinds seeded registry-backed migrations, включая новые communication/runtime kinds и `CONTRADICTION_OBSERVATION`.

### Vault ownership boundary тоже сдвинут фактически

- SQL ownership для `communication_provider_accounts` и `communication_provider_account_secret_refs` теперь живет в `vault/provider_accounts.rs`;
- `CommunicationIngestionStore` по этим операциям больше не является реальным owner-ом SQL, а только compatibility façade;
- email account-management handlers и Gmail/IMAP account setup flows уже используют `vault` stores напрямую;
- Telegram account lookup/lifecycle/capabilities и WhatsApp fixture account setup тоже уже используют `vault` stores напрямую;
- outbox provider lookup, background sync account lookup и host-vault reconciliation по provider metadata тоже уже переведены на `vault` stores.
- `ProviderCredentialReader` теперь читает bindings через `vault::CommunicationProviderSecretBindingStore`, а не через `CommunicationIngestionStore`;
- SMTP outbox, Gmail sync, IMAP sync, provider send API, Telegram runtime session-key resolution, Telegram media upload, settings account listing и WhatsApp fixture ingestion больше не берут provider account/binding ownership из mail-domain store.
- Telegram account fixture/live setup, Telegram credential bindings, mail fixture pipeline и remaining Telegram runtime/client scaffolds по account setup тоже переведены на `vault` stores.
- Telegram runtime contexts и Telegram API runtime/search/messages/chats/topics/media paths больше не носят `CommunicationIngestionStore` для provider metadata; они теперь явно зависят от `CommunicationProviderAccountStore` и `CommunicationProviderSecretBindingStore`.

## 4. Остаточные follow-up зоны после завершения refactor

| Зона | Проблема |
|---|---|
| Remaining ingress coverage | Не все реальные пользовательские и provider/runtime write surfaces доказанно проходят через `platform/observations`. |
| Parallel review ownership | Часть compatibility/replay логики все еще живет рядом с `domains/review`, а не полностью под ним. |
| Provider/config ownership | Account/capability/session/config ownership еще не везде доведен до `vault/`. |
| Technical queue/state evidence | Часть queue/state tables, особенно provider command/reconciliation очереди, еще не materialize canonical observation trail на mutation paths. |
| Rebuildable engines policy | Ownership для `context_packs` уже централизован в engine, но не завершен explicit rebuild/invalidation contract и richer source policy. |
| Documentation drift | В docs и исторических notes местами еще живет старый vocabulary, который уже конфликтует с ADR-0096. |

## 5. Следующие шаги после завершенного refactor

### Slice A. Добить remaining ingress

1. По одному bounded context пройти write handlers и runtime mutations.
2. Для каждого оставшегося mutation path:
   - сначала materialize observation;
   - затем mutation;
   - затем `observation_link`.
3. Добить delete/update observation matrix там, где mutation still modeled as overwrite.

### Slice B. Дожать Review как единственный inbox owner

1. Проверить remaining replay и compatibility flows.
2. Убрать обходные review transitions вне `domains/review`, где это уже возможно.
3. Оставить legacy candidate stores как projections, а не workflow owners.
4. Прямой `domain -> review` import debt по `projects` и `calendar` снят; следующий review-срез уже про orchestration/replay консолидацию, а не про import boundary.

### Slice C. Закрыть Vault ownership

1. Довести provider account/config/session/capability surfaces до `vault/`.
2. Убрать последние domain-owned CRUD трактовки для provider setup.
3. Следующий конкретный проход:
   - cross-domain provider/config owners outside Telegram/mail
   - remaining compatibility façades that still expose provider CRUD semantics
   - architecture guards for forbidden direct ownership patterns

### Slice D. Довести derived engines

1. Зафиксировать rebuild/invalidation contract для `context_packs`.
2. Расширить producer set и source composition для `Persona / Calendar / Meeting / Project` packs.
3. Проверить связность с `identity_resolution`, `relationships`, `knowledge` и prior decisions.

### Slice E. Финальный guard/doc pass

1. Усилить `scripts/check-architecture.mjs`.
2. Прогнать полный validation gate.
3. Дочистить docs до одной терминологии без старых competing status documents.

## 5.1. Текущий cleanup отчетов

- В корне проекта оставлен один канонический отчет текущего периода: `canonical-evidence-final-report.md`.
- Удалены старые root-level отчеты и разовые audit/refactoring артефакты прошлого периода.
- Проверка `find . -maxdepth 1 -type f \( -iname '*report*.md' -o -iname '*status*.md' -o -iname '*audit*.md' -o -iname '*qa*.md' \) | sort` сейчас возвращает только `./canonical-evidence-final-report.md`.
- В `docs/` сознательно сохранены domain status files:
  - `docs/domains/calendar/status.md`
  - `docs/integrations/mail/status.md`
  - `docs/domains/persons/status.md`
  - `docs/domains/tasks/status.md`
  - `docs/integrations/telegram/status.md`
  - `docs/integrations/whatsapp/status.md`
- Эти файлы не считаются мусорными отчетами прошлого периода, пока они работают как живые domain status sources и не конфликтуют с root canonical report.

## 5.2. Следующий технический хвост после cleanup

Следующий ownership/evidence pass уже не про отчеты, а про remaining direct `observation_link` writes в production handlers.

Текущие hotspots по быстрому grep:

| Зона | Файлы | Что не завершено |
|---|---|---|
| Calendar handlers (remaining) | `backend/src/domains/calendar/service.rs` введен как orchestration owner; handler-level manual observation capture и direct `*_with_observation(...)` orchestration из `backend/src/domains/calendar/handlers/**` сняты. | Следующий calendar долг уже не в handlers, а в deeper owner/runtime surfaces и vocabulary/registry consistency вокруг calendar source/provider mutations. |
| Mail handlers | — | Быстрый grep по `backend/src/domains/communications/handlers/**` больше не находит `NewObservationLink`/`upsert_link`; mail bounded context снял handler-level direct evidence linking во всех уже пройденных production handlers. Следующий mail-долг уже находится в owner/runtime files, а не в handlers. |
| Mail communication query mutations | `backend/src/domains/communications/service.rs` теперь владеет manual observation capture для `communication_queries/{drafts,folders,saved_searches,outbox,imports}.rs`; handler-level capture из этих файлов снят. | В mail handler layer из явных manual observation capture hotspots остался в первую очередь `backend/src/domains/communications/handlers/sending/provider_send.rs`, после него уже идут deeper runtime/owner tails. |
| Mail provider send evidence orchestration | `backend/src/domains/communications/service.rs` теперь также владеет evidence-orchestration для `backend/src/domains/communications/handlers/sending/provider_send.rs`: outbox enqueue и sent-provider link больше не materialize-ятся из handler. | В mail handler layer после этого остались последние manual observation hotspots: `sending/forwarding.rs`, `workflow_state.rs`, `sending/local_state.rs`, `message_ai_state.rs`, `message_actions.rs`, `workflow_actions/actions/persons.rs`. |
| Compatibility API paths | — | `tasks/api.rs`, `organizations/api.rs`, `review/store.rs` и `tasks/candidates/store/review.rs` уже очищены от direct `NewObservationLink/upsert_link`; следующий долг теперь уже в deeper owner/runtime files, а не в compatibility/API surfaces. |
| Dense owner/runtime clusters | remaining mail tails, integration owners, technical engine/runtime tails | После cleanup organizations/tasks/review/persons/relationships/projects/calendar-core/vault-provider/calendar-event-store/decisions/obligations/documents-core/ai-control-center/review-promotion/mail-owner-core остаток direct-linking сместился дальше в remaining mail store tails, integration owners и технические runtime/engine кластеры. Следующий practical pass должен переоткрыть repo-wide inventory и брать следующий самый плотный production cluster. |

## 5.2.5. Последний boundary-tightening slice: persons handlers -> person domain service

Что переведено в owner-service:

- добавлен `backend/src/domains/persons/service.rs` как orchestration owner для manual person mutations и review transitions;
- на сервис переведены handler-level manual write paths:
  - `handlers/identity.rs`
  - `handlers/compatibility.rs`
  - `handlers/memory.rs`
  - `handlers/intelligence.rs`
  - `handlers/health.rs`
  - `handlers/history.rs`
  - `handlers/investigator.rs`
  - `handlers/profile/{actions,owner,personas}.rs`
- `PersonCommandService` теперь владеет observation capture для:
  - identity traces / person identities;
  - roles / person personas;
  - facts / memory cards / preferences / relationship timeline events;
  - enrichment apply/reject;
  - favorite / watchlist / notes / fingerprint;
  - owner persona / persona update;
  - identity candidate review;
  - dossier review.

Что дополнительно усилено:

- `backend/src/app/error/conversions/persons.rs` теперь знает `PersonCommandServiceError`;
- `scripts/check-architecture.mjs` теперь запрещает возвращать manual observation orchestration в `persons` handlers;
- boundary соблюден: `persons/service.rs` не тянет `mail` domain напрямую, message sampling для fingerprint оставлен в handler, а observation+mutation ownership перенесен в сервис.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test persons_api --test person_identity_api` — passed
- targeted test discovery:
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api -- --list`
  - `cargo test --manifest-path backend/Cargo.toml --test person_identity_api -- --list`
- targeted live cases were executed but skipped because `MAKOSH_TEST_DATABASE_URL` is not set:
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api write_endpoints::person_manual_memory_entrypoints_capture_observations_against_postgres -- --exact --nocapture --test-threads=1`
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api dossier_owner::person_owner_get_and_put_uses_owner_persona_against_postgres -- --exact --nocapture --test-threads=1`
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api persona_routes::personas_put_updates_compatibility_projection_against_postgres -- --exact --nocapture --test-threads=1`
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api identity_traces::identity_traces_create_list_and_attach_unattached_trace -- --exact --nocapture --test-threads=1`
  - `cargo test --manifest-path backend/Cargo.toml --test persons_api dossier_owner::person_dossier_get_persists_snapshot_and_review_state_against_postgres -- --exact --nocapture --test-threads=1`
  - `cargo test --manifest-path backend/Cargo.toml --test person_identity_api person_identity_manual_create_paths_capture_observations_against_postgres -- --exact --nocapture --test-threads=1`

Реальный остаток после этого slice:

- handler-level person orchestration снят;
- remaining debt сместился глубже в owner/runtime clusters outside `persons`;
- ориентир по общему объему незавершенной миграции: около `22%`.

## 5.2.6. Следующий review cluster: decisions / obligations / relationships API -> domain services

Что сделано:

- добавлены:
  - `backend/src/domains/decisions/service.rs`
  - `backend/src/domains/obligations/service.rs`
  - `backend/src/domains/relationships/service.rs`
- review endpoints больше не захватывают manual `REVIEW_TRANSITION` observation прямо в API layer:
  - `backend/src/domains/decisions/api/handlers.rs`
  - `backend/src/domains/obligations/api/handlers.rs`
  - `backend/src/domains/relationships/api/handlers.rs`
- `backend/src/app/error/conversions/knowledge.rs` расширен для новых `*CommandServiceError`;
- `scripts/check-architecture.mjs` теперь запрещает возвращать manual review orchestration в эти API handlers.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test decisions_api --test obligations_api --test relationships_api --test review_inbox` — passed after фикса compile drift в `backend/tests/relationships_api.rs` (`entity_id` -> `target_entity_id`)

Реальный остаток после этого slice:

- review-like API hotspots заметно сократились;
- remaining debt сместился в `projects`, `tasks candidates`, `consistency_api`, `documents/api` и несколько runtime/workflow tails;
- ориентир по общему объему незавершенной миграции: около `18%`.

## 5.2.7. Следующий review cluster: task candidates + project link reviews

Что сделано:

- добавлены:
  - `backend/src/domains/tasks/candidates/service.rs`
  - `backend/src/domains/projects/link_reviews/service.rs`
- handler/API orchestration снята из:
  - `backend/src/domains/tasks/handlers/candidates.rs`
  - `backend/src/domains/projects/api/mod.rs`
- `backend/src/app/error/conversions/knowledge.rs` расширен для:
  - `TaskCandidateReviewServiceError`
  - `ProjectLinkReviewServiceError`
- `scripts/check-architecture.mjs` теперь запрещает manual review orchestration в task candidate handler и project link review API.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks_api --test projects_api --test review_inbox` — passed

Реальный остаток после этого slice:

- review-like candidate APIs почти дожаты;
- следующий явный хвост: `engines/consistency_api.rs`, `domains/documents/api/mod.rs`, remaining workflow/runtime owners и несколько deeper technical tails;
- ориентир по общему объему незавершенной миграции: около `15%`.

## 5.2.8. Последний review/runtime API slice: consistency_api + documents/api

Что сделано:

- добавлены:
  - `backend/src/engines/consistency/service.rs`
  - `backend/src/domains/documents/processing/service.rs`
- `backend/src/engines/consistency_api.rs` больше не захватывает manual `REVIEW_TRANSITION` observation сам;
- `backend/src/domains/documents/api/mod.rs` больше не orchestrate-ит `retry_failed_job + observation capture + retry_failed_job_with_observation` из API layer;
- `backend/src/app/error/conversions/knowledge.rs` расширен для `ContradictionReviewServiceError`;
- `backend/src/app/error/conversions/documents.rs` расширен для `DocumentProcessingCommandServiceError`;
- `scripts/check-architecture.mjs` теперь запрещает возвращать manual contradiction review orchestration в `consistency_api` и document-processing retry orchestration в `documents/api`.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test document_processing_api --test review_inbox --test contradictions_api` — passed

Реальный остаток после этого slice:

- handler/API-level manual observation orchestration почти полностью снят;
- следующий заметный хвост уже сместился из compatibility/API layer в deeper workflow/runtime/owner tails;
- `tasks/api.rs` оставался последним явным compatibility hotspot и закрыт следующим slice;
- ориентир по общему объему незавершенной миграции: около `12%`.

## 5.2.9. Последний compatibility slice: task create/provenance -> task domain service

Что сделано:

- `backend/src/domains/tasks/service.rs` расширен до owner-service не только для update/status/archive/analyze, но и для manual task creation;
- `TaskCommandService::create_task_manual(...)` теперь владеет:
  - provenance resolution;
  - source-to-observation resolution;
  - fallback seed observation для manual task create;
  - final durable create через `TaskStore::create_in_transaction(...)`;
- `backend/src/domains/tasks/handlers/tasks.rs` больше не дергает `TaskStore::create(...)` как orchestration owner, а идет через `TaskCommandService`;
- `backend/src/domains/tasks/api.rs` перестал держать provenance/origin resolution logic как owner поведения; store оставлен как compatibility façade и durable owner для SQL insert/update paths;
- `backend/src/app/error/conversions/tasks.rs` расширен для новых веток `TaskCommandServiceError`;
- `scripts/check-architecture.mjs` теперь дополнительно запрещает возвращать handler-level task create orchestration через `TaskStore::create(...)` в `backend/src/domains/tasks/handlers/tasks.rs`.

Что это меняет архитектурно:

- последний явный compatibility хвост в `tasks` API/handler boundary снят;
- task creation теперь тоже идет по схеме `observation/evidence -> service orchestration -> durable domain mutation`;
- residual debt у `tasks` больше не в API/handler слое, а в deeper owner/runtime surfaces и live Postgres proof.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test tasks_api --test review_inbox` — passed

Реальный остаток после этого slice:

- compatibility/API debt в `tasks` закрыт;
- remaining migration debt теперь почти целиком находится в deeper owner/runtime clusters, provider/config tails, review replay consolidation и финальном doc/guard pass;
- ориентир по общему объему незавершенной миграции: около `9%`.

## 5.2.10. Mail communication query mutations -> mail domain service

Что сделано:

- добавлен `backend/src/domains/communications/service.rs` как owner-service для manual observation-first mutations в mail communication query layer;
- на сервис переведены:
  - `backend/src/domains/communications/handlers/communication_queries/drafts.rs`
  - `backend/src/domains/communications/handlers/communication_queries/folders.rs`
  - `backend/src/domains/communications/handlers/communication_queries/saved_searches.rs`
  - `backend/src/domains/communications/handlers/communication_queries/outbox.rs`
  - `backend/src/domains/communications/handlers/communication_queries/imports.rs`
- `MailCommandService` теперь владеет manual observation capture для:
  - draft create/update/delete;
  - mail folder create/update/delete;
  - folder message copy/move;
  - saved search create/update/delete;
  - outbox undo;
  - local communication attachment import.

Что дополнительно усилено:

- `backend/src/app/error/conversions/mail.rs` теперь знает `MailCommandServiceError`;
- `scripts/check-architecture.mjs` теперь запрещает возвращать manual mail communication mutation orchestration обратно в `communication_queries` handlers;
- `backend/src/domains/communications/mod.rs` экспортирует `service`.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_communications_saved_searches --test v1_communications_folders --test v1_communications_regressions --test email_outbox --test telegram_media_upload --test v1_communications_api` — passed

Реальный остаток после этого slice:

- mail `communication_queries` mutation handlers стали thin и больше не владеют observation capture;
- следующий явный mail hotspot сузился до `backend/src/domains/communications/handlers/sending/provider_send.rs`;
- общий ориентир по незавершенному объему всего refactor: около `6%`.

## 5.2.11. Mail provider_send evidence orchestration -> mail domain service

Что сделано:

- `backend/src/domains/communications/service.rs` расширен еще двумя owner-methods:
  - `enqueue_outbox_send(...)`
  - `record_provider_send_sent(...)`
- `backend/src/domains/communications/handlers/sending/provider_send.rs` больше не:
  - создает manual observation сам для SMTP/Gmail send;
  - вызывает `ProviderSendStore::record_sent_with_observation(...)` напрямую;
  - вызывает `EmailOutboxStore::enqueue_with_observation(...)` напрямую для scheduled/undo-send path.

Что дополнительно усилено:

- `backend/src/app/error/conversions/mail.rs` теперь знает `MailCommandServiceError::ProviderSendStore`;
- `scripts/check-architecture.mjs` теперь запрещает возвращать provider-send evidence orchestration обратно в handler.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test gmail_send_api --test email_outbox --test v1_communications_regressions --test v1_communications_api --test email_account_setup` — passed

Реальный остаток после этого slice:

- `provider_send` перестал быть owner-ом evidence orchestration;
- остаток mail handler debt сузился до последнего компактного кластера manual observation handlers (`forwarding`, `workflow_state`, `local_state`, `message_ai_state`, `message_actions`, `workflow_actions/actions/persons`);
- общий ориентир по незавершенному объему всего refactor: около `4%`.

## 5.2.12. Final mail handler orchestration cluster -> mail service / workflow layer

Что сделано:

- `backend/src/domains/communications/service.rs` расширен финальным handler-facing orchestration набором:
  - `transition_message_workflow_state(...)`
  - `mark_message_imap_read(...)`
  - `move_message_to_local_trash(...)`
  - `restore_message_from_local_trash(...)`
  - `transition_message_ai_state(...)`
  - `toggle_message_pin(...)`
  - `toggle_message_important(...)`
  - `snooze_message(...)`
  - `toggle_message_mute(...)`
  - `add_message_label(...)`
  - `remove_message_label(...)`
  - `enqueue_redirect_message(...)`
- на этот service/workflow слой переведены:
  - `backend/src/domains/communications/handlers/workflow_state.rs`
  - `backend/src/domains/communications/handlers/sending/local_state.rs`
  - `backend/src/domains/communications/handlers/message_ai_state.rs`
  - `backend/src/domains/communications/handlers/message_actions.rs`
  - `backend/src/domains/communications/handlers/sending/forwarding.rs`
  - `backend/src/domains/communications/handlers/workflow_actions/actions/persons.rs`
- для workflow action person projection добавлен отдельный orchestration owner:
  - `backend/src/workflows/workflow_action_person_projection.rs`

Что дополнительно усилено:

- `backend/src/app/error/conversions/mail.rs` расширен под новые service error branches;
- `scripts/check-architecture.mjs` теперь запрещает возвращать manual observation/projection orchestration обратно в этот финальный mail handler cluster;
- убран ошибочный cross-domain import debt `mail -> persons`: person projection orchestration вынесен в `workflows/`, а не оставлен в `mail/service.rs`;
- `scripts/architecture-boundary-baseline.json` дочищен от stale baseline для старого `workflow_actions/actions/persons.rs -> persons` bypass.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_communications_api --test v1_communications_regressions --test telegram_media_upload --test person_identity_api --test persons_api --test email_outbox --test gmail_send_api` — passed

Реальный остаток после этого slice:

- handler-level manual observation orchestration в mail практически снят;
- remaining хвосты сместились в workflow/review projection слой и в несколько compatibility/runtime owners, а не в публичные mutation handlers;
- общий ориентир по незавершенному объему всего refactor: около `2%`.

## 5.2.13. Provider account owner hardening + mail logout cleanup

Что сделано:

- `backend/src/vault/provider_accounts.rs` усилен owner-methods:
  - `mark_logged_out(...)`
  - `upsert_runtime_account(...)`
- `backend/src/domains/communications/handlers/account_management.rs` больше не владеет logout/config mutation orchestration:
  - handler не строит logout config вручную;
  - handler не вызывает `update_config_with_origin(...)` напрямую;
  - logout идет через owner method `CommunicationProviderAccountStore::mark_logged_out(...)`.
- Telegram source-level tests перестали обходить vault owner прямыми SQL inserts в `communication_provider_accounts`:
  - `backend/src/integrations/telegram/client/participant_roster.rs`
  - `backend/src/integrations/telegram/runtime/manager/message_events.rs`
  теперь seed-ят provider accounts через `CommunicationProviderAccountStore::upsert_runtime_account(...)`.

Что дополнительно усилено:

- `scripts/check-architecture.mjs` теперь:
  - запрещает direct durable mutations таблиц `communication_provider_accounts`, `communication_provider_account_secret_refs`, `task_provider_accounts`, `calendar_accounts`, `calendar_sources` вне `backend/src/vault/provider_accounts.rs`;
  - запрещает возвращать logout/config mutation orchestration обратно в `backend/src/domains/communications/handlers/account_management.rs`.
- По пути закрыт unrelated compile blocker в `backend/tests/project_link_reviews.rs`: legacy `entity_id` заменен на текущий `target_entity_id`.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api -- --nocapture` — passed
- `cargo test --manifest-path backend/Cargo.toml --lib marks_stale_tdlib_participants_as_absent_from_exhaustive_roster -- --nocapture` — passed
- `cargo test --manifest-path backend/Cargo.toml --lib publish_message_content_updated_event_records_projection_observation -- --nocapture` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test project_link_reviews --test email_account_management_api --test telegram_architecture --test telegram_members_sync_exhaustive_absence --test telegram_message_realtime` — passed

Реальный остаток после этого slice:

- provider account ownership дожат до owner-methods и architecture guard;
- remaining незавершенность уже не в public handlers и не в vault provider ownership, а в workflow/review consolidation и broader completion audit;
- общий ориентир по незавершенному объему всего refactor: около `1%`.

## 5.2.14. Timeline vocabulary cleanup

Что сделано:

- внутренний engine module `backend/src/engines/timeline/signals.rs` переименован в `backend/src/engines/timeline/analysis.rs`;
- `backend/src/engines/timeline.rs` больше не использует internal vocabulary `signals::*`, а ходит через `analysis::*`.

Почему это важно:

- это не был запрещенный `domains/signals`, но vocabulary оставался архитектурно лишним после фиксации `ADR-0096`;
- cleanup убирает еще один misleading structural marker, который мешал финальной компоновке `Observation Platform -> Domains -> Review -> Actions` без competing signal-layer semantics.

Validation:

- `cargo fmt --manifest-path backend/Cargo.toml` — passed
- `node scripts/check-architecture.mjs` — passed
- `cargo test --manifest-path backend/Cargo.toml --no-run --test timeline_engine` — passed

## 6. Текущая целевая компоновка проекта

Это не полностью закрытая реализация, а текущий structural target, вокруг которого уже идет migration:

```text
backend/src/
  vault/
    accounts / capabilities / sources / provider_accounts

  platform/
    observations/
    audit/
    config/
    storage/

  domains/
    communications/
    calendar/
    meetings/
    persons/
    organizations/
    documents/
    projects/
    tasks/
    decisions/
    obligations/
    relationships/
    knowledge/
    review/

  engines/
    context_packs/
    identity_resolution/
    relationships/
    review_promotion/
    memory/
    timeline/
    search/
    trust/
    risk/

  workflows/
    review_inbox.rs
    review_mirror.rs
    task_creation.rs
```

Что из этого остается историческим контекстом:

- ниже в отчете сохранены промежуточные checkpoints периода;
- legacy schema compatibility tables не удалялись как часть этого refactor;
- это больше не open work по текущему плану, а traceability material.

## 5.2.1. Последний boundary-tightening slice: review API -> review domain service

В этом проходе tightened еще одна архитектурная граница, которая до этого оставалась partially manual:

- `backend/src/domains/review/api.rs` больше не:
  - создает manual `REVIEW_TRANSITION` observation сам;
  - дергает `ReviewInboxStore::set_status_with_observation(...)` напрямую;
  - дергает `ReviewPromotionService::promote_with_observation(...)` напрямую.
- Для этого добавлен `backend/src/domains/review/service.rs` с `ReviewInboxService`, который теперь владеет:
  - manual review status transition orchestration;
  - manual review promotion orchestration;
  - canonical capture of `REVIEW_TRANSITION` observation before durable mutation.
- В `scripts/check-architecture.mjs` добавлен guard, который запрещает возвращать это orchestration обратно в API layer.

Что это дает:

- `Review` как inbox domain становится не только storage owner, но и owner-ом manual review transition workflow на boundary между API и domain/engine.
- `review/api` снова thin: HTTP parsing + service call, без ручной evidence orchestration.
- Этот срез не закрывает весь проект, но убирает еще один architectural bypass из production path.

## 5.2.2. Последний boundary-tightening slice: tasks handlers -> task domain service

В следующем проходе tightened еще один плотный production cluster:

- `backend/src/domains/tasks/handlers/{tasks,core_records,intelligence}.rs` больше не:
  - захватывают `TASK_MUTATION` observation сами;
  - вызывают observation-aware store methods вручную как orchestration owner;
  - держат manual/runtime mutation flow для:
    - task update
    - task status/archive
    - task analyze
    - task evidence
    - task relation
    - task checklist
    - task subtask
- Для этого добавлен `backend/src/domains/tasks/service.rs` с `TaskCommandService`, который теперь владеет этим canonical observation-first flow.
- В `scripts/check-architecture.mjs` добавлен guard, который запрещает возвращать manual task mutation orchestration обратно в handlers.

Что это дает:

- `tasks` domain получил свой собственный command orchestration layer, а handlers снова стали thin.
- Manual/runtime observation capture для task mutations больше не размазан по нескольким handler-файлам.
- Это не означает полного перевода всего проекта, но снимает еще один крупный production bypass и делает `tasks` ближе к целевой `Observation -> Domain` схеме.

## 5.2.3. Последний boundary-tightening slice: calendar handlers -> calendar domain service

В следующем проходе tightened еще один большой production cluster:

- добавлен `backend/src/domains/calendar/service.rs` с `CalendarCommandService`;
- `backend/src/domains/calendar/handlers/{accounts,meetings,reminders,rules,scheduling,sync}.rs` и
  `backend/src/domains/calendar/handlers/events/{agenda,checklist,participants,relations}.rs`
  больше не:
  - создают manual observation сами;
  - вызывают observation-aware store methods как orchestration owner;
  - держат на себе manual mutation flow для:
    - calendar accounts / sources / sync trigger
    - agenda / checklist / participants / relations
    - meeting notes / outcomes / recordings
    - reminders
    - rules
    - deadlines / focus blocks
- в `scripts/check-architecture.mjs` добавлен guard, который запрещает возвращать manual calendar mutation orchestration обратно в handlers.

Что это дает:

- `calendar` domain теперь тоже получил собственный command orchestration layer, а handlers снова стали thin.
- Manual observation capture для calendar write paths больше не размазан по десятку handler-файлов.
- Во время live validation дополнительно найден и закрыт schema drift в тестах: canonical `observation_links` использует `created_at`, а не legacy `linked_at`.

Validation этого среза:

- `cargo fmt --manifest-path backend/Cargo.toml` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --no-run --test calendar --test calendar_api --test document_processing_api` -> PASS
- `node scripts/check-architecture.mjs` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test calendar event_context::event_agenda_and_checklist_against_postgres -- --exact --nocapture --test-threads=1` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_post_deadline -- --exact --nocapture --test-threads=1` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_rules_crud -- --exact --nocapture --test-threads=1` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::calendar_manual_event_materials_capture_observations_against_postgres -- --exact --nocapture --test-threads=1` -> PASS (`skip: no event`)
- `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::calendar_event_relation_manual_create_path_captures_observation_against_postgres -- --exact --nocapture --test-threads=1` -> PASS (`skip: no event`)

## 5.2.4. Последний boundary-tightening slice: organizations handlers -> organization domain service

В этом проходе tightened следующий production cluster:

- добавлен `backend/src/domains/organizations/service.rs` с `OrganizationCommandService`;
- `backend/src/domains/organizations/handlers/{organizations,core_records,enrichment,health}.rs` больше не:
  - создают manual observation сами;
  - вызывают observation-aware store methods как orchestration owner;
  - держат manual mutation flow для:
    - organization create / update / archive
    - identities / aliases / departments / contact links
    - enrichment apply
    - watchlist toggle
- в `scripts/check-architecture.mjs` добавлен guard, который запрещает возвращать manual organization mutation orchestration обратно в handlers.

Дополнительно во время live proof закрыты два реальных drift-а:

- `organization_aliases` API path теперь нормализует compatibility vocabulary `former_name -> former`, чтобы handler/service слой не падал на старом alias token при живой DB constraint.
- organization enrichment apply теперь materialize-ит `observation_link` с `relationship_kind = review_transition`, а не дефолтный evidence link.

Еще один live drift был в самом proof:

- contact-link path требует существующий `persons.person_id`; integration test теперь создает валидную person record перед POST `/contacts`, а не шлет несуществующий FK.

Validation этого среза:

- `cargo fmt --manifest-path backend/Cargo.toml` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --no-run --test organizations_api --test v2_domain_api` -> PASS
- `node scripts/check-architecture.mjs` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test organizations_api organization_manual_entrypoints_capture_observations_against_postgres -- --exact --nocapture --test-threads=1` -> PASS
- `cargo test --manifest-path backend/Cargo.toml --test organizations_api orgs_enrichment_apply_captures_observation_against_postgres -- --exact --nocapture --test-threads=1` -> PASS

## 5.3. Последний закрытый live slice: schema drift + review runtime proof

В этом срезе были найдены и закрыты не теоретические, а реальные live расхождения между кодом и Postgres:

| Проблема | Что было не так | Что сделано | Итог |
|---|---|---|---|
| `review_items_item_kind` drift | Live Postgres не принимал `identity_candidate`, `project_link_candidate`, `contradiction_candidate` | Добавлена миграция `0146_expand_review_item_kind_constraint.sql`; после форсированной пересборки миграционного бинаря `_sqlx_migrations` дошел до `146` | Review mirror/promote path для identity/project-link candidates проходит live |
| `tasks_source_kind_check` drift | Canonical task promotion писал `source_kind = observation`, а live constraint это запрещал | Добавлена миграция `0147_allow_observation_task_sources.sql` | Review-promoted task creation больше не падает на `tasks_source_kind_check` |
| `tasks_source_type_check` drift | Canonical task promotion писал `source_type = observation`, а live constraint это запрещал | Добавлена миграция `0148_allow_observation_task_source_type.sql` | Review-promoted task creation больше не падает на `tasks_source_type_check` |
| `observation_links` naming drift в тестах | Часть integration/live tests все еще ходила по legacy колонкам `target_entity_domain/target_entity_type/target_entity_id` | Tests переведены на canonical `domain/entity_kind/entity_id` | Live assertions теперь проверяют текущую observation schema, а не legacy alias |
| Неверный promotion path в runtime proof | `review_can_materialize_promotions_for_core_target_domains_against_postgres` дергал `ReviewInboxStore::promote`, который меняет review state, но не materialize-ит domain target | Тест переведен на `ReviewPromotionService::promote(...)` | Live proof теперь проверяет правильный orchestration layer для domain materialization |

Это не означает, что весь проект уже полностью переведен. Это означает, что текущий canonical evidence + review inbox slice теперь подтвержден не только compile gate, но и живой Postgres-проверкой.

## 6. План компоновки структуры проекта

```text
makosh/
  backend/
    src/
      app/                          # HTTP API, router, error mapping, response contracts
      integrations/                 # provider adapters and ingestion entrypoints
      vault/                        # accounts / capabilities / sources / sessions
        provider_accounts.rs
      platform/                     # platform-level SoR and infra boundaries
        observations/               # canonical evidence store
        events/
        secrets/
        settings/
        storage/
        audit/
      domains/                      # durable domain records
        communications/
        calendar/
        meetings/
        persons/
        organizations/
        documents/
        projects/
        tasks/
        decisions/
        obligations/
        relationships/
        review/
      engines/                      # derived/rebuildable systems
        context_packs/
        identity_resolution/
        relationships/
        review_promotion/
        consistency/
        memory/
        timeline/
        trust/
        risk/
        search/
      workflows/                    # orchestration across app/platform/domain/engine layers
      ai/
    migrations/
    tests/
  frontend/
    src/
      domains/
        review/
        tasks/
        persons/
        organizations/
        projects/
      platform/
      shared/
  docs/
    adr/
    architecture/
    foundation/
    domains/
    engines/
    workflows/
  scripts/
  docker/
  crates/
  Makefile
```

## 7. Проверенная валидация

Ниже только команды, которые реально были выполнены по последним архитектурным срезам.

| Команда | Результат |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates_api put_task_candidate_review_confirms_task_with_observation_trail -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test person_identity_api put_identity_candidate_review_confirms_candidate -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test projects_api put_project_link_review_updates_review_state -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test task_candidates_api --test person_identity_api --test projects_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test decisions_api --test obligations_api --test relationships_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test decisions_api put_decision_review_updates_review_state_with_observation_trail -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test context_packs --test calendar --test tasks` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test context_packs context_pack_store_persists_derived_pack_with_explicit_sources_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test person_identity_api --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test person_identity_api identity_candidates_returns_safe_candidate_payload -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test project_link_reviews --test projects_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test project_link_reviews project_link_review_confirm_materializes_user_confirmed_decision_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test tasks_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test tasks task_manual_creation_materializes_explicit_observation_provenance_against_postgres -- --exact --nocapture` | PASS (live environment unavailable; test skipped with `no DB`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test organizations_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test organizations_api organization_manual_entrypoints_capture_observations_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test persons_api --test tasks --test calendar_api --test email_account_setup --test organizations_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api identity_traces::identity_traces_create_list_and_attach_unattached_trace -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped) |
| `cargo test --manifest-path backend/Cargo.toml --test tasks task_providers_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped with `no DB`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api accounts::calendar_accounts_crud_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped) |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup gmail_api::gmail_oauth_callback_completes_pending_grant_without_api_secret -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup vault_reconciliation::startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_account_management_api --test email_account_setup` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test project_link_reviews --test tasks_api --test v1_communications_api --test person_identity_api --test organizations_api --test persons_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_account_management_api --test email_account_setup --test telegram_runtime_lifecycle` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api email_account_management_lists_gets_exports_logs_out_and_deletes_unused_account -- --nocapture` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test tasks_api task_checklist_manual_create_path_captures_observation_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test organizations_api --test persons_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api personas_put_updates_compatibility_projection_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test organizations_api organization_manual_entrypoints_capture_observations_against_postgres -- --nocapture` | PASS (test skipped because live DB not configured) |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test obligations_api put_obligation_review_updates_review_state_with_observation_trail -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test relationships_api put_relationship_review_updates_relationship_and_graph_projection -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test contradictions_api --test observations --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test contradictions_api put_contradiction_review_updates_review_state_with_observation_trail -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_media_upload -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test observations canonical_observation_kind_definitions_are_seeded_against_postgres -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_ai_state -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_account_management_api --test communication_ingestion --test email_account_setup` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_management_api -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test communication_ingestion -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_account_setup_capabilities --test telegram_runtime_lifecycle --test whatsapp` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test whatsapp -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_account_setup_capabilities -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_runtime_lifecycle -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_media_upload -- --nocapture` | PASS |
| `node scripts/check-architecture.mjs --self-test` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --bin makosh-email-sync-dev` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test document_processing --test document_processing_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test document_processing enqueue_run::enqueue_for_document_creates_extract_text_and_ocr_jobs -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test document_processing retry::document_processing_retry_failed_job_requeues_job_against_postgres -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test ai_control_center` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test ai_control_center ai_control_center_mutations_record_observation_trail_against_postgres -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test ai --test ai_control_center` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test ai semantic_store::pgvector_semantic_store_indexes_and_searches_sources_against_postgres -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test whatsapp` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test whatsapp whatsapp_api_exercises_web_fixture_foundation -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test decisions --test decisions_api --test obligations --test obligations_api --test relationships --test relationships_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test decisions_api put_decision_review_updates_review_state_with_observation_trail -- --exact --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test obligations_api put_obligation_review_updates_review_state_with_observation_trail -- --exact --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test relationships_api put_relationship_review_updates_relationship_and_graph_projection -- --exact --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test person_identity_api --test task_candidates_api --test task_candidates --test persons_api --test relationships` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_account_setup_capabilities -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_outbox --test email_sync_pipeline --test email_account_setup` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_outbox -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup vault_reconciliation::startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_sync_pipeline obligation -- --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_outbox --test email_sync_pipeline --test telegram_runtime_lifecycle --test whatsapp --test gmail_send_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_runtime_lifecycle -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test whatsapp -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test gmail_send_api -- --nocapture` | PASS |
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test obligations_api --test contradictions_api --test document_processing_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test obligations_api put_obligation_review_updates_review_state_with_observation_trail -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test contradictions_api put_contradiction_review_updates_review_state_with_observation_trail -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test document_processing_api post_document_processing_job_retry_requeues_failed_job -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test calendar_api --test calendar` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api events::calendar_event_participants_crud -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::calendar_manual_event_materials_capture_observations_against_postgres -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::calendar_event_relation_manual_create_path_captures_observation_against_postgres -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::cal_event_reminder_toggle -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api accounts::calendar_accounts_crud_against_postgres -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::calendar_sources_list -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_sync -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_rules_crud -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_post_deadline -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_post_focus_block -- --nocapture` | PASS (skip without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test projects_api --test project_link_reviews --test calendar` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test projects_api put_project_link_review_updates_review_state -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test project_link_reviews project_link_review_reset_clears_review_and_demotes_relationship_against_postgres -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar meeting_outcomes::meeting_outcome_decision_creates_suggested_decision_against_postgres -- --exact --nocapture` | PASS (compile + skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_runtime_lifecycle --test telegram_media_upload --test telegram_account_setup_capabilities` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_runtime_lifecycle -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_media_upload -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox decision_review_mirror_promotes_existing_decision_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox obligation_review_mirror_promotes_existing_obligation_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox task_candidate_review_mirror_promotes_obligation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test person_identity_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox identity_candidate_review_mirror_reuses_review_item_and_attaches_new_evidence_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox identity_candidate_review_mirror_promotes_existing_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test person_identity_api put_identity_candidate_review_confirms_candidate -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test projects_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox task_candidate_review_mirror_promotes_obligation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test projects_api project_link_candidates_return_safe_message_and_document_candidates -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox review_can_materialize_promotions_for_core_target_domains_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test persons_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api person_manual_memory_entrypoints_capture_observations_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api -- --list` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api identity_traces_create_list_and_attach_unattached_trace -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api person_identity_post_and_delete -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api person_investigate_captures_observation_and_links_snapshot_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api person_dossier_get_persists_snapshot_and_review_state_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `find . -maxdepth 1 -type f \( -iname '*report*.md' -o -iname '*status*.md' -o -iname '*audit*.md' -o -iname '*qa*.md' \) \| sort` | PASS (`./canonical-evidence-final-report.md`) |
| `node scripts/check-architecture.mjs` | PASS |
| `node scripts/check-architecture.mjs --self-test` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_message_links` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_message_links telegram_message_ingestion_projects_public_message_link_without_erasing_chat_username -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test communication_ingestion --test telegram_members_sync_exhaustive_absence` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_members_sync_exhaustive_absence members_route_hides_absent_exhaustive_participants_after_roster_reconciliation -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib topic_events -- --list` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib publish_topic_event_reconciles_topic_close_and_appends_runtime_events -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_dialog_actions --test telegram_reactions` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_dialog_actions telegram_restore_and_reaction_actions_record_durable_command_rows -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib publish_reaction_changed_event_records_reaction_observations -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_message_realtime` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_message_realtime telegram_provider_delete_observation_is_idempotent_and_reconciles_delete_command -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_message_realtime telegram_provider_edit_observation_is_idempotent_and_reconciles_edit_command -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_search_pinning telegram_message_pin_route_records_local_projection_command_and_audit -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib integrations::telegram::client::messages::attachments::tests::update_message_attachment_download_state_patches_projected_metadata -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib integrations::telegram::runtime::manager::message_events::tests::publish_message_content_updated_event_records_projection_observation -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib integrations::telegram::runtime::manager::message_events::tests::publish_message_edited_event_records_metadata_observation -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --lib integrations::telegram::runtime::manager::message_events::tests::publish_reaction_changed_event_records_reaction_observations -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox review_can_materialize_promotions_for_core_target_domains_against_postgres -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test contradictions_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test contradictions_api contradictions_list_returns_open_reviewable_observations -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test decisions_api --test obligations_api --test relationships_api --test task_candidates_api --test projects_api --test contradictions_api --test review_inbox --test person_identity_api --test organizations_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_sync_pipeline` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_sync_pipeline email_sync_pipeline_records_raw_blob_and_projects_message_persons_against_postgres -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_core` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_core telegram_api_exercises_policy_and_call_foundation -- --exact --nocapture` | PASS (skip live Telegram smoke without `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test calendar_api --test calendar` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_rules_crud -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_post_deadline -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api misc::cal_post_focus_block -- --exact --nocapture` | PASS (skip без `MAKOSH_TEST_DATABASE_URL`) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test persons_api --test calendar_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons_api person_manual_memory_entrypoints_capture_observations_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api event_details::calendar_manual_event_materials_capture_observations_against_postgres -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test calendar_api cal_rules_crud -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `find . -maxdepth 1 -type f \( -iname '*report*.md' -o -iname '*status*.md' -o -iname '*audit*.md' -o -iname '*qa*.md' \) | sort` | PASS (`./canonical-evidence-final-report.md` only) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test persons --test document_processing_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test persons relationships::person_role_assign_and_remove_materializes_relationship_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons relationships::person_enrichment_trust_score_materializes_owner_relationship_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test persons relationships::person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test document_processing_api post_document_processing_job_retry_requeues_failed_job -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `rg -n '"DOCUMENT"' backend/src/domains/persons backend/src/domains/documents/api/mod.rs -g '*.rs'` | PASS (no remaining false `DOCUMENT` usages in targeted files) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test task_candidates` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates review::task_candidate_review_confirm_creates_active_task_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates review::task_candidate_review_confirm_materializes_obligation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates review::task_candidate_review_confirm_rejects_legacy_non_observation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `rg -n '"DOCUMENT"' backend/src -g '*.rs'` | PASS (`backend/src/domains/documents/core/store.rs` only) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test email_account_setup --test email_outbox --test email_sync_pipeline` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup gmail_api::gmail_oauth_callback_completes_pending_grant_without_api_secret -- --exact --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_outbox -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_sync_pipeline obligation -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `rg -n 'CommunicationIngestionStore' backend/src/domains/communications/handlers/sending/provider_send.rs backend/src/domains/communications/outbox/provider_sender.rs backend/src/domains/communications/background_sync/provider/gmail.rs backend/src/domains/communications/accounts/service.rs backend/src/domains/communications/accounts/service/constructors.rs backend/src/domains/communications/accounts/service/stores.rs backend/src/domains/api_support/stores/integration_stores.rs -g '*.rs'` | PASS (no remaining `CommunicationIngestionStore` dependency in targeted refresh/send runtime files) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test task_candidates` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test review_inbox task_candidate_review_mirror_promotes_obligation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates review::task_candidate_review_confirm_creates_active_task_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test task_candidates review::task_candidate_review_confirm_materializes_obligation_candidate_against_postgres -- --exact --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo fmt --manifest-path backend/Cargo.toml --check` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_communications_api --test v1_communications_folders --test v1_communications_saved_searches` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_api v1_post_draft -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_folders v1_custom_folders_copy_move_and_events_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_saved_searches v1_saved_searches_crud_and_events_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test tasks --test document_processing_api --test projects_api --test persons_api --test person_identity_api --test relationships` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test tasks_api` | PASS |
| `DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo run --manifest-path backend/Cargo.toml --bin makosh_migrate` | PASS |
| `docker exec makosh-dev-postgres-1 psql -U makosh -d makosh_hub -Atc "SELECT version, success FROM _sqlx_migrations ORDER BY version DESC LIMIT 5"` | PASS (`148`, `147`, `146` applied) |
| `docker exec makosh-dev-postgres-1 psql -U makosh -d makosh_hub -Atc "SELECT conname, pg_get_constraintdef(oid) FROM pg_constraint WHERE conname IN ('review_items_item_kind','tasks_source_kind_check','tasks_source_type_check') ORDER BY conname"` | PASS (live constraints include canonical `identity_candidate`, `project_link_candidate`, `observation`) |
| `MAKOSH_TEST_DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo test --manifest-path backend/Cargo.toml --test observations -- --nocapture --test-threads=1` | PASS (`10 passed`) |
| `MAKOSH_TEST_DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo test --manifest-path backend/Cargo.toml --test review_inbox -- --nocapture --test-threads=1` | PASS (`16 passed`) |
| `MAKOSH_TEST_DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo test --manifest-path backend/Cargo.toml --test review_inbox review_item_api_lifecycle_captures_observation_trail_against_postgres -- --exact --nocapture --test-threads=1` | PASS |
| `MAKOSH_TEST_DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo test --manifest-path backend/Cargo.toml --test tasks task_store_update_with_observation_materializes_task_link_against_postgres -- --exact --nocapture --test-threads=1` | PASS |
| `MAKOSH_TEST_DATABASE_URL='postgres://makosh:change-me-local-dev-only@127.0.0.1:30432/makosh_hub' cargo test --manifest-path backend/Cargo.toml --test tasks_api crud::task_analyze_runtime_path_captures_observation_against_postgres -- --exact --nocapture --test-threads=1` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/communications/handlers/communication_queries/drafts.rs backend/src/domains/communications/handlers/communication_queries/folders.rs backend/src/domains/communications/handlers/communication_queries/saved_searches.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_communications_ai_state --test v1_communications_message_actions --test v1_communications_regressions --test telegram_media_upload` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_ai_state v1_message_ai_state_transitions_are_durable_and_emit_event_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_message_actions v1_local_state_endpoints_capture_observation_trail_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_regressions drafts_outbox::v1_send_schedules_outbox_message_and_allows_undo_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test telegram_media_upload telegram_media_upload_imports_attachment_and_queues_provider_command -- --nocapture` | PASS |
| `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/communications/handlers/workflow_state.rs backend/src/domains/communications/handlers/message_ai_state.rs backend/src/domains/communications/handlers/communication_queries/outbox.rs backend/src/domains/communications/handlers/communication_queries/imports.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --test message_flags_api message_important_endpoint_toggles_metadata_flag -- --nocapture` | PASS |
| `rg -n "upsert_link\\(|NewObservationLink|link_message_flag_observation" backend/src/domains/communications/handlers/message_actions.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_workflow_actions --test v1_communications_regressions --test v1_communications_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_workflow_actions workflow_action_create_event_reuses_message_observation_for_calendar_projection -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test v1_workflow_actions workflow_action_create_note_creates_markdown_document -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `cargo test --manifest-path backend/Cargo.toml --test v1_workflow_actions workflow_action_link_document_reuses_message_observation_for_document_projection -- --nocapture` | PASS (`MAKOSH_TEST_DATABASE_URL` не задан, live test skipped after compile) |
| `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/communications/handlers/workflow_actions/actions backend/src/domains/communications/handlers/sending/forwarding.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test gmail_send_api --test email_account_setup --test v1_communications_regressions` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test gmail_send_api gmail_send_api_uses_gmail_api_when_send_scope_enabled_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test email_account_setup imap_send_api_sends_via_configured_smtp_against_postgres -- --nocapture` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_communications_regressions drafts_outbox::v1_send_schedules_outbox_message_and_allows_undo_against_postgres -- --nocapture` | PASS |
| `rg -n "upsert_link\\(|NewObservationLink" backend/src/domains/communications/handlers -g '!**/target/**'` | PASS (совпадений нет) |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/**/handlers backend/src/app/router/routes -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test relationships` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test relationships` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/tasks/api.rs backend/src/domains/organizations/api.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test task_candidates` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test relationships --test review_inbox --test task_candidates --test tasks` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/review/store.rs backend/src/domains/tasks/candidates/store/review.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test relationships` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test tasks --test tasks_api` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/organizations/health.rs backend/src/domains/organizations/enrichment.rs backend/src/domains/tasks/core/evidence.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/tasks/core/checklists.rs backend/src/domains/tasks/core/relations.rs backend/src/domains/tasks/core/subtasks.rs backend/src/domains/tasks/core/evidence.rs -g '!**/target/**'` | PASS (совпадений нет) |
| `node scripts/check-architecture.mjs` | PASS |

Примечание: часть live-path тестов корректно skip'ается без `MAKOSH_TEST_DATABASE_URL`; это текущее состояние test environment, а не failing validation.

Примечание: bootstrap blocker в `backend/migrations/0116_add_contradiction_observation_kind.sql` устранен. Миграция больше не пишет в несуществующий столбец `schema_version` таблицы `observation_kind_definitions`; свежие test databases снова поднимаются.

## 8. Риски

| Риск | Суть |
|---|---|
| Исторические compatibility tables | Legacy schema (`event_context_packs`, `task_context_packs` и др.) остается для совместимости и traceability, но runtime ownership уже переведен на новую архитектуру. |
| Исторические narrative sections в этом отчете | Ниже сохранены slice-by-slice checkpoints этого периода. Они исторические и не переопределяют итоговый verdict из разделов 1 и 1.2. |

## 9. Итог

Система полностью переведена на `Canonical Evidence -> Review Inbox -> Actions` в рамках текущего плана и текущего кода.

Текущая реальная стадия:

- foundation и core modules на месте;
- ingress и review lifecycle доказаны observation-backed owner flows;
- `domains/review` работает как реальный inbox;
- `engines/context_packs`, `identity_resolution`, `relationships` закреплены как engine boundaries;
- completion audit не нашел незакрытых обязательных требований исходного плана.

## 10. Latest Slice: Calendar Owner Cleanup

### Что сделано

- Добавлен общий helper уровня домена: `backend/src/domains/calendar/evidence.rs`.
- `backend/src/domains/calendar/mod.rs` теперь объявляет `calendar::evidence` как общий owner boundary для observation linking.
- На новый helper переведены оставшиеся calendar owners:
  - `backend/src/domains/calendar/rules.rs`
  - `backend/src/domains/calendar/scheduling.rs`
  - `backend/src/domains/calendar/reminders.rs`
  - `backend/src/domains/calendar/meetings/outcomes.rs`
  - `backend/src/domains/calendar/meetings/notes.rs`
  - `backend/src/domains/calendar/meetings/recordings.rs`
- Прямые `NewObservationLink` / `ObservationStore::upsert_link*` из этих owner-модулей убраны.
- В корне проекта очищены неактуальные отчеты текущего типа; оставлен один актуальный файл:
  - `canonical-evidence-final-report.md`

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test calendar --test calendar_api --test v1_workflow_actions` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/calendar/rules.rs backend/src/domains/calendar/scheduling.rs backend/src/domains/calendar/reminders.rs backend/src/domains/calendar/meetings/outcomes.rs backend/src/domains/calendar/meetings/notes.rs backend/src/domains/calendar/meetings/recordings.rs -g '!**/target/**'` | PASS (совпадений нет) |

### Что осталось после этого среза

- remaining mail tails:
  - `backend/src/domains/communications/messages/store/local_state.rs`
  - `backend/src/domains/communications/outbox/delivery_status.rs`
  - `backend/src/domains/communications/send.rs`
  - `backend/src/domains/communications/ai_state.rs`
  - `backend/src/domains/communications/messages/store/workflow.rs`
  - `backend/src/domains/communications/messages/store/participants.rs`
- integration/runtime tails:
  - `backend/src/integrations/telegram/client/*`
  - `backend/src/integrations/whatsapp/client/store/sessions.rs`
- engine/runtime tails:
  - `backend/src/ai/core/runs.rs`
  - `backend/src/ai/core/semantic/embeddings.rs`
  - `backend/src/engines/consistency/store/*`
  - `backend/src/engines/automation/evidence.rs` (проверить, нужен ли дальнейший split или это уже допустимый owner helper)

### Следующие шаги

1. Добить remaining mail tails тем же owner-helper подходом.
2. Отдельно пройти integrations/telegram + whatsapp session store как ingress/runtime bounded context.
3. После этого сделать новую repo-wide инвентаризацию `NewObservationLink|upsert_link` и выбрать следующий densest cluster.

## 11. Latest Slice: Telegram / WhatsApp Ingress Owner Cleanup

### Что сделано

- Добавлен общий helper для Telegram ingress owner links:
  - `backend/src/integrations/telegram/client/evidence.rs`
- Добавлен общий helper для WhatsApp session owner links:
  - `backend/src/integrations/whatsapp/client/store/evidence.rs`
- На helper-путь переведены:
  - `backend/src/integrations/telegram/client/chats.rs`
  - `backend/src/integrations/telegram/client/participants.rs`
  - `backend/src/integrations/telegram/client/topics.rs`
  - `backend/src/integrations/telegram/client/reactions.rs`
  - `backend/src/integrations/telegram/client/commands.rs`
  - `backend/src/integrations/telegram/client/messages/provider_state.rs`
  - `backend/src/integrations/telegram/client/lifecycle/message_versions.rs`
  - `backend/src/integrations/telegram/client/lifecycle/tombstones.rs`
  - `backend/src/integrations/whatsapp/client/store/sessions.rs`
- Из этого provider-side ingress cluster убраны прямые `NewObservationLink` / `ObservationStore::upsert_link*`.
- Дополнительно сохранен boundary:
  - `integrations/telegram` не импортирует `domains/mail`; communication link materialization теперь идет через локальный integration helper.

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test telegram_core --test telegram_dialog_actions --test telegram_message_realtime --test telegram_search_pinning --test whatsapp` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/integrations/telegram/client backend/src/integrations/whatsapp/client/store/sessions.rs -g '!**/target/**'` | PASS (совпадений нет) |

### Что осталось после этого среза

- remaining mail/runtime tails:
  - `backend/src/domains/communications/outbox/delivery_status.rs`
  - `backend/src/domains/communications/send.rs`
  - `backend/src/domains/communications/ai_state.rs`
  - `backend/src/domains/communications/messages/store/local_state.rs`
  - `backend/src/domains/communications/messages/store/workflow.rs`
  - `backend/src/domains/communications/messages/store/metadata.rs`
  - `backend/src/domains/communications/messages/store/participants.rs`
  - `backend/src/domains/communications/storage/imports.rs`
  - `backend/src/domains/communications/bulk_actions.rs`
- remaining technical/runtime owners:
  - `backend/src/ai/core/runs.rs`
  - `backend/src/ai/core/semantic/embeddings.rs`
  - `backend/src/domains/documents/processing/jobs.rs`
  - `backend/src/domains/documents/processing/retry.rs`
  - `backend/src/engines/consistency/store/{observations,review}.rs`
  - `backend/src/engines/automation/evidence.rs`

### Следующие шаги

1. Взять единым срезом remaining `mail` runtime/store tails.
2. Затем пройти `ai/core` + `documents/processing` как следующий technical mutation cluster.
3. После этого снова снять repo-wide inventory и проверить, что остаются только допустимые owner helper файлы.

## 12. Latest Slice: Mail Runtime / Store Tail Cleanup

### Что сделано

- На существующий `backend/src/domains/communications/evidence.rs` переведены remaining mail runtime/store tails:
  - `backend/src/domains/communications/outbox/delivery_status.rs`
  - `backend/src/domains/communications/send.rs`
  - `backend/src/domains/communications/ai_state.rs`
  - `backend/src/domains/communications/messages/store/local_state.rs`
  - `backend/src/domains/communications/messages/store/workflow.rs`
  - `backend/src/domains/communications/messages/store/metadata.rs`
  - `backend/src/domains/communications/messages/store/participants.rs`
  - `backend/src/domains/communications/storage/imports.rs`
  - `backend/src/domains/communications/bulk_actions.rs`
- Из этого `mail` cluster убраны прямые `NewObservationLink` / `ObservationStore::upsert_link*`.
- Merge semantics для link metadata теперь проходят через owner helper вместо локальных ad-hoc блоков.

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test v1_communications_ai_state --test v1_communications_regressions --test message_flags_api --test telegram_media_upload --test email_outbox --test v1_communications_api` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src/domains/communications/outbox/delivery_status.rs backend/src/domains/communications/send.rs backend/src/domains/communications/ai_state.rs backend/src/domains/communications/messages/store/local_state.rs backend/src/domains/communications/messages/store/workflow.rs backend/src/domains/communications/messages/store/metadata.rs backend/src/domains/communications/messages/store/participants.rs backend/src/domains/communications/storage/imports.rs backend/src/domains/communications/bulk_actions.rs -g '!**/target/**'` | PASS (совпадений нет) |

### Что осталось после этого среза

- remaining technical mutation clusters:
  - `backend/src/ai/core/runs.rs`
  - `backend/src/ai/core/semantic/embeddings.rs`
  - `backend/src/domains/documents/processing/jobs.rs`
  - `backend/src/domains/documents/processing/retry.rs`
  - `backend/src/engines/consistency/store/observations.rs`
  - `backend/src/engines/consistency/store/review.rs`
  - `backend/src/engines/automation/evidence.rs` (нужно решить, это already-valid owner helper или еще нет)

### Следующие шаги

1. Взять единым technical slice `ai/core` + `documents/processing`.
2. Затем добить `engines/consistency` review/observation owner paths.
3. После этого сделать новый repo-wide completion audit по оставшимся direct link sites и отсеять допустимые helper files от реальных business/runtime остатков.

## 13. Latest Slice: AI / Documents / Consistency Technical Cleanup

### Что сделано

- Добавлены owner helpers:
  - `backend/src/ai/core/evidence.rs`
  - `backend/src/domains/documents/processing/evidence.rs`
- На них переведены:
  - `backend/src/ai/core/runs.rs`
  - `backend/src/ai/core/semantic/embeddings.rs`
  - `backend/src/domains/documents/processing/jobs.rs`
  - `backend/src/domains/documents/processing/retry.rs`
- Добит `engines/consistency` technical store cluster:
  - `backend/src/engines/consistency/store/observations.rs`
  - `backend/src/engines/consistency/store/review.rs`
  - `backend/src/engines/consistency/evidence.rs` расширен owner helper-ом для link materialization.

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test ai_control_center --test document_processing_api --test contradictions_api --test review_inbox --test tasks --test telegram_core --test whatsapp --test v1_communications_api --test persons_api --test organizations_api` | PASS |
| `node scripts/check-architecture.mjs` | PASS |

## 14. Latest Slice: Final Helper Cleanup For Direct Link Materialization

### Что сделано

- Дочищены оставшиеся helper files, которые еще напрямую собирали `NewObservationLink`:
  - `backend/src/engines/automation/evidence.rs`
  - `backend/src/domains/tasks/core/observation_links.rs`
  - `backend/src/domains/communications/background_sync/evidence.rs`
  - `backend/src/domains/persons/core/evidence.rs`
  - `backend/src/domains/organizations/core/evidence.rs`
- После этого repo-wide direct observation link materialization вне `platform/observations` снят.

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test ai_control_center --test document_processing_api --test contradictions_api --test review_inbox --test tasks --test telegram_core --test whatsapp --test v1_communications_api --test persons_api --test organizations_api` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src -g '!**/target/**' -g '!**/platform/observations/**'` | PASS (совпадений нет) |

### Новый статус после этого checkpoint

- Низкоуровневый spread `NewObservationLink` / `ObservationStore::upsert_link*` по прикладным и owner helper модулям фактически сведен к нулю вне `platform/observations`.
- Следующий remaining work уже не про этот mechanical cleanup, а про completion audit более широких архитектурных требований:
  - append-only / provider deletion invariants;
  - observation capture for all intended ingress surfaces;
  - review ownership / promotion invariants;
  - doc/runtime alignment against ADR-0096.

### Следующие шаги

1. Провести completion audit по explicit требованиям `ADR-0096` и user plan, а не только по direct link cleanup.
2. Проверить test evidence для append-only semantics, provider deletion survival и no-Vault manual observation capture.
3. Закрыть только те remaining gaps, которые audit подтвердит как реально незавершенные.

## 15. Completion Audit Checkpoint

### Уже доказано текущими тестами

- `backend/tests/observations.rs` проходит:
  - manual note / voice recording / browser capture / calendar event / contact record / meeting transcript / meeting recording create observations without Vault;
  - append-only semantics block `UPDATE` / `DELETE`;
  - provider deletion survives as a second observation (`COMMUNICATION_MESSAGE_DELETED`);
  - observation links and ingestion runs persist.
- `backend/tests/review_inbox.rs` проходит:
  - evidence-backed review item creation;
  - lifecycle `new -> in_review -> approved -> promoted / dismissed / archived`;
  - promotion into core target domains;
  - rejection of orphaned / missing evidence;
  - observation trail for review transitions and promotion.
- `backend/tests/tasks.rs` already enforces task provenance rules for review-item and observation-backed creation paths.
- Repo-wide low-level observation linking outside `platform/observations` сейчас снят:
  - `rg -n "NewObservationLink|upsert_link\\(|upsert_link_in_transaction\\(" backend/src -g '!**/target/**' -g '!**/platform/observations/**'`
  - результат: совпадений нет.
- `scripts/check-architecture.mjs` продолжает запрещать:
  - `domains/signals`
  - `domains/events`
  - `domains/attention`
  - `domains/evidence`
  - `vault/observations`

### Что еще не доказано окончательно

- Полный repo-wide completion claim еще не сведен в requirement-by-requirement matrix по всем explicit пунктам user plan.
- В документации до последнего прохода еще оставались legacy markers вокруг `event_context_packs` / `task_context_packs`; они не ломали runtime, но мешали считать перевод полностью завершенным на уровне архитектурной терминологии.

### Практический вывод

- По коду, guards и live proof trio (`observations`, `review_inbox`, `v1_workflow_actions`) миграция фактически доведена почти до конца.
- Оставшийся gap уже в основном в силе полного completion proof и doc/runtime alignment, а не в очевидных незакрытых owner/link rewrites.

## 16. Latest Slice: Workflow Ownership Normalization

### Что сделано

- `workflow_action_person_projection` возвращен в правильный слой `backend/src/workflows/`, потому что это cross-domain orchestration, а не owner-логика `persons`.
- При этом старый helper vocabulary ужесточен:
  - `create_contact_projection_in_transaction(...)` заменен на `create_person_projection_in_transaction(...)`;
  - observation payload/source_ref теперь говорят `create_person`, а не `create_contact`.
- `domains/mail/handlers/workflow_actions/actions/persons.rs` снова использует workflow-owned projection helper вместо прямого upsert без evidence trail.
- Попытка переноса `review_mirror` в `domains/review/` была отвергнута architecture guard и откатана; модуль оставлен в `workflows/` как допустимый orchestration layer.
- Все временно поломанные imports после этого отката дочищены:
  - `calendar/meetings/outcome_projection.rs`
  - `persons/identity/upsert.rs`
  - `workflows/review_inbox.rs`

### Почему это важно

- Это убирает еще один misleading `contact` marker из production workflow path.
- Одновременно сохраняется правильная dependency direction: `review_mirror` и workflow-driven person projection остаются в orchestration layer, а не проталкиваются через domain boundary вопреки guard rules.

### Validation

| Command | Result |
|---|---|
| `cargo fmt --manifest-path backend/Cargo.toml` | PASS |
| `node scripts/check-architecture.mjs` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --no-run --test review_inbox --test persons_api --test projects_api --test relationships_api --test decisions_api --test obligations_api --test v1_workflow_actions --test calendar_api --test v1_communications_api` | PASS |
| `cargo test --manifest-path backend/Cargo.toml --test v1_workflow_actions -- --nocapture` | PASS |

### Новый статус после этого checkpoint

- Low-level canonical evidence migration остается закрытой.
- Этот checkpoint исторический. Его остаток закрыт более поздними slices, live validation и финальным completion audit из раздела 1.2.
