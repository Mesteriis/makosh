# ADR-0282: Полное восстановление возможностей Communications и Settings

Статус: Принято
Дата: 2026-07-26
Состояние реализации: завершено. Exact inventory содержит 91 из 91 независимо
доказанных `implemented` slices. Последний `call_transcription_v1` закрыт
managed real-audio, outage/reconnect, authenticated Gateway/SSE/ClientBlob и
restart conformance. Полный root `make pre-push` является финальным evidence
gate этой ревизии. Aggregate marker
`communications_settings_reconstruction_complete_v1` не становится generic
production capability: каждый runtime и capability остаётся отдельно admitted,
revocable и owner-bound.

Зависит от:

- [ADR-0201: Core communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: integration boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0213: ownership and SRP](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0214: Scheduler](ADR-0214-durable-job-platform-scheduler-and-runtime-reconfiguration.md);
- [ADR-0222: Settings Registry](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0226: use-case-specific AI workflows](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0253: Communications legacy disposition](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0265: provider operational client transport](ADR-0265-provider-operational-client-transport-admission.md);
- [ADR-0281: frontend clean-room composition](ADR-0281-communications-frontend-clean-room-composition.md);
- [ADR-0287: Telegram operational realtime](ADR-0287-telegram-operational-realtime-replay-boundary.md).
- [ADR-0353: reply suggestion and AI inference boundary](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md).
- [ADR-0390: call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md).
- [ADR-0391: Whisper STT provider integration](ADR-0391-whisper-stt-provider-integration.md).
- [ADR-0394: desktop call recording host and consent authority](ADR-0394-desktop-call-recording-host-capture-and-consent-authority.md).

## Контекст

ADR-0281 завершил архитектурный cutover frontend: удалил mixed
Communications/Settings controllers, handwritten REST, общий provider state и
разместил canonical Communications, Mail, Telegram, WhatsApp, Zulip и Settings
в правильных frontend owners. Это было завершение transport/ownership cutover,
но не восстановление всех полезных возможностей прежнего продукта.

После cutover отсутствуют или представлены только минимальными read-only
surfaces:

- provider-specific folders, threads, drafts, compose, templates, signatures,
  account lifecycle и operational actions Mail;
- полный Telegram operational client, хотя большая часть backend contract уже
  существует;
- WhatsApp operational queries и history, хотя typed query messages уже
  существуют внутри integration contract;
- Zulip streams, topics, direct messages, history, search и lifecycle;
- canonical content read, saved searches и более полная evidence navigation;
- Calls, recordings и transcript use cases;
- AI Reply, summary, translation, explanation и extraction use cases;
- Review actions, outbox, delayed delivery и bulk/cross-channel orchestration;
- application, platform, maintenance, telemetry, scheduler, AI и integration
  settings surfaces.

Возврат прежних файлов или routes не является переносом. Historical source
может быть только behavioral evidence. Wire contracts, ownership, persistence,
runtime, errors, privacy и tests создаются заново по clean-room правилам.

## Цель

Восстановить все подтверждённые полезные Communications и Settings use cases,
сохранив:

1. Communications как provider-neutral owner canonical evidence;
2. Mail, Telegram, WhatsApp и Zulip как четыре независимые integrations;
3. Core как capability routing, event spine и client transport;
4. Kernel как owner-neutral admission, lifecycle, fencing и settings control
   plane, но не business mediator;
5. explicit workflows для cross-owner orchestration и AI use cases;
6. Settings как app-level composition владельцев, а не business domain;
7. отдельную единицу сборки, admission, storage и failure boundary для каждого
   owner или workflow;
8. SRP по ответственности и причине изменения.

## Не-цели

- восстановление legacy REST, aliases, proxy, dual-read или dual-write;
- копирование provider payload, arbitrary metadata maps или raw DTO unions в
  Communications;
- превращение Kernel, Gateway, Settings Registry или app composition в
  business facade;
- объявление frontend screen доказательством backend/runtime readiness;
- имитация функций, которые в historical UI были только demo/facade;
- создание Slack, Discord, Mattermost, Zoom, Google Meet, Microsoft Teams или
  Telemost integrations без отдельного owner admission и реальной provider
  реализации;
- возврат общего `Channels` provider switch или общего provider store внутрь
  Communications domain.

## Решение

### Business data не проходит через Kernel

Kernel согласует registration, exact executable, descriptor, GrantSet,
runtime generation, settings revision и route admission. После admission:

```text
client
  -> Core Gateway
  -> owner public contract
  -> owner runtime

integration outbox
  -> NATS JetStream
  -> target owner inbox
  -> target owner mutation
```

Kernel не декодирует provider или Communications payload, не выбирает
business target и не вызывает domain handler. Integration не общается с
Communications implementation или storage. Она использует Core data plane и
публикует typed durable event. Communications принимает его через собственный
inbox и создаёт canonical evidence.

Cross-owner command выполняет explicit workflow:

```text
source owner query/event
  -> use-case workflow
  -> target owner command
  -> target owner result/event
```

### Владельцы

| Capability family | Owner | Граница |
|---|---|---|
| Canonical accounts, conversations, messages, participants, references, attachment anchors and evidence search | `communications` domain | Только provider-neutral evidence и provenance |
| Mail folders, threads, drafts, compose, templates, signatures, mailbox lifecycle, sync and provider delivery | `mail` integration | Mail-specific operational truth |
| Telegram authorization, chats, topics, folders, history, search, media, mutations, automation and calls | `telegram` integration | Telegram operational truth; runtime reconfiguration, folder reassignment, automation and calls remain separate capability slices |
| WhatsApp host lifecycle, dialogs, history, search, media, status and mutations | `whatsapp` integration | WhatsApp operational truth; execution remains host-only |
| Zulip authorization, streams, topics, direct messages, history, search, files and mutations | `zulip` integration | Zulip operational truth |
| Cross-provider delivery intent, delayed send, undo/outbox orchestration and bulk/cross-channel action | distinct use-case workflow | Не хранит provider operational truth |
| Reply suggestion, summary, translation, explanation and extraction | one workflow per AI use case plus `ai` public contract | AI output is candidate, never canonical truth |
| Pin, snooze, mute, important, review state and promotion | `review` domain after separate admission | Не provider label/read/archive state |
| App locale, theme, layout and navigation preferences | first-party client/app owner | Не module settings |
| Generic desired/effective settings revisions | Kernel Settings Registry | Не интерпретирует owner fields |
| Provider setup and settings semantics | owning integration | Secrets remain Vault leases; sessions/cursors are not settings |
| Job schedules and run controls | `scheduler` platform | Job implementation remains at owner |
| Logs, traces and diagnostics | `telemetry` platform | Sanitized diagnostics only |
| Backup, restore and maintenance controls | owning Kernel/Storage/Vault/Blob components | App only composes public controls |
| AI provider/model routes and consent | `ai` owner and admitted provider adapters | No generic cross-owner context API |
| Signal Hub source/runtime view | app composition over Kernel bootstrap, integration panels, Telemetry and Review | No new hidden business owner |

### Canonical Communications

Communications owns:

- canonical account, conversation and message identity;
- bounded authorized content read with provenance and content classification;
- participants, references and attachment anchors;
- evidence history and source revisions;
- provider-neutral search and saved search definitions;
- call evidence as a communication kind when a provider emits an admitted
  typed call observation;
- references to recordings/transcripts without owning provider execution or
  Blob bytes.

Communications does not own:

- provider folders, labels, read/archive state or drafts;
- provider send/reply/forward/delete execution;
- provider authorization, sync cursors, health or session state;
- AI-generated answers, summaries or extracted business entities;
- Review workflow state;
- recording/transcription execution.

Message content admission must define bounded response sizes, content
classification, sanitization, authorization, attachment separation and
negative-output privacy. Search indexes and saved search projections are
derived/rebuildable; canonical evidence remains authoritative.

### Mail integration

Mail reconstruction includes:

- account create/import/export/logout/delete and provider authorization;
- IMAP/Gmail sync, mailbox health and subscription controls;
- folders and provider labels, thread/message operational views;
- drafts, compose, reply, forward, redirect and send;
- read, archive, trash, restore, provider delete and supported bulk actions;
- signatures, Mail-specific rich templates and mail-merge preview;
- delivery status, delayed delivery, undo when provider semantics permit it;
- Mail attachment upload/download through admitted Blob and Attachment Security
  paths;
- provider-specific settings and resource mappings.

Mail-specific compose and templates stay in Mail. A provider-neutral
cross-channel intent is not routed through Mail UI directly and requires a
workflow.

### Telegram integration

The existing Telegram contract is the starting point, not proof of a complete
slice. Reconstruction includes:

- authorization and lifecycle;
- chat list, history, search, participants, topics and folders;
- text/media send, reply, forward, edit, delete/restore and reactions;
- pin, read, archive, mute, join and leave when they are Telegram operations;
- command status, replay/reconciliation and explicit failure states;
- provider-owned settings and setup.

The gate opens only when public generated clients, persistence/runtime routes,
descriptor capabilities, frontend controllers and live conformance agree on
the same exact contract revision.

### WhatsApp integration

Reconstruction includes:

- host bridge lifecycle and account status;
- dialogs, cached history, search, participants and replay;
- text/media/voice/status send;
- reply, forward, edit, delete and reactions where provider behavior supports
  them;
- conversation actions, join/leave and operation status;
- provider-owned settings and host diagnostics.

Existing internal query messages must not be exposed accidentally. Public query
service admission requires exact route, GrantSet, runtime handler, privacy
limits and generated client evidence.

### Zulip integration

Reconstruction includes:

- authorization and lifecycle;
- streams, topics, direct-message threads, history and search;
- send/update/delete, reactions and file transfer;
- subscriptions, participants and operation status;
- provider-owned settings and diagnostics.

The previous generic `Channels` UI is not restored. Zulip owns its operational
page. Slack, Discord and Mattermost remain absent until independently admitted.

### Compose, outbox and delivery workflows

Provider-specific compose remains at its integration. Separate workflows are
introduced only for actual cross-owner coordination:

- `communication_delivery_intent` for provider-neutral outbound intent;
- `communication_delayed_delivery` for schedule, acceptance and cancellation;
- `communication_bulk_action` for bounded fan-out with per-target receipts;
- `communication_cross_channel_forward` for explicit source evidence to target
  provider command.

Each workflow has one generated request/result contract, its own runtime and
durable state when needed, no direct owner storage access and no generic
`execute(any)` command.

Outbox UI is an app composition over workflow receipts and provider operation
status. `accepted` never means delivered. Undo is available only when the exact
workflow/provider state declares it safe.

### Content-derived and AI use-case workflows

The following are separate use cases, not modes of a generic AI endpoint:

- `communication_reply_suggestion`;
- `communication_summary`;
- `communication_translation`;
- `communication_explanation`;
- `communication_recipient_suggestion`;
- `communication_task_candidate_extraction`;
- `communication_note_candidate_extraction`;
- `attachment_text_extraction`;
- `call_transcription`.

Each workflow:

1. queries exact public owner contracts;
2. assembles a distinct typed AI request with `AiContextReceiptV1` when the
   admitted use case actually uses AI; deterministic recipient suggestion uses
   its own typed source receipt and does not acquire an AI/provider dependency;
3. records source revisions, completeness and applicable policy/model receipt;
4. returns a candidate/result without mutating Communications or another
   business domain;
5. promotes a task, note or Review item only through a target-domain command;
6. fails closed when required source content or egress consent is unavailable.

### Calls and recordings

`Calls` is a product surface, not a new business domain.

- WhatsApp, Telegram and Zulip own their provider call/meeting operational
  state if their admitted provider contracts support it.
- Communications may store provider-neutral call observations, participants,
  times and evidence references.
- Blob owns recording bytes.
- `call_transcription` owns transcription execution and candidate output.
- App composes canonical call evidence with exact provider actions.

ADR-0390 closes an underspecified dependency in the original inventory:
transcription cannot derive audio from Communications text or use generic LLM
generation. The first exact contour therefore has three independent
prerequisites before `call_transcription_v1`:

- `desktop_call_recording_v1` — explicit-consent recording integration and
  source-owned Blob custody, implemented after signed managed, authenticated
  SSE, Blob/NATS outage and restart conformance;
- `speech_to_text_engine_v1` — provider-neutral STT engine contract/runtime;
- `whisper_stt_provider_v1` — separately admitted concrete STT integration.

These gates do not merge recording, workflow, engine or integration owners.
Transcript bytes remain a workflow-owned Blob artifact and do not enter
Communications persistence, durable events, query responses or SSE.

Zoom, Google Meet, Microsoft Teams, Telemost and phone integrations are not
manufactured from historical cards. Each requires a separate integration ADR,
package inventory, credentials, runtime and provider conformance before it can
appear as active.

### Review and attention actions

Two similarly named actions are kept separate:

- provider read/archive/mute/pin/label action belongs to the integration;
- Макошь review/pin/snooze/important/promotion state belongs to `review`.

The `review` domain is currently blocked and is not silently opened by this
ADR. Its Communications use cases require a separate owner admission ADR,
typed evidence reference, storage/runtime packages and tests. Until then the
frontend must not simulate those controls.

### Settings composition

Settings remains an app workbench:

```text
app settings route
  + client preferences
  + Kernel/System Control
  + Scheduler controls
  + Telemetry diagnostics
  + Storage/Vault/Blob maintenance
  + AI owner settings
  + exact integration panels
  + Review/Signal composition when admitted
```

Rules:

- app chooses a panel and composes public presentation contracts;
- panel owner performs its query/command;
- Settings Registry stores desired/effective revisions and applies declared
  schemas, but does not merge or interpret provider settings;
- secrets use Vault leases and never appear in settings projections;
- provider sessions, cursors, checkpoints, health and Scheduler runs are not
  settings;
- application preferences are not provider settings;
- background jobs are Scheduler controls, not integration settings;
- maintenance uses component-owned commands, not generic filesystem access;
- Trace Logs use Telemetry, not direct log files;
- Signal Hub is composition, not a facade that owns integration state.

### Frontend layout

```text
frontend/src/domains/communications
frontend/src/integrations/mail
frontend/src/integrations/telegram
frontend/src/integrations/whatsapp
frontend/src/integrations/zulip
frontend/src/workflows/<use-case>
frontend/src/platform/<platform-owner>
frontend/src/app
frontend/src/shared
```

Domain не импортирует integration или workflow implementation. Integration не
импортирует domain implementation. Workflow adapters зависят только от
generated public contracts. `app` является единственным cross-owner
composition root. `shared` содержит stateless primitives, а не business state.

### SRP и единицы сборки

SRP определяется одной ответственностью и одной причиной изменения:

- contract package меняется из-за wire semantics;
- domain/core package — из-за owner rules;
- persistence package — из-за owner storage mapping;
- runtime package — из-за transport/lifecycle orchestration;
- integration adapter — из-за provider protocol;
- workflow — из-за одного cross-owner use case;
- frontend client — из-за одного generated service;
- controller — из-за одного user use case;
- mapper — из-за одного wire-to-view transformation;
- presentation component — из-за одного visual responsibility.

Количество строк не является критерием. Маленький generic facade, который
скрывает несколько owners, нарушает SRP. Большой typed owner-local algorithm не
делится искусственно, если имеет одну причину изменения.

Domain, integration, workflow, platform package и frontend owner surface
являются разными единицами сборки. Integration assembly не является runtime,
runtime не является domain, а app composition не становится owner.

## Capability register

| Historical capability | Target | Required gate |
|---|---|---|
| Canonical list/detail/search/evidence | Communications | `communications_canonical_read_v2` |
| Authorized message content and versions | Communications | `communications_content_read_v1` |
| Saved searches | Communications derived projection | `communications_saved_search_v1` |
| Top senders and provider-neutral sender insights | Communications derived projection | `communications_sender_insights_v1` |
| Evidence export | `communications_export` workflow plus Blob | `communications_export_v1` |
| Mail account import/export/logout/delete and authorization | Mail | `mail_account_lifecycle_v1` |
| Scoped credential retire/delete | Vault platform | `vault_credential_retirement_v1` |
| Mail sync, subscriptions and mailbox health | Mail | `mail_sync_health_v1` |
| Mail accounts/sync/folders/threads/messages | Mail | `mail_operational_read_v1` |
| Mail drafts/compose/templates/signatures | Mail | `mail_composition_v1` |
| Mail read/unread and star/unstar | Mail | `mail_message_flags_command_v1` |
| Mail stable message identity, IMAP locator and mailbox roles | Mail | `mail_provider_location_identity_v1` |
| Mail archive/trash/restore/move | Mail | `mail_message_location_command_v1` |
| Mail permanent provider delete | Mail | `mail_message_permanent_delete_command_v1` |
| Mail full mutation and delivery closure | Mail | `mail_operational_command_v1` after both location gates |
| Telegram authorization, history, search, media, mutations and operation audit | Telegram | `telegram_core_operational_v1` |
| Telegram user-only TDLib account identity and QR authorization | Telegram | `telegram_tdlib_user_qr_identity_v1` |
| Telegram atomic runtime restart/reconfiguration | Telegram | `telegram_runtime_reconfiguration_v1` |
| Telegram folder reassignment | Telegram | `telegram_folder_reassignment_v1` |
| Telegram automation policies, templates and dry-run | Telegram | `telegram_automation_v1` |
| Telegram provider call history | Telegram | `telegram_call_history_v1` |
| Telegram provider call signaling | Telegram | `telegram_call_signaling_v1` |
| Telegram real one-to-one audio media | Telegram | `telegram_call_media_v1` |
| Telegram complete provider calls | Telegram | `telegram_calls_operational_v1` after the three Calls gates above |
| Telegram full operational client closure | Telegram | `telegram_full_operational_v1` after all Telegram gates above |
| WhatsApp backend admission, command/status and host handoff | WhatsApp | `whatsapp_integration_v1` |
| WhatsApp operational read projection/search | WhatsApp | `whatsapp_operational_read_v1` |
| WhatsApp operational realtime/replay | WhatsApp | `whatsapp_operational_realtime_v1` |
| WhatsApp full generated client and UI closure | WhatsApp | `whatsapp_full_operational_v1` after both WhatsApp backend gates |
| Zulip lifecycle/history/search/client | Zulip | `zulip_full_operational_v1` |
| Provider-owned Telegram/WhatsApp QR frontend | first-party client composition | `provider_qr_account_linking_frontend_v1` |
| Browser development credential sealing | first-party development host | `loopback_browser_owner_vault_host_v1` |
| Browser development owner-device proof | first-party development host | `loopback_native_owner_device_proof_host_v1` |
| Immutable legacy provider recovery source | first-party client maintenance adapter | `legacy_provider_recovery_bundle_v1` |
| Native legacy secret custody and provider-specific apply | first-party client composition | `legacy_provider_native_secret_custody_v1` |
| Capability-routed current owner lookup for workflows | Kernel capability router | `capability_routed_module_query_rpc_v1` |
| Capability-routed typed module request | Kernel capability router | `capability_routed_module_request_rpc_v1` |
| Capability-routed managed client realtime | Kernel capability router | `capability_routed_managed_client_realtime_v1` |
| Client-safe platform system status over shared realtime | Kernel system-status projection + Core Gateway | `client_system_status_realtime_v1` |
| Provider-neutral delivery intent | `communication_delivery_intent` workflow | `communication_delivery_intent_v1` |
| Durable module-originated one-shot schedule control | Scheduler platform | `scheduler_module_schedule_control_v1` |
| Delayed delivery | `communication_delayed_delivery` workflow | `communication_delayed_delivery_v1` |
| Bulk action | `communication_bulk_action` workflow | `communication_bulk_action_v1` after `capability_routed_module_request_rpc_v1` |
| Cross-channel forward | `communication_cross_channel_forward` workflow | `communication_cross_channel_forward_v1` |
| Communications AI source handoff | Communications | `communications_ai_context_source_v1` |
| Typed local inference | `ai` engine | `ai_inference_v1` |
| Local Ollama provider execution | `ollama` integration | `ollama_ai_provider_v1` |
| AI Reply | `communication_reply_suggestion` workflow | `communication_reply_suggestion_v1` after the three AI dependency gates above |
| Summary | `communication_summary` workflow | `communication_summary_v1` |
| Translation | `communication_translation` workflow | `communication_translation_v1` |
| Explanation | `communication_explanation` workflow | `communication_explanation_v1` |
| Recipient suggestion source handoff | Communications | `communications_recipient_source_v1` |
| Smart CC/recipient suggestion | `communication_recipient_suggestion` workflow | `communication_recipient_suggestion_v1` |
| Task extraction | `communication_task_candidate_extraction` workflow plus Tasks command | `communication_task_candidate_extraction_v1` |
| Note extraction | `communication_note_candidate_extraction` workflow plus target command | `communication_note_candidate_extraction_v1` |
| Attachment text extraction | Blob, Attachment Security and explicit content workflow | `attachment_text_extraction_v1` |
| Attachment preview | Blob, Attachment Security and explicit preview workflow | `attachment_preview_v1` |
| Retained attachment evidence replay | Explicit owner-authorized producer replay workflow; no Kernel/outbox facade | `attachment_preview_retained_evidence_replay_v1` |
| Attachment archive inspection | Dedicated bounded engine | `attachment_archive_inspection_v1` |
| Attachment translation | `attachment_translation` workflow | `attachment_translation_v1` |
| Provider actions | Exact integration | integration command gate |
| Макошь pin/snooze/important/review | Review | `review_communications_attention_v1` after Review admission |
| Contacts identity command for Mail address-book entries | Contacts | `contacts_mail_identity_command_v1` |
| Mail address-book synchronization | `mail_contacts_sync` workflow plus Contacts command | `mail_contacts_sync_v1` after `contacts_mail_identity_command_v1` |
| Calls aggregation | app plus Communications call evidence | `communications_call_evidence_v1` |
| Explicit-consent desktop recording | Desktop recording integration plus Blob | `desktop_call_recording_v1` |
| Provider-neutral speech recognition | Speech-to-Text engine | `speech_to_text_engine_v1` |
| Concrete local Whisper execution | Whisper STT integration | `whisper_stt_provider_v1` |
| Recording/transcript | Blob plus transcription workflow | `call_transcription_v1` after all three STT/recording gates |
| App preferences | client/app | `application_preferences_v1` |
| Background jobs | Scheduler | `scheduler_settings_surface_v1` |
| Trace Logs | Telemetry | `telemetry_diagnostics_surface_v1` |
| Maintenance | owning platform components | `platform_maintenance_surface_v1` |
| Architecture blockers/status | app composition over sanitized platform status | `system_architecture_status_surface_v1` |
| Calendar account settings | app composition of separately admitted calendar integrations | `calendar_account_settings_composition_v1` |
| AI Control Center | AI plus provider adapters | `ai_settings_surface_v1` plus separate provider gates |
| Signal Hub | app composition of admitted owners | `signal_hub_composition_v1` |

## Порядок реализации

Порядок является dependency order, а не обещанием объединить slices в один
release.

1. `communications_capability_reconstruction_inventory_v1` — этот ADR,
   executable inventory assertions и gap ledger.
2. Telegram проходит независимые gates:
   implemented `telegram_core_operational_v1`,
   implemented `telegram_tdlib_user_qr_identity_v1`,
   `telegram_runtime_reconfiguration_v1`,
   implemented `telegram_folder_reassignment_v1`,
   `telegram_automation_v1`,
   `telegram_calls_operational_v1`. Calls umbrella отдельно требует
   `telegram_call_history_v1`, `telegram_call_signaling_v1` и
   `telegram_call_media_v1` по ADR-0284. Только их совокупное evidence закрывает
   umbrella gate `telegram_full_operational_v1`; существующий typed backend и
   frontend не считаются полным переносом, пока отсутствует хотя бы один из
   этих контрактов.
3. WhatsApp проходит independently admitted
   `whatsapp_operational_read_v1` и
   `whatsapp_operational_realtime_v1` по ADR-0286. Только после них,
   generated frontend client и integration-owned UI cutover открывается
   `whatsapp_full_operational_v1`.
4. Zulip сохраняет реализованный backend foundation
   `zulip_integration_v1` по ADR-0271 и проходит независимые
   provider-neutral `managed_integration_settings_apply_v1` по ADR-0292,
   `zulip_account_lifecycle_v1`,
   `zulip_history_sync_v1`, `zulip_operational_read_v1` и
   `zulip_operational_realtime_v1` по ADR-0291. Только после них, generated
   frontend client и integration-owned UI cutover открывается
   `zulip_full_operational_v1`.
5. Vault закрывает provider-neutral `vault_credential_retirement_v1` по
   ADR-0293 и implemented `owner_vault_provisioning_backend_v1` по ADR-0295.
   First-party desktop отдельно закрывает implemented
   `owner_vault_provisioning_desktop_v1`, включая generated client, native host
   adapter и integration-owned UI. First-party development assembly отдельно
   закрывает `loopback_browser_owner_vault_host_v1` и
   `loopback_native_owner_device_proof_host_v1`, а provider frontend
   закрывает `provider_qr_account_linking_frontend_v1`; эти gates не
   превращают host adapter в integration или QR authority. Multi-client
   `owner_vault_provisioning_v1`
   остаётся planned до Android adapter; backend gate не подменяет client gates.
   Mail независимо проходит implemented
   `mail_account_credential_binding_v1`, implemented
   `mail_account_retire_delete_v1` и implemented desktop
   `mail_account_portability_v1` по ADR-0294; implemented umbrella
   `mail_account_lifecycle_v1` требует все три slices, уже
   реализованный `mail_gmail_oauth_v1`, generic managed Settings apply и
   provider-neutral public `owner_module_settings_gateway_v1` по ADR-0296.
   Fresh-proof typed export отдельно проходит implemented
   `owner_module_settings_export_v1` по ADR-0297 и является dependency
   portability, а не implicit read-all правом Gateway.
   Owner-authorized legacy source отдельно проходит implemented
   `legacy_provider_recovery_bundle_v1` по ADR-0321. Его подключение к current
   Vault и provider-specific Mail/Telegram flows остаётся planned
   `legacy_provider_native_secret_custody_v1` и требует working loopback owner
   proof по ADR-0322; source adapter не становится
   integration, domain или generic account service.
   Затем Mail read, composition и command gates проходят независимо, без
   одного всесильного Mail capability.
6. Communications canonical read/content/saved-search gates.
7. Delivery/outbox workflows, каждый отдельным package/gate.
8. AI use-case workflows, каждый отдельным package/gate.
9. Calls evidence and transcription slices.
10. App/platform/AI/integration Settings panels по owner gates.
11. Review admission и Signal Hub composition.
12. `communications_settings_reconstruction_complete_v1` — только после
    closure всего register без unclassified supported capability.

Порядок между независимыми slices может меняться, но dependency и admission
rules не ослабляются.

Executable reconstruction inventory различает:

- `planned` — обязательный slice классифицирован, но его evidence gate не
  закрыт;
- `implemented` — slice имеет полный contract/runtime/storage/client/live
  evidence, но сам по себе не добавлен в production owner inventory.

`implemented` не означает `admitted`. Production admission остаётся отдельным
exact phase gate в `backend/architecture/policy.json`. Для Telegram отдельные
implemented slices не добавляют bundled integration в production inventory:
это разрешено только после закрытия `telegram_full_operational_v1`.

## Evidence для каждого slice

Gate может быть открыт только при наличии:

1. exact owner, role, package inventory и Cargo/build isolation;
2. versioned typed request/query/command/event/result/ack contracts;
3. descriptor capabilities, GrantSet, route and schema hash admission;
4. owner-local persistence, migrations, outbox/inbox and replay semantics;
5. runtime generation, grant epoch, credential and storage fencing;
6. bounded inputs/outputs, authorization, privacy and negative tests;
7. generated desktop/Android-neutral client contract;
8. owner-local frontend client/controller/presentation and exact capability
   guard;
9. unit, contract, persistence, runtime, architecture and frontend tests;
10. live managed-launch/provider conformance where the slice performs provider
    I/O;
11. policy and architecture evidence hashes;
12. scoped diff review and a commit containing only the slice.

Package tests, TypeScript typecheck or rendered screen alone do not open a
backend/runtime gate.

## Rollback

Каждый slice должен быть independently revocable:

- revoke exact capability/GrantSet and stop routing new commands;
- stop the owning managed runtime without stopping unrelated owners;
- keep accepted durable records and receipts queryable where safe;
- drain/reconcile in-flight commands according to owner semantics;
- keep schema migrations additive/forward-compatible;
- hide frontend action when exact capability is unavailable;
- never fall back to legacy REST or another owner;
- preserve canonical evidence already accepted through valid event lineage.

Rollback одного provider не отключает Communications или другие integrations.
Rollback workflow не изменяет target-domain truth directly.

## Completion contract

Полный перенос считается завершённым только когда:

- каждый supported historical capability из register реализован и admitted;
- каждый historical facade explicitly classified as non-capability;
- нет active scoped legacy REST/query/realtime/business state;
- Mail, Telegram, WhatsApp и Zulip имеют независимые backend/runtime/frontend
  closure;
- Communications не импортирует provider code;
- Settings не интерпретирует чужую семантику;
- cross-owner and AI behavior проходит только через explicit workflows;
- production inventory, live conformance и client UI подтверждают одинаковые
  contract revisions;
- worktree, architecture evidence и full relevant gates green.

ADR-0281 остаётся clean-room ownership/transport cutover, а этот ADR фиксирует
полное закрытие capability reconstruction inventory без возврата legacy facade.

## Последствия

- Возможности возвращаются без восстановления прежнего monolith.
- Backend readiness становится обязательным предшественником frontend action.
- Provider failures и releases остаются независимыми.
- Новые workflows увеличивают число units, но делают authority и failure
  semantics явными.
- Неподтверждённые demo cards не превращаются в fake product behavior.
- Completion будет дольше, чем возврат старых Vue файлов, но будет проверяемым
  и совместимым с clean-room архитектурой.

## Отклонённые варианты

### Вернуть удалённые Communications и Settings directories

Отклонено: вместе с UI возвращаются mixed ownership, REST authority, provider
DTO unions и скрытые domain-to-integration calls.

### Реализовать общий Communications backend для provider operations

Отклонено: domain начинает владеть integrations и provider truth.

### Пропустить backend и восстановить frontend mock/projection

Отклонено: screen не является capability и создаёт fake done.

### Один workflow для всех cross-owner и AI действий

Отклонено: generic executor скрывает authority, context, egress policy,
idempotency и failure semantics.

### Считать Settings отдельным business domain

Отклонено: Settings является composition и control plane разных владельцев, а
не источником общей business truth.
