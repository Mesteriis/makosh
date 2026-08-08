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

- Chunk ID / ID чанка: `118-doc-docs-part-009`
- Group / Группа: `docs`
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

### `docs/platform/event-tracing/testing.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/event-tracing/testing.md`
- Size bytes / Размер в байтах: `1653`
- Included characters / Включено символов: `1653`
- Truncated / Обрезано: `no`

````markdown
# Event Tracing Testing

## Purpose

Trace tests prove that Макошь can reconstruct causal event graphs without
external telemetry infrastructure.

## Unit Tests

Required unit coverage:

- `TraceContext::root`;
- `TraceContext::child_of`;
- event builder correlation normalization;
- trace graph reconstruction from stored events;
- missing parent detection;
- orphan root handling.

## Integration Tests

Required PostgreSQL-backed coverage:

- observation to raw signal to accepted signal to communication event;
- Telegram fixture trace;
- WhatsApp fixture trace;
- Mail fixture trace;
- DLQ annotation on failed consumer;
- realtime payload includes trace fields.

Use existing repository conventions:

- `testcontainers-rs` through `crates/testkit`;
- `cargo nextest` through Makefile targets for full backend validation;
- deterministic fixtures instead of live Telegram, WhatsApp, Mail or external
  providers;
- no dependency on a developer's local PostgreSQL instance.

## API Tests

API tests should exercise:

- `GET /api/v1/events/{event_id}/trace`;
- `GET /api/v1/event-traces/{correlation_id}`;
- `GET /api/v1/events/{event_id}/children`;
- payload sanitization on trace and realtime responses;
- missing event response behavior.

## Frontend Tests

Frontend tests cover:

- platform trace API paths;
- provider-neutral query keys;
- shared trace panel ownership boundaries.

## Regression Cases

Every bug that disconnects a trace chain should add a test showing the broken
chain before the fix:

```text
observation.captured.v1
  -> signal.raw.<source>.<thing>.observed
  -> signal.accepted.<source>.<thing>
  -> owning domain event
```
````

### `docs/platform/realtime-conversation/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/realtime-conversation/README.md`
- Size bytes / Размер в байтах: `1584`
- Included characters / Включено символов: `1576`
- Truncated / Обрезано: `no`

````markdown
# Макошь Realtime Conversation Platform

Status: `TARGET_ARCHITECTURE`, 2026-06-28.

The Realtime Conversation Platform is the provider-neutral layer for live
conversations in Макошь. Zoom, Yandex Telemost, Google Meet, Jitsi, Discord and
future call providers are external systems. They do not own Макошь memory. They
only provide runtime access, links, local capture opportunities and provider
evidence.

Макошь owns the durable memory object:

```text
Live conversation
↓
Call Bundle
↓
Transcription / diarization / speaker identity
↓
Call Intelligence
↓
Timeline / Radar / Documents / Knowledge Graph / Tasks
```

This keeps provider integrations thin and reusable. The durable value is the
evidence-backed memory of what was said, who said it, what was decided, what
was promised and which context it belongs to.

## Provider-neutral invariants

- Provider integrations never become domains.
- A provider conference is evidence, not the source of truth.
- Local recording is explicit, visible and consent-gated.
- WebView speaker state is a hint, not truth.
- AI produces candidates with source, confidence and evidence.
- Domain mutations go through workflows/events, not direct integration calls.
- Reprocessing must be possible when better models appear later.

## Documents

- [Architecture](./architecture.md)
- [Recording bundle](./recording-bundle.md)
- [Call intelligence](../../engines/call-intelligence/README.md)
- [Speaker identity](../../engines/speaker-identity/README.md)
- [Providers](./providers.md)
- [Replay and live notes](./replay-and-live-notes.md)
````

### `docs/platform/realtime-conversation/architecture.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/realtime-conversation/architecture.md`
- Size bytes / Размер в байтах: `2850`
- Included characters / Включено символов: `2838`
- Truncated / Обрезано: `no`

````markdown
# Realtime Conversation Platform Architecture

## Boundary

The Meeting Platform is a provider-neutral kernel for realtime conversations.
It lives conceptually around:

```text
backend/src/platform/realtime_conversation
backend/src/engines/call_intelligence
backend/src/engines/speaker_identity
backend/src/workflows/realtime_conversation_*
frontend/src/integrations/<provider>
frontend/src-tauri/src/<provider>_companion.rs
```

Provider-specific code lives under `integrations/*` or desktop companion
modules. Durable meeting memory belongs to provider-neutral bundles, workflows,
engines and projections.

## Flow

```text
External provider
↓
Provider runtime adapter / visible WebView / local recorder
↓
Call Bundle
↓
Transcription and diarization
↓
Speaker identity merge
↓
Call Intelligence
↓
Radar / Timeline / Knowledge Graph / Tasks / Documents
```

## Ownership

| Layer | Owns | Must not own |
|---|---|---|
| Provider integration | API, runtime auth, provider commands, WebView opening | Calendar, Tasks, Radar, Knowledge |
| Desktop companion | visible WebView, local recording process, speaker hints | business truth |
| Call Bundle | immutable local artifacts and manifest | final decisions |
| Call Intelligence engine | candidates and extracted structure | domain state |
| Workflows | orchestration and projections | raw provider sessions |
| Domains | accepted business state | provider runtime |

## Event language

Provider integrations publish provider facts:

```text
integration.<provider>.conference.created
integration.<provider>.conference.observed
integration.<provider>.speaker_hint.observed
integration.<provider>.local_recording.completed
```

Provider-neutral workflows publish meeting memory facts:

```text
realtime_conversation.bundle.created
realtime_conversation.transcript.completed
realtime_conversation.speaker_identity.candidate_detected
realtime_conversation.decision.candidate_detected
realtime_conversation.action_item.candidate_detected
realtime_conversation.radar_signal.detected
```

Domain owners later accept or reject candidates. AI output remains a candidate
until it is backed by evidence and accepted through the owning review workflow.

## Source policy

Every generated insight must include:

```text
source artifact
source time range
confidence
model/tool version if applicable
evidence reference
```

Examples:

```text
source = audio.mp3#t=00:15:20-00:16:02
source = transcript.json#segment=42
source = speaker-hints.jsonl#line=17
source = screenshots/screen-00042.png
```

## Reprocessing

The bundle is intentionally artifact-first. When diarization, Whisper, OCR or
entity extraction improves, Макошь can rerun the pipeline without asking the
provider for the meeting again. Providers are unreliable memory. Local evidence
is less glamorous, but it survives product redesigns.
````

### `docs/platform/realtime-conversation/providers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/realtime-conversation/providers.md`
- Size bytes / Размер в байтах: `1817`
- Included characters / Включено символов: `1799`
- Truncated / Обрезано: `no`

````markdown
# Meeting Providers

A meeting provider is an adapter. It opens or observes a live conversation and
exposes capabilities. It does not own meeting memory.

## Provider capability surface

```text
conferences.create
conferences.read
conferences.update
webview.open
audio.local_capture
speaker_hints.webview
chat.capture
participants.observe
screenshare.detect
screenshots.capture
recording.local_mp3
transcript.provider_read
recording.provider_read
live_stream.create
```

## Provider types

| Provider | Role |
|---|---|
| Yandex Telemost | API-created conference + visible WebView + local capture |
| Zoom | provider API + possible meeting evidence + future shared call bundle |
| Jitsi | link/WebView-first provider |
| Google Meet | WebView/browser-first provider |
| Discord/Signal calls | realtime conversation source, API surface varies |

## Integration shape

```text
backend/src/integrations/<provider>
frontend/src/integrations/<provider>
frontend/src-tauri/src/<provider>_companion.rs
```

Provider code must expose runtime/account/capability APIs and emit integration
events. It must not write to Calendar, Tasks, Radar, Documents or Knowledge.

## Provider command flow

```text
App/Calendar intent
↓
workflow/provider command
↓
integration provider command handler
↓
external provider API/WebView action
↓
integration.<provider>.command.completed/failed
↓
provider-neutral projection
```

## Provider evidence flow

```text
visible session / provider API / local recorder
↓
integration.<provider>.*.observed
↓
Call Bundle
↓
Call Intelligence
↓
Radar/Timeline/Knowledge/Tasks candidates
```

The goal is not to make Макошь dependent on any vendor's meeting product. The
goal is to use providers as temporary doors into conversations, then store the
useful evidence in Макошь-owned form.
````

### `docs/platform/realtime-conversation/recording-bundle.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/realtime-conversation/recording-bundle.md`
- Size bytes / Размер в байтах: `2746`
- Included characters / Включено символов: `2614`
- Truncated / Обрезано: `no`

````markdown
# Recording Bundle Contract

A Call Bundle is the durable evidence package for a live conversation.

## Layout

```text
call-bundle-{id}/
├── manifest.json
├── meeting.json
├── provider.json
├── participants.json
├── audio.mp3
├── speaker-hints.jsonl
├── speaker-timeline.txt
├── event-track.jsonl
├── chat.json
├── transcript.json
├── transcript.md
├── summary.md
├── topics.json
├── entities.json
├── decisions.json
├── tasks.json
├── knowledge.json
├── metrics.json
├── radar-signals.json
├── screenshots/
├── attachments/
└── ocr/
```

Only a subset exists immediately after recording. Later pipeline stages append
new artifacts.

## Manifest

```json
{
  "schema_version": 1,
  "bundle_id": "call_...",
  "provider_kind": "yandex_telemost",
  "provider_shape": "visible_webview_local_capture",
  "account_id": "...",
  "provider_conference_id": "...",
  "join_url": "...",
  "calendar_event_id": null,
  "project_id": null,
  "organization_id": null,
  "artifacts": [
    {
      "kind": "audio",
      "relative_path": "audio.mp3",
      "source": "local_audio_loopback",
      "truth_status": "capture_artifact",
      "media_type": "audio/mpeg"
    },
    {
      "kind": "speaker_hints",
      "relative_path": "speaker-hints.jsonl",
      "source": "visible_webview_dom_heuristic",
      "truth_status": "hint_not_truth",
      "media_type": "application/x-ndjson"
    }
  ]
}
```

## Artifact policy

| Artifact | Truth status | Purpose |
|---|---|---|
| `audio.mp3` | capture artifact | transcription, diarization |
| `speaker-hints.jsonl` | hint, not truth | warm start speaker assignment |
| `event-track.jsonl` | observed runtime events | meeting timeline |
| `chat.json` | provider/UI capture | communication context |
| `screenshots/*` | local visual evidence | OCR, screen intelligence |
| `transcript.json` | model output | searchable text with evidence links |
| `decisions.json` | candidate output | review/Radar/ADR candidates |
| `tasks.json` | candidate output | review/Radar/task candidates |

## Privacy and consent

A bundle may only be created from a visible user-owned session with explicit
recording consent. Hidden capture, silent device installation and background
meeting joins are forbidden.

The bundle should record privacy metadata:

```json
{
  "capture_mode": "visible_webview_local_loopback",
  "consent_attested": true,
  "hidden_capture": false,
  "provider_recording": false,
  "local_only": true
}
```

## Immutability

Raw artifacts should be append-only. Derived artifacts may be superseded by a
new version, but they should not overwrite the evidence they came from.
````

### `docs/platform/realtime-conversation/replay-and-live-notes.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/realtime-conversation/replay-and-live-notes.md`
- Size bytes / Размер в байтах: `1556`
- Included characters / Включено символов: `1514`
- Truncated / Обрезано: `no`

````markdown
# Replay and Live Notes

Meeting memory should be useful during the meeting and after it.

## Live notes panel

A meeting WebView can be paired with a Макошь side panel:

```text
Meeting
├── active speaker hints
├── emerging topics
├── candidate action items
├── candidate decisions
├── mentioned people/organizations
├── attached documents
└── Radar signals
```

Live output must be marked as provisional. The post-meeting pipeline may correct
speaker identity, segment boundaries and candidate confidence.

## Replay model

A completed Call Bundle should support synchronized replay:

```text
audio position
transcript segment
speaker identity
topic timeline
chat messages
screenshots/OCR
candidate decisions
action items
Radar signals
```

Selecting `15:23` should jump to:

```text
audio t=15:23
transcript segment covering 15:23
nearest screenshot
current topic
current speaker candidate
related tasks/decisions if accepted
```

## Event track

`event-track.jsonl` records non-transcript events:

```json
{"offset_ms":0,"event":"meeting_opened"}
{"offset_ms":12000,"event":"participant_joined","label":"Ivan"}
{"offset_ms":42000,"event":"screen_share_started"}
{"offset_ms":960000,"event":"decision_candidate_detected"}
{"offset_ms":1420000,"event":"meeting_left"}
```

## Search

Meeting memory should be searchable by:

```text
spoken text
topic
participant
organization
project
decision
action item
screen OCR
attachment name
```

This turns meetings from disposable conversation into searchable evidence.
````

### `docs/platform/settings/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/platform/settings/README.md`
- Size bytes / Размер в байтах: `1587`
- Included characters / Включено символов: `1587`
- Truncated / Обрезано: `no`

```markdown
# Application Settings

Status: code-aligned documentation package created from ADR-0054 and current
backend/frontend modules.

Application settings are allowlisted, typed, non-secret runtime and UI values.
They are a platform contract and Settings UI surface, not a product domain that
owns provider accounts or credentials.

ADR source of truth:

- [ADR-0054 Application Settings Store](../../adr/ADR-0054-application-settings-store.md)
- [ADR-0081 Opt-In OmniRoute AI Runtime](../../adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [ADR-0082 AI Settings Control Center](../../adr/ADR-0082-ai-settings-control-center.md)

## Current Implementation Evidence

Current backend files:

- `backend/src/platform/settings.rs`;
- `backend/src/platform/settings/store.rs`;
- `backend/src/platform/settings/models.rs`;
- `backend/src/platform/settings/definitions.rs`;
- `backend/src/app/router/routes/settings.rs`;
- `backend/src/app/handlers/settings/mod.rs`.

Current frontend package:

- `frontend/src/domains/settings`.

Current API routes include:

- `/api/v1/settings`;
- `/api/v1/settings/accounts`;
- `/api/v1/settings/{setting_key}`.

`ApplicationSettingsStore` lists declared settings, updates editable declared
values, derives AI runtime settings and repairs the declared settings table at
startup. Setting value kinds are boolean, integer, string and JSON.

## Boundary Rule

Settings store declared non-secret values only. Provider accounts remain
provider/account records, and credential material remains behind the vault or
secret boundary. Secret-like setting keys are rejected.
```

### `docs/product/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/product/README.md`
- Size bytes / Размер в байтах: `421`
- Included characters / Включено символов: `421`
- Truncated / Обрезано: `no`

```markdown
# Product

Status: documentation package aligned to the current repository structure.

Product documents define the product scope, master specification and roadmap.
They are product-level sources of truth, not implementation status reports.

## Navigation

- [Master Spec](./master-spec.md)
- [Product Charter](./product-charter.md)
- [Product Scope](./product-scope.md)
- [Development Roadmap](./development-roadmap.md)
```

### `docs/product/development-roadmap.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/product/development-roadmap.md`
- Size bytes / Размер в байтах: `13769`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Макошь Product Development Roadmap

## Status

This roadmap is derived from the Product Master Spec. It is forward-looking and
records target-vs-current gaps discovered from the current repository.

It does not replace historical version checklists under `docs/roadmap/`. Those
files describe past implementation milestones and may contain compatibility
terminology.

## Roadmap Principle

Development should follow the product spine:

```text
Communication
  -> Evidence
  -> Knowledge / Memory
  -> Relationships / Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Each slice should improve Макошь as one Personal Memory System, not as a set of
separate apps.

## Current Implementation Baseline

The repository already contains meaningful implementation slices:

| Area | Current implementation evidence |
|---|---|
| Communications and email | `domains/communications`, communication ingestion/messages migrations, `/api/v1/communications/*`, mail sync, drafts, send/reply/forward, workflow state, analytics, invoices, legal docs, certificates and attachment metadata. |
| Telegram | Telegram integration modules, runtime manager, migrations for chats/messages/policies/calls, `/api/v1/integrations/telegram/*` routes, Telegram frontend page and production capability target in `docs/integrations/telegram/` / ADR-0091. |
| WhatsApp | WhatsApp integration modules, WhatsApp Web sessions/messages migrations, `/api/v1/integrations/whatsapp/*` routes and WhatsApp frontend page. |
| Graph | Graph tables, graph projection module and `/api/v1/graph/*` routes. |
| Documents | Document tables, processing jobs/artifacts, document processing APIs and Documents frontend page. |
| Projects | Project tables, project link review workflow, project APIs and Projects frontend page. |
| Personas compatibility | `persons` tables, identity candidates, memory cards, preferences, timeline events, expertise, risks, promises and `/api/v1/persons/*` routes. |
| Organizations | Organization tables, identities, departments, contacts, memory, risks, finance/enrichment routes and Organizations frontend page. |
| Calendar and events | Calendar account/source/event tables, meetings, outcomes, deadlines, focus blocks, rules and Calendar frontend page. |
| Tasks | Task candidates, tasks, task evidence/context/relations/rules/templates and Tasks frontend page. |
| AI and agents | AI runtime, semantic embeddings, AI control center, AI APIs and Agents frontend page. |
| Settings and security | Application settings, secret references, encrypted vault entries, host vault, audit log and capability decision code. |
| UI operating surface | Frontend pages for home, communications, knowledge, timeline, notes, settings and domain surfaces. |

## Target-State Gaps

| Gap | Why it matters | Refactoring or delivery direction |
|---|---|---|
| Communication is not yet documented as the single ingestion spine across all channels. | Mail, Telegram, WhatsApp, calls and meetings can drift into separate apps. | Create Communications domain spec and normalize channel docs around source evidence and canonical Communication. |
| Telegram production capability model is broader than current implementation. | The requested production surface includes account/session lifecycle, proxies, chat management, message commands, soft delete, message history, attachments, voice/video messages, calls, channels, groups, forums, search, drafts, notifications, address book data, media gallery, offline, export and desktop UX. | Use ADR-0091 and `docs/integrations/telegram/` as the delivery contract; expose every new feature through backend capability states before enabling UI controls. |
| Persona target model is not implemented end-to-end. | Current `persons` compatibility still carries contact/person history. Owner Persona storage, GET/PUT owner compatibility route, ADR-0090 Persona-native read/write compatibility bridge, PersonaType, person role Relationship adapters, `person_personas` interaction-context Preference adapters, enrichment trust Relationship adapters and notes-to-memory-card adapters have baselines. | Plan physical Persona-native schema migration beyond the compatibility bridge. |
| Relationships are not a complete first-class model. | Roles, organization links, graph edges and relationship events are spread across domains. Durable Relationship records, graph projection for all current Relationship entity kinds, guarded entity/global review routes, manual/API person role adapters, manual/API and email-sync organization contact link adapters, manual task relation adapters, project link review adapters, Personas workspace review and cross-domain Review workspace review/action routing have a baseline. | Migrate remaining relationship-shaped compatibility surfaces behind Relationship records and keep review routing in the Review workspace. |
| Polygraph engine is partially implemented. | Structured direct contradictions can be stored as reviewable observations, deterministic structured and limited natural-language `location` / `status` claims can be extracted from Communication/Document/Event evidence text, projected email/Telegram/WhatsApp message refresh, imported Document refresh, meeting-note refresh and call-transcript refresh can compare active `person_facts` Memory claims against evidence by Persona email identity, active Telegram/WhatsApp identity, event participant link or active Telegram call identity, guarded backend routes can list/review observations without overwriting Memory, and the Knowledge workspace plus cross-domain Review workspace have Polygraph review panels and shared review action routing. Broad natural-language extraction and broader provider evidence remain incomplete. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics. |
| Decisions and Obligations are partial top-level domains. | Both have source-backed persistence, deterministic candidate detectors where explicit evidence exists, accepted graph projection, guarded backend entity/global list/review routes, a global Tasks workspace review panel and a cross-domain Review workspace panel with shared action routing. Message and document task candidate refresh use Obligation detection for explicit commitments/requests, confirmed `obligation_task` candidates now materialize accepted Obligations linked to Tasks, reset/reject review on those candidates synchronizes the durable Obligation state, email sync and Telegram/WhatsApp fixture ingestion refresh reviewable Decision and obligation-derived task candidates for projected Communications, compatibility `person_promises` now materialize accepted Obligations, explicit message/imported-document Decision candidates now persist as source-backed `suggested` Decisions, project link reviews now materialize accepted Decisions, and meeting outcomes now create reviewable Decisions or Obligations for `decision`, `promise`, `task` and `follow_up` outcomes. Broader live-provider ingestion, candidate routing and follow-ups can still blur together. | Wire remaining candidate extraction and review workflows to accepted Decisions and Obligations, then add adapters from compatibility surfaces. |
| Engine boundaries are not fully separated. | Memory, Timeline, Trust, Risk, Enrichment and Obligation behavior appears inside domain modules. | Write engine specs before extraction or renaming. |
| Notes remain ambiguous. | Frontend has Notes page, but foundation treats Notes as document-like artifacts. | Keep Notes as capture/document artifacts until a future ADR promotes them. |
| Documentation tree is incomplete. | Developers cannot yet derive all domain behavior from one product model. | Complete Wave 1 first, then create domain, engine and workflow specs in order. |

## Slice 1: Communication Memory Spine

Goal: make Communications the clear ingestion backbone.

Documentation outcomes:

- `docs/domains/communications/README.md`;
- channel mapping for email, Telegram, WhatsApp, calls and meetings;
- source evidence rules;
- canonical Communication lifecycle;
- current implementation compatibility notes for `domains/communications` and route names.

Implementation plan topics:

- keep provider-specific adapters;
- avoid renaming code until route/schema migration is explicitly planned;
- ensure every communication source can produce evidence, events and graph links.

## Slice 2: Persona And Relationship Memory

Goal: move from compatibility `persons` toward Persona and first-class
Relationship memory.

Documentation outcomes:

- `docs/domains/persons/spec.md`;
- `docs/domains/relationships/README.md`;
- Owner Persona behavior;
- PersonaType behavior;
- Persona identity trace lifecycle;
- Relationship trust, strength, provenance and validity rules.

Refactoring plan topics:

- introduce Owner Persona semantics;
- decide `/persons` compatibility strategy before any `/personas` route work;
- retire remaining Persona root compatibility caches only through a
  schema/API migration plan;
- preserve existing identity candidate review behavior while changing language.

## Slice 3: Knowledge And Polygraph

Goal: make Макошь able to detect conflicts between new evidence and accepted
memory.

Documentation outcomes:

- `docs/domains/graph/README.md`;
- `docs/engines/consistency/README.md`;
- contradiction review workflow;
- memory conflict taxonomy.

Refactoring plan topics:

- define `ContradictionObservation` as a target concept before schema work;
- connect contradictions to Communications, Documents, Decisions, Obligations,
  Personas, Organizations and Projects;
- ensure contradictions create review items rather than automatic truth changes.

## Slice 4: Obligations, Tasks And Decisions

Goal: separate commitments, executable work and durable choices.

Documentation outcomes:

- `docs/domains/obligations/README.md`;
- `docs/domains/tasks/spec.md`;
- `docs/domains/decisions/README.md`;
- workflow docs for communication-to-obligation and meeting-to-decisions.

Refactoring plan topics:

- separate Obligation from Task and Follow-Up in code and docs;
- map remaining compatibility follow-up cases and task candidates into target
  concepts;
- define how accepted obligations create tasks, follow-ups or risks.

## Slice 5: Projects And Documents Context

Goal: make Projects and Documents the durable context anchors for knowledge
work.

Documentation outcomes:

- `docs/domains/projects/README.md`;
- `docs/domains/documents/README.md`;
- workflow docs for document-to-context;
- project context and document evidence rules.

Refactoring plan topics:

- ensure project memory uses source-backed graph links;
- preserve immutable document evidence before summaries;
- keep Notes as document-like capture artifacts unless an ADR changes scope.

## Slice 6: Agents Over Context

Goal: make agents operate through source-backed context, not private guesses.

Documentation outcomes:

- agent context rules in `docs/ai/agents/`;
- workflow doc for agent-assisted recall;
- alignment with AI control center and capability policy.

Refactoring plan topics:

- represent AI agents as Personas when they enter the graph;
- make agent writes auditable;
- keep AI output derived until accepted by domain rules.

## Slice 7: Operating Surface

Goal: present Макошь as a dense desktop personal operating environment.

Documentation outcomes:

- UI information architecture tied to master-spec domains;
- navigation model for Communications, Context, Memory, Projects and Actions;
- compatibility strategy for current Persons, Notes, Timeline and Knowledge pages.

Refactoring plan topics:

- align UI labels with foundation vocabulary;
- avoid app-like silos where the product model says shared context;
- use current frontend pages as surfaces over one memory system.

## Documentation Workstream

Recommended order:

1. Product spine: master spec, development roadmap, docs index.
2. Core domain specs: Communications,
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/product/master-spec.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/product/master-spec.md`
- Size bytes / Размер в байтах: `20811`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Макошь Product Master Spec

## Status

This is the product-level source of truth for active Макошь documentation.

It describes the target product model and the current implementation baseline at
the same time. When these differ, the target model governs future product
direction, while the implementation baseline tells developers what actually
exists today.

This document does not define API routes, database migrations or runtime
implementation details.

## Canonical Product Definition

Макошь is a local-first Personal Memory System.

Its product experience is a personal operating surface for:

- Communications;
- Knowledge;
- Memory;
- Relationships;
- Projects;
- Documents;
- Decisions;
- Obligations;
- Context.

The primary value is context. CRUD screens, inboxes, calendars, task lists and
document viewers are product surfaces, not the product thesis.

## Product Thesis

Макошь turns communications into durable personal memory and actionable context.

The core product cycle is:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Макошь should help the owner answer:

- what happened;
- who and what is involved;
- why something matters;
- what evidence supports it;
- what changed compared with previous memory;
- what obligations, decisions or tasks emerged;
- what context is needed before acting.

## What Макошь Is Not

Макошь is not:

- an email client;
- a messenger;
- a CRM;
- an address book;
- a task tracker;
- a calendar app;
- a note-taking app;
- a generic knowledge base;
- an AI chatbot.

These surfaces may exist inside Макошь, but only as views and workflows over one
source-backed memory system.

## Communication As Primary Ingestion Spine

Communication is the primary way real-world signals enter Макошь.

Communication includes:

- email;
- Telegram messages;
- WhatsApp messages;
- calls;
- meetings;
- threads and conversations;
- attachments and linked documents;
- replies, delays, silences and follow-ups where they carry meaning.

A Communication is not just an inbox item. It is evidence that can produce
knowledge, relationships, obligations, decisions, tasks and project context.

Provider-specific production behavior stays under channel capability specs. The
current Telegram channel capability matrix is
[Telegram Channel Capability Spec](../integrations/telegram/README.md) and
[Telegram Gap Analysis](../integrations/telegram/gap-analysis.md), governed by
ADR-0091 and ADR-0097.

Communications are primary, but they are not the only source of evidence.
Documents, calendar events, manual owner input, imported files and provider
records can also create durable memory.

## Source Evidence To Memory Flow

Макошь must preserve evidence before extracting meaning.

```text
Provider or local source
  -> Source Record
  -> Canonical Event
  -> Domain Projection
  -> Knowledge / Memory Candidate
  -> Review or Policy Acceptance
  -> Durable Memory
  -> Derived Views and Agent Context
```

Rules:

- raw provider records and local artifacts are evidence;
- canonical events explain change;
- domain records own accepted truth;
- AI output is derived until accepted under domain rules;
- derived views must be rebuildable where practical;
- answers and actions must cite source evidence.

## Domain Model

Макошь domains are not separate applications. They are ownership boundaries
inside one memory system.

| Domain | Product role | Source-of-truth responsibility |
|---|---|---|
| Communications | Main ingestion spine for messages, calls, meetings, participants and attachments. | Canonical interactions and source communication evidence. |
| Personas | Memory anchors for subjects: owner, people, AI agents, system actors and organization proxies. | Persona identity traces, Persona memory anchors and Persona relationships. |
| Organizations | Collective actor memory. | Organization identity, relationships, portals, procedures, playbooks and organization memory. |
| Projects | Bounded work contexts. | Project state, goals, linked context, decisions and project memory. |
| Documents | Evidence artifacts. | Document versions, extracted content, metadata and document evidence. |
| Knowledge | Evidence-backed understanding. | Reviewed facts, observations and knowledge items with provenance. |
| Decisions | Durable choices. | Rationale, evidence and affected entities for decisions. |
| Obligations | Commitments and duties. | Evidence-backed commitments, expected actions and follow-up state. |
| Tasks | Executable work. | Action lifecycle, task status, task evidence and provider overlays. |
| Events | Things that happened or are scheduled. | Append-only event facts and scheduled event records. |
| Relationships | First-class links between entities. | Typed, source-backed connections with confidence and validity. |

Boundary rule:

```text
Domains own durable truth.
Engines produce derived intelligence.
Agents operate over context.
```

## Engine Model

Engines are shared mechanisms. They do not own domain entities.

| Engine | Purpose | Output type |
|---|---|---|
| Memory Engine | Assemble durable, source-backed memory across domains. | memory views, context summaries, memory gaps |
| Timeline Engine | Build chronological views across entities. | timeline views, diffs, period summaries |
| Trust Engine | Assess relationship and source reliability. | trust signals, confidence adjustments |
| Search Engine | Retrieve source-backed context. | ranked results, snippets, retrieval plans |
| Enrichment Engine | Propose additional knowledge from approved sources. | candidates, observations, conflicts |
| Obligation Engine | Detect commitments, duties and follow-ups. | obligations, task candidates, follow-up candidates |
| Risk Engine | Detect evidence-backed risks and attention signals. | risk observations, attention views |
| Consistency / Contradiction Engine | Detect conflicts between new evidence and accepted memory. | contradiction observations and review items |

### Consistency / Contradiction Engine

The user-facing alias for this engine is Polygraph.

The engine compares new evidence against accepted memory. It detects
contradictions, stale facts, disputed claims, conflicting decisions and
mismatched obligations.

It must not call a person a liar and must not overwrite memory. It creates a
source-backed observation for review.

Example:

```text
New email: "We never approved budget X."
Existing Decision: "Budget X approved on 2026-05-14."
Output: ContradictionObservation linked to Decision, Communication, Project and Personas.
```

Required observation fields:

- old source;
- new source;
- affected entities;
- conflict type;
- confidence;
- review state.

## Current Implementation Inventory

This inventory is based on current repository files.

### Backend Domains And Modules

The backend currently has domain modules for:

- calendar;
- communications;
- decisions;
- documents;
- graph;
- obligations;
- organizations;
- persons;
- projects;
- relationships;
- review;
- signal_hub;
- tasks.

The backend also exports `domains/settings`, but its current module file is
empty. Working application settings logic lives under `platform/settings`.

The backend also has AI, engines, integrations, platform and workflow modules.

Notable integrations:

- Mail;
- Ollama;
- Omniroute;
- Telegram;
- WhatsApp;
- Zoom.

Platform support exists for:

- event log;
- audit log;
- capabilities;
- calls and transcripts;
- observations;
- secrets;
- settings;
- storage;
- host vault.

### Persistence Baseline

Current migrations include storage for:

- event log and projection cursors;
- communication provider accounts, raw records and canonical messages;
- mail blob and attachment metadata;
- documents and document processing jobs;
- graph nodes, edges and evidence;
- first-class relationships and relationship evidence;
- first-class decisions, decision evidence and impacted entity links;
- first-class obligations, obligation evidence and task links;
- projects and project link reviews;
- task candidates and tasks;
- persons compatibility tables and person memory tables;
- organizations and organization memory/workflow tables;
- calendar accounts, events, meetings, deadlines, focus blocks and rules;
- Telegram accounts, chats, messages, policies, calls and transcripts;
- WhatsApp Web sessions and messages;
- application settings, secret references, encrypted vault entries and host vault support;
- AI runtime, semantic embeddings and AI control center tables.

### API Surface Baseline

Routes are currently registered centrally in `backend/src/app/router.rs`.

Implemented route groups include:

- `/api/v1/communications/*`;
- `/api/v1/graph/*`;
- `/api/v1/projects/*`;
- `/api/v1/documents/*` and `/api/v1/document-processing/*`;
- `/api/v1/persons/*`;
- `/api/v1/calendar/*`;
- `/api/v1/organizations/*`;
- `/api/v1/tasks/*` and `/api/v1/task-candidates/*`;
- `/api/v1/settings/*`;
- `/api/v1/ai/*`;
- `/api/v1/integrations/telegram/*`;
- `/api/v1/integrations/whatsapp/*`;
- `/api/v1/policies/*`;
- `/api/v1/calls/*`;
- `/api/v1/integrations/mail/accounts/*`;
- `/api/v1/events/*` and `/api/v1/audit/events`.

This route list is implementation evidence only. It is not the target product
model.

### Frontend Surface Baseline

The frontend currently has page surfaces for:

- Agents;
- Calendar;
- Communications;
- Documents;
- Home;
- Knowledge;
- Notes;
- Organizations;
- Persons;
- Projects;
- Settings;
- Tasks;
- Telegram;
- Timeline;
- WhatsApp.

Several surfaces still use compatibility names such as Persons, Notes, health or
watchtower. Those names must be interpreted through the foundation glossary and
future product roadmap.

## Target Gaps And Refactoring Direction

The current implementation is meaningful but not yet fully aligned with the
target product model.

| Gap | Current evidence | Direction |
|---|---|---|
| Persona-native model incomplete | `persons`, `person_id`, `person_roles`, `person_personas`, `person_promises` and `/api/v1/persons/*` still exist. Owner Persona, PersonaType, Persona-native read/write compatibility bridge per ADR-0090, role-to-Relationship, interaction-context-to-Preference, enrichment trust-to-Relationship, notes-to-memory-card, favorite-to-preference, watchlist-to-preference, risk-to-health-cache, Dossier section adapters and reviewable Dossier snapshots have compatibility baselines. | Keep compatibility short-term. Plan physical Persona-native schema migration under a dedicated migration ADR. |
| Owner Persona partially implemented | Migration `0059` adds `is_self` uniqueness and `person_type` constraints on the compatibility `persons` table, and GET/PUT `/api/v1/persons/owner` exposes the compatibility Owner Persona route. Agents and UI still need to consistently route owner-scoped context through that Owner Persona. | Wire agent attribution and context assembly to the Owner Persona before expanding autonomous actions. |
| First-class Relationships partially implemented | Migrations `0060`, `0061` and `0068` plus `backend/src/domains/relationships/` add first-class Relationship persistence with evidence, trust score, strength score, confidence, review state, graph projection for all current Relationship entity kinds, and guarded entity/global review routes. Manual/API person roles now materialize source-backed `has_role` Relationships from Persona to role Knowledge anchors and demote those Relationships to `user_rejected` when the role is removed. Manual/API and email-sync organization contact links now materialize source-backed `member_of` Relationships from Persona to Organization. Manual task relations now materialize source-backed Relationships from Task to known target entity kinds. Explicit project link reviews now materialize source-backed Relationships from Project to reviewed Communication or Document and demote th
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/product/product-charter.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/product/product-charter.md`
- Size bytes / Размер в байтах: `2282`
- Included characters / Включено символов: `2282`
- Truncated / Обрезано: `no`

```markdown
# Product Charter

## Purpose

Макошь creates a personal operational memory layer around communications,
documents, projects, relationships, decisions and obligations. The product helps
the owner understand what happened, what matters, what requires action and how
entities connect.

See the canonical foundation documents:

- [Foundation Vision](../foundation/vision.md)
- [World Model](../foundation/world-model.md)
- [Architecture Principles](../foundation/architecture-principles.md)

## User

The primary user is one technically strong owner who manages personal and
professional communications, documents, projects, relationships, obligations and
knowledge. Макошь is a personal system first; architecture should not block
future family/team modes, but those modes are not the current product identity.

The owner is represented inside the world model by the Owner Persona.

## Core Scenarios

- unified communication context across channels;
- extraction and tracking of obligations and task candidates;
- source-backed search across memory;
- history of relationships with a Persona or Organization;
- linking documents to Projects, Personas, Organizations, Events, Tasks,
  Decisions and Obligations;
- AI-assisted triage with user control;
- analysis of changes over time;
- context preparation before meetings or actions;
- explanation of why Макошь produced a conclusion.

## Product Constraints

- The system is not optimized for a quick MVP.
- Implementation may be incremental, but documentation should describe the
  target model.
- Cloud providers are optional integrations, not the memory layer.
- Personal data is not used for fine-tuning.
- AI features must degrade safely when a model is unavailable.
- Every automatic conclusion must preserve provenance.

## Product Quality

Макошь should feel like a serious personal operating environment, not a
dashboard collection. The UI should be fast, dense, keyboard-first and
contextual.

## Quality Metrics

- ingestion completeness for connected sources;
- identity resolution quality for Personas and Organizations;
- context retrieval latency;
- accuracy of obligation/task extraction;
- share of AI answers with sufficient provenance;
- backup/restore success;
- manual actions required per common workflow.
```

### `docs/product/product-scope.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/product/product-scope.md`
- Size bytes / Размер в байтах: `3511`
- Included characters / Включено символов: `3511`
- Truncated / Обрезано: `no`

```markdown
# Product Scope

## In Scope

### Communications

- Email, Telegram, WhatsApp and calls as communication channels.
- Provider source preservation.
- Canonical messages, conversations, participants, attachments and delivery
  metadata.
- Relevance, spam, marketing and risk classification as engine output.

### Personas

- Persona identity traces.
- Relationship history and graph neighborhood.
- Communication context.
- Persona memory and dossier.
- Identity resolution and reviewed merge/split workflows.

### Organizations

- Organizations as first-class entities.
- Organization identities, domains, portals, procedures and playbooks.
- Relationships to Personas, Projects, Documents, Communications and
  Obligations.

### Projects

- Bounded work contexts.
- Linked communications, documents, decisions, tasks, obligations and Personas.
- Project context and timeline views through engines.

### Documents

- PDF, Office, images and Markdown.
- Versioning, extraction, OCR, metadata, summaries and entity mentions.
- Links to other world-model entities.

### Tasks And Obligations

- Task candidates extracted from evidence.
- Actionable Tasks with lifecycle.
- Obligations as commitments or duties with evidence.
- Follow-Ups as prompts that may become Tasks or remain reminders.

### Events And Timeline

- Canonical system events.
- Calendar/meeting events.
- Timeline Engine views over source-backed events and domain records.

### Knowledge And Graph

- First-class relationships with provenance.
- Evidence-backed facts, decisions and observations.
- Graph-aware context assembly.

### Engines

- Memory Engine.
- Timeline Engine.
- Trust Engine.
- Search Engine.
- Enrichment Engine.
- Obligation Engine.
- Risk Engine.
- Consistency / Contradiction Engine, user-facing alias Polygraph.

### Agents

- HESTIA as coordinator.
- Specialized agents for communications, memory, analysis and tool automation.
- Typed tools, explicit permissions and source-backed context.

## Out Of Scope For Initial Implementation, But Architecturally Supported

- multi-user SaaS;
- enterprise CRM workflows;
- public API marketplace;
- global cloud sync as a required dependency;
- end-to-end encrypted multi-device sync;
- autonomous external actions without explicit permission policy.

These items must not dictate the current implementation, but the architecture
should not unnecessarily block future work.

## Non-Goals

- replace Gmail, Telegram, WhatsApp or calendars as network providers;
- train a personal LLM on private data;
- store only embeddings without source evidence;
- hide automatic decisions without provenance;
- create one universal activity table.

## Capability Map

| Capability | Core entities | Primary output |
|---|---|---|
| Communication ingestion | Communication, Source record, Event, Persona | normalized events and messages |
| Persona memory | Persona, Relationship, Knowledge item | source-backed Persona context |
| Organization memory | Organization, Relationship, Document | organization context |
| Document understanding | Document, Version, Entity mention | indexed and linked evidence |
| Obligation extraction | Obligation, Task candidate, Source record | reviewed commitments and actions |
| Search and recall | Source record, Entity, Relationship, Event | ranked source-backed results |
| Agent orchestration | Agent Persona, Tool, Context | explainable AI workflows |
| Project memory | Project, Task, Document, Decision, Obligation | project context and timeline |
```

### `docs/refactoring/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/README.md`
- Size bytes / Размер в байтах: `582`
- Included characters / Включено символов: `582`
- Truncated / Обрезано: `no`

```markdown
# Refactoring

Status: documentation package aligned to the current repository structure.

This package tracks known implementation-alignment gaps and migration planning.
It is not a replacement for ADRs when an architectural decision changes.

## Navigation

- [Implementation Alignment Plan](./implementation-alignment-plan.md)
- [Documentation Code Alignment Report](./documentation-code-alignment-report.md)
- [Product Alignment Plan](./product-alignment-plan.md)
- [Naming Conflicts Inventory](./naming-conflicts-inventory.md)
- [UI States Inventory](./ui-states-inventory.md)
```

### `docs/refactoring/documentation-code-alignment-report.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/documentation-code-alignment-report.md`
- Size bytes / Размер в байтах: `5416`
- Included characters / Включено символов: `5416`
- Truncated / Обрезано: `no`

```markdown
# Documentation Code Alignment Report

Status: current audit report for documentation/code alignment.

Date: 2026-06-28

Scope: documentation structure under `docs/` compared with current
`backend/src` and selected `frontend/src` packages. ADR remains the source of
truth where code and docs disagree.

## ADR Applied

- [ADR-0073 Backend Module Organization](../adr/ADR-0073-backend-module-organization.md)
- [ADR-0054 Application Settings Store](../adr/ADR-0054-application-settings-store.md)
- [ADR-0081 Opt-In OmniRoute AI Runtime](../adr/ADR-0081-opt-in-omniroute-ai-runtime.md)
- [ADR-0096 Canonical Evidence, Review Inbox and Context Packs](../adr/ADR-0096-canonical-evidence-review-and-context-packs.md)
- [ADR-0097 Communications Channel Domains To Integrations](../adr/ADR-0097-communications-channel-domains-to-integrations.md)
- [ADR-0098 Provider-Neutral Communications API And Strict Boundaries](../adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md)
- [ADR-0102 Zoom Provider Runtime Boundary](../adr/ADR-0102-zoom-provider-runtime-boundary.md)

## Verified Backend Inventory

Current top-level backend code areas:

- `ai`;
- `app`;
- `application`;
- `bin`;
- `domains`;
- `engines`;
- `integrations`;
- `platform`;
- `vault`;
- `workflows`.

Current exported domain directories:

- `calendar`;
- `communications`;
- `decisions`;
- `documents`;
- `graph`;
- `obligations`;
- `organizations`;
- `persons`;
- `projects`;
- `relationships`;
- `review`;
- `settings`;
- `signal_hub`;
- `tasks`.

Current backend has an empty `backend/src/domains/settings/mod.rs`. Settings is
core application surface, not a current product domain; the working settings
implementation is under `platform/settings`.

Current exported engine modules:

- `automation`;
- `consistency`;
- `context_packs`;
- `enrichment`;
- `identity_resolution`;
- `memory`;
- `obligation`;
- `relationships`;
- `risk`;
- `search`;
- `timeline`;
- `trust`.

Current exported integrations:

- `mail`;
- `ollama`;
- `omniroute`;
- `telegram`;
- `whatsapp`;
- `zoom`.

Current workflow directories:

- `email_intelligence`;
- `email_sync_pipeline`;
- `graph_projection`;
- `mail_background_sync`;

`backend/src/workflows/mod.rs` also exports additional workflow modules that
are single files rather than directories.

## Documentation Updates Made

Added doc packages for code/ADR-backed gaps:

- [Review Domain](../domains/review/README.md);
- [Automation Engine](../engines/automation/README.md);
- [Context Packs Engine](../engines/context-packs/README.md);
- [Identity Resolution Engine](../engines/identity-resolution/README.md);
- [Relationship Candidate Engine](../engines/relationships/README.md);
- [Ollama Integration](../integrations/ollama/README.md);
- [OmniRoute Integration](../integrations/omniroute/README.md);
- [Application Settings](../platform/settings/README.md).

Updated central catalogs to reference those packages and current backend
inventory.

## Confirmed Boundaries

- Review is a domain by ADR-0096 and current backend code.
- Context Packs, Identity Resolution and Relationship Candidate behavior are
  engines by ADR-0096.
- Mail, Telegram, WhatsApp and Zoom are integrations, not product domains, per
  ADR-0097, ADR-0098 and ADR-0102.
- Ollama and OmniRoute are AI runtime integrations. Ollama is local default;
  OmniRoute is explicit opt-in.
- Settings implementation currently lives in the platform layer and frontend
  Settings UI. It should not be documented as accepted product-domain truth
  unless a later ADR changes ownership.

## Future Gaps

The following items remain future gaps or ownership decisions:

- `docs/domains/agents/` has canonical product language, and
  `frontend/src/domains/agents` exists, but there is no current
  `backend/src/domains/agents` package. Backend AI/agent implementation is
  split across `backend/src/ai`, app routes and settings/control surfaces.
- `docs/domains/notes/` documents Notes as document-like artifacts. There is a
  frontend `frontend/src/domains/notes` package, but no backend domain package
  and no ADR found in this pass that promotes Notes to a first-class domain.
- `docs/workflows/` is currently product-workflow documentation. It does not
  yet mirror each concrete backend workflow module. I did not create empty
  workflow package files because several workflow modules need a separate
  ownership pass.
- Frontend has domain packages such as `knowledge`, `home`, `timeline`,
  `personas` and `settings` that do not map one-to-one to backend domain
  directories. These appear to be UI/workspace packages rather than backend
  domain ownership, but that mapping was not fully audited in this pass.

## Follow-Up Candidates

- Add a workflow-module documentation pass for concrete modules under
  `backend/src/workflows` without replacing product workflow specs.
- Add a frontend-domain-to-backend-domain mapping document if UI workspace
  package boundaries keep diverging from backend domain ownership.

## Validation

Checks run for this report:

- `node scripts/check-architecture.mjs` passed.
- `git diff --check` passed.
- Markdown local-link checker passed for 315 docs markdown files.
- `make test-architecture` passed 26/26 architecture tests.
- Documentation/code package comparison found expected open items only:
  extra docs for `agents` and `notes` because they are not current backend
  domain packages.
```

### `docs/refactoring/implementation-alignment-plan.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/implementation-alignment-plan.md`
- Size bytes / Размер в байтах: `32911`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```markdown
# Implementation Alignment Plan

This document maps the current repository implementation to the canonical
Макошь product model.

It is a planning document only. It does not authorize code changes, route
renames, schema migrations or API redesign without a follow-up implementation
task and ADR where required.

## Target Model

Canonical references:

- [Product Master Spec](../product/master-spec.md)
- [Domain Catalog](../domains/README.md)
- [Engine Catalog](../engines/README.md)
- [Workflow Catalog](../workflows/README.md)
- [ADR-0084 Persona Intelligence System](../adr/ADR-0084-persona-intelligence-system.md)
- [ADR-0085 Communication Spine and Consistency / Contradiction Engine](../adr/ADR-0085-communication-spine-and-contradiction-engine.md)
- [ADR-0086 First-Class Relationship Persistence](../adr/ADR-0086-first-class-relationship-persistence.md)
- [ADR-0087 Contradiction Observation Persistence](../adr/ADR-0087-contradiction-observation-persistence.md)
- [ADR-0088 Obligation Persistence](../adr/ADR-0088-obligation-persistence.md)

Макошь is a local-first Personal Memory System. Communications are the primary
ingestion spine. Domains own source-of-truth entities. Engines produce derived
views, candidates, scores and review items.

## Current Implementation Evidence

The current backend has these relevant surfaces:

- route registration in `backend/src/app/router.rs`;
- domain modules under `backend/src/domains/`;
- search and automation modules under `backend/src/engines/`;
- workflow modules under `backend/src/workflows/`;
- provider integrations under `backend/src/integrations/`;
- migrations `0001` through `0069`;
- frontend pages under `frontend/src/lib/pages/`.

## Documentation Drift Corrected

This alignment pass corrected documentation that conflicted with current
implementation evidence:

- `docs/integrations/mail/modules.md` now maps to actual files under
  `backend/src/domains/communications/`, `backend/src/workflows/` and
  `backend/src/integrations/`.
- `docs/domains/calendar/architecture.md` now maps to actual files under
  `backend/src/domains/calendar/`.
- `docs/domains/tasks/architecture.md` now maps to actual files under
  `backend/src/domains/tasks/`.
- `docs/domains/organizations/architecture.md` now maps to actual files under
  `backend/src/domains/organizations/`.
- `docs/domains/tasks/api.md` now uses the current router base `/api/v1`, not the stale
  `/api/v2` value.
- Channel/status docs now clarify that implementation coverage percentages are
  local surface coverage, not product-wide completion of Memory, Knowledge,
  Obligations, Decisions or Polygraph.
- `docs/architecture/security-model.md` now follows ADR-0056 and current code:
  `MAKOSH_LOCAL_API_SECRET` plus `X-Макошь-Secret` are current; token/actor-id
  headers are historical terms from superseded ADRs.
- `docs/architecture/context-diagram.md` and
  `docs/architecture/container-diagram.md` now show Макошь as the Personal
  Memory System with Communications, Events, Documents, shared Engines and the
  Owner Persona.
- `docs/reviews/backend-architecture-review-2026-06-06.md` is explicitly marked
  as a historical review rather than the current implementation map.
- Root, backend and frontend README files now distinguish the current host vault
  from legacy database-vault compatibility, describe email networking under
  ADR-0055 read/write capability boundaries, and use Persona-compatible identity
  wording instead of target-level Contact terminology.
- Provider runtime/setup/account-control APIs now live under
  `/api/v1/integrations/*`, while product/business Communications read/write
  APIs remain under `/api/v1/communications/*`.
- `docs/integrations/zoom/` and ADR-0102 now document the Zoom provider foundation. The
  current checkout contains `backend/src/integrations/zoom`,
  `frontend/src/integrations/zoom`, migration `0160` and
  `/api/v1/integrations/zoom/*` routes; ADR-0102 is accepted after the full
  backend/frontend validation gate passed.
- Vault ownership is now constrained to host vault lifecycle, encrypted secret
  payload storage, secret references, resolver contracts and provider session
  secret storage. Domain-owned provider account/account-binding stores live
  under their owning domains instead of `backend/src/vault/`.
- Workflow coordination is being tightened around explicit command/query ports
  rather than concrete store imports, and communications-domain generic
  `Mail*` naming is being retired incrementally in favor of neutral
  `Communication*` symbols.
- `backend/migrations/0059_persona_owner_type_constraints.sql` and
  `backend/src/domains/persons/api.rs` now provide the first compatibility-layer
  implementation of `PersonaType` and single Owner Persona semantics on the
  existing `persons` table.
- `backend/src/domains/persons/handlers/mod.rs` and `backend/src/app/router.rs`
  expose GET/PUT `/api/v1/persons/owner` as the compatibility route for reading
  and assigning the current Owner Persona.
- `/api/v1/ai/agents` now materializes registry-backed AI agents (`HESTIA`,
  `MAKOSH`, `MNEMOSYNE`, `ATHENA`, `HEPHAESTUS`) as `persona_type = ai_agent`
  Personas and graph nodes. Compatibility email identities use lowercase agent
  IDs at `sh-inc.ru`, such as `hestia@sh-inc.ru`.
- `ai_agent_runs` now stores `agent_persona_id` and `owner_persona_id`
  attribution for service-created AI runs when an Owner Persona exists.
- `backend/migrations/0071_person_identity_trace_types.sql` extends
  compatibility `person_identities` to accept `document_mention` and
  `message_participant` Persona identity traces.
- `backend/migrations/0072_person_identity_disputed_status.sql` extends
  compatibility `person_identities` to accept `disputed` identity trace status.
- `backend/migrations/0073_person_identity_unattached_traces.sql` and
  `PersonsIdentityStore::create_unattached` / `attach_to_persona` provide the
  first backend workflow for identity traces that exist before Persona
  assignment.
- `/api/v1/identity-traces` now exposes guarded compatibility create/list
  routes for unattached identity traces, and
  `/api/v1/identity-traces/{identity_id}/assignment` attaches a trace to a
  Persona.
- `backend/migrations/0060_create_relationships.sql` and
  `backend/src/domains/relationships/mod.rs` now provide the first durable
  Relationship persistence baseline with evidence, trust score, strength score,
  confidence and review state.
- `backend/migrations/0061_relationship_graph_projection.sql` now connects
  active Persona-to-Persona Relationship records to graph traversal through
  generic `entity_relationship` graph edges.
- `backend/src/domains/relationships/api.rs` now exposes guarded backend routes
  for listing Relationships by entity and changing review state while keeping
  active Persona-to-Persona graph projections aligned.
- `backend/migrations/0062_create_contradiction_observations.sql` and
  `backend/src/engines/consistency.rs` now provide the first
  Consistency / Contradiction Engine baseline: structured direct-contradiction
  detection and reviewable `ContradictionObservation` persistence.
- `ContradictionObservationStore::refresh_deterministic_observations` now adds
  the first Communication/Document/Event-to-Polygraph refresh paths by
  comparing active `person_facts` Memory claims with structured claims
  extracted from projected Communication message subject/body evidence matched
  by Persona email sender, imported Document title/extracted-text evidence that
  references the Persona email, meeting-note content linked through event
  participants and successful call transcript text linked through active
  Telegram identity.
- `backend/src/app/handlers/consistency.rs` now exposes guarded backend routes
  for listing open contradiction observations and changing review state without
  automatically overwriting Memory.
- `backend/migrations/0063_create_obligations.sql` and
  `backend/src/domains/obligations/mod.rs` now provide the first source-backed
  Obligation persistence baseline with evidence, status, review state, risk
  state, confidence and task links.
- `backend/migrations/0064_create_decisions.sql` and
  `backend/src/domains/decisions/mod.rs` now provide the first source-backed
  Decision persistence baseline with evidence, rationale, alternatives, review
  state, confidence and impacted entity links.
- `backend/src/domains/decisions/api.rs` now exposes guarded backend routes for
  listing accepted Decisions by entity and changing accepted Decision review
  state without changing Projects, Tasks or Obligations.
- `backend/src/engines/obligation/` now provides the first Obligation Engine
  candidate detection baseline for explicit commitment and request language.
- `backend/src/domains/tasks/candidates.rs` now uses the Obligation Engine when
  refreshing message task candidates for explicit commitment/request language
  that the legacy task scanner does not match.
- `backend/migrations/0067_task_candidate_kind_metadata.sql` and
  `backend/src/domains/tasks/candidates.rs` now classify obligation-derived
  task candidates and materialize confirmed `obligation_task` candidates into
  source-backed `user_confirmed` Obligations linked to the created Task through
  `fulfillment_task`. Resetting or rejecting an obligation-derived task
  candidate now synchronizes the durable Obligation review state instead of
  leaving stale `user_confirmed` Obligations behind. Task candidate refresh is
  also idempotent across the source/title identity enforced by the database.
- `backend/src/workflows/email_sync_pipeline.rs` now refreshes explicit
  Decision candidates and obligation-derived task candidates for projected
  email Communication messages in the current sync batch. It creates reviewable
  candidates only and does not auto-create Tasks, Projects or accepted
  Obligations.
- `backend/src/domains/tasks/candidates.rs` now also applies the Obligation
  Engine to explicit document commitments when refreshing document task
  candidates. This creates reviewable `obligation_task` candidates with
  document evidence only and does not auto-create Tasks or accepted Obligations.
- `backend/src/integrations/telegram/client.rs` and
  `backend/src/integrations/whatsapp/client.rs` now refresh explicit Decision
  candidates and obligation-derived task candidates for newly projected fixture
  provider Communications. They create reviewable candidates only and do not
  auto-create Tasks, Projects or accepted Obligations.
- `backend/src/domains/obligations/api.rs` now exposes guarded backend routes
  for listing accepted Obligations by entity and changing accepted Obligation
  review state without creating Tasks.
- `backend/src/domains/decisions/mod.rs` now refreshes explicit Communication
  message and imported Document Decision candidates into source-backed
  `suggested` Decisions and preserves reviewed Decision state across repeat
  refreshes.
- `backend/src/domains/calendar/meetings.rs` now adapts meeting outcomes into
  reviewable domain records: `decision` outcomes create source-backed
  `suggested` Decisions impacted by the meeting Event, and `promise`, `task`
  and `follow_up` outcomes create source-backed `suggested` Obligations without
  creating Tasks.
- `backend/src/domains/persons/trust.rs` now adapts compatibility
  `person_promises` records into source-backed `user_confirmed` Obligations
  with `raw_record` evidence and without creating Tasks.
- `backend/src/domains/projects/link_reviews.rs` now adapts explicit project
  link review decisions into source-backed `user_confirmed` Decisions impacted
  by the Project and reviewed Communication or Document.
- `backend/src/domains/projects/link_reviews.rs` now also adapts explicit
  project link reviews into source-backed Relationships from Project to the
  reviewed Communication or Document. Resetting an explicit project link review
  demotes the durable Relationship candidate back to `suggested` instead of
  leaving
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/refactoring/naming-conflicts-inventory.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/naming-conflicts-inventory.md`
- Size bytes / Размер в байтах: `6589`
- Included characters / Включено символов: `5724`
- Truncated / Обрезано: `no`

````markdown
# Инвентаризация naming conflicts: Persons ↔ Personas

> Создано: 2026-06-14 в рамках Phase 1 (Foundation & Safety Net)
> Цель: Задокументировать все naming conflicts перед Phase 2 (Persona Naming Alignment)

## 1. Двойное именование: Persons ↔ Personas

### 1.1 API Routes

| Route | Файл | Статус |
|-------|------|--------|
| `/api/v1/persons` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:5) | Legacy |
| `/api/v1/personas` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:6) | Native |
| `/api/v1/persons/owner` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:12) | Legacy |
| `/api/v1/persons/{person_id}` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:15) | Legacy |
| `/api/v1/personas/{persona_id}` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:8) | Native |
| `/api/v1/persons/{person_id}/personas` | [`backend/src/app/router/routes/persons.rs`](../../backend/src/app/router/routes/persons.rs:62) | Mixed |

**Итог:** 90% routes используют `/api/v1/persons/`, 10% используют `/api/v1/personas/`. Оба набора routes работают параллельно.

### 1.2 Database Schema

| Таблица | Колонка | Статус |
|---------|---------|--------|
| `persons` | `person_id` | Legacy |
| `persons` | `is_self` | Допустимо для Persona Owner |
| `personas` | `persona_id` | Native |
| `personas` | `person_id` (FK → persons) | Mixed |

### 1.3 Backend Module Names

| Модуль | Статус |
|--------|--------|
| `domains/persons/` | Legacy — основной domain модуль |
| `domains/persons/core/` | Mixed — содержит `PersonPersona` и `PersonsIdentityStore` |
| `domains/persons/api/` | Legacy — `PersonProjectionStore` |
| `domains/persons/handlers/` | Legacy — но содержит persona handlers |
| `domains/persons/api/store/persona_reads.rs` | Native |
| `domains/persons/api/store/persona_writes.rs` | Native |
| `domains/persons/api/store/persona_type.rs` | Native |

### 1.4 Rust Types

| Тип | Статус |
|-----|--------|
| `PersonPersona` | Mixed — тип называется Persona, но префикс Person |
| `NewPersonPersona` | Mixed |
| `PersonPersonaStore` | Mixed |
| `PersonsIdentityStore` | Legacy |
| `PersonProjectionStore` | Legacy |
| `PersonIdentity` | Legacy |
| `PersonRole` | Legacy |

### 1.5 Frontend Module Names

| Модуль/файл | Статус |
|-------------|--------|
| `frontend/src/domains/personas/` | Native |
| `frontend/src/domains/personas/api/personas.ts` | Mixed — экспортирует `fetchPersons()` и `fetchOrganizations()` |
| `frontend/src/domains/personas/queries/usePersonasQuery.ts` | Native |
| `frontend/src/domains/personas/types/persona.ts` | Native |

### 1.6 Frontend API Functions

| Функция | Статус |
|---------|--------|
| `fetchPersons()` | Legacy — идёт к `/api/v1/persons` |
| `fetchPersonDossier()` | Legacy |
| `fetchIdentityCandidates()` | Legacy |
| `fetchRelationships()` | Отдельный domain |
| `fetchOrganizations()` | Отдельный domain (но находится в personas/api) — **cross-domain** |

### 1.7 Compatibility Layer

Файл: [`backend/src/domains/persons/handlers/compatibility.rs`](../../backend/src/domains/persons/handlers/compatibility.rs)

Содержит хендлеры для `/api/v1/personas/` (native routes), которые преобразуют данные из persons-ориентированной модели в persona-ориентированную.

## 2. SemanticSourceKind → "contact"

В [`backend/src/ai/core/semantic/sources.rs`](../../backend/src/ai/core/semantic/sources.rs): `SemanticSourceKind::Person` сериализуется как `"contact"`.

```rust
pub enum SemanticSourceKind {
    Person,     // → "contact"
    Document,   // → "document"
    Email,      // → "email"
    Task,       // → "task"
    Note,       // → "note"
}
```

Это legacy naming, несовместимое с Persona-моделью. Требует изменения в Phase 2.

## 3. Cross-domain imports

### 3.1 persons → organizations

Файл: [`frontend/src/domains/personas/api/personas.ts`](../../frontend/src/domains/personas/api/personas.ts)

Содержит `fetchOrganizations()` и `fetchOrganization()` — функции, относящиеся к organizations domain, но находящиеся в personas/api.

### 3.2 review → persons + tasks + knowledge

Файл: [`frontend/src/domains/review/stores/review.ts`](../../frontend/src/domains/review/stores/review.ts)

Импортирует из:
- `../../personas/api/personas` (relationships)
- `../../tasks/api/tasks` (decisions, obligations)
- `../../knowledge/api/knowledge` (contradictions)

### 3.3 organizations queries → persons

Файл: [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts)

Импортирует `fetchOrganizations` и `fetchOrganization` из `../../personas/api/personas`.

## 4. Communications module naming

| Файл/модуль | Проблема |
|-------------|----------|
| `domains/communications/` | Название `mail` вместо `communications` |
| `backend/src/domains/communications/` | ~100+ файлов в God-директории |
| `/api/v1/communications/*` | API уже использует правильное имя |
| `frontend/src/domains/communications/` | Frontend уже использует правильное имя |

**Итог:** Backend domain называется `mail`, но API роуты и фронтенд используют `communications`. Это несоответствие требует рефакторинга.

## 5. Резюме для Phase 2

Приоритетные изменения:
1. Переименовать `domains/persons/` → `domains/personas/` (или создать facade)
2. `SemanticSourceKind::Person` → `"persona"` (не `"contact"`)
3. Перенести `fetchOrganizations` из personas/api в organizations/api
4. Устранить cross-domain imports в review store
5. Начать рефакторинг `domains/communications/` → `domains/communications/`
````

### `docs/refactoring/product-alignment-plan.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/product-alignment-plan.md`
- Size bytes / Размер в байтах: `10728`
- Included characters / Включено символов: `10728`
- Truncated / Обрезано: `no`

````markdown
# Product Alignment Refactoring Plan

Date: 2026-06-12

Scope: documentation-derived product and implementation alignment plan.

This document records where the current implementation differs from the Product
Master Spec target model and what refactoring or delivery plans are needed.

It is not an implementation plan for code changes. Each implementation item
below requires its own ADR review, design or execution plan before code changes.

## Alignment Baseline

Target product model:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
  -> Timeline / Dossier / Recall
```

Current implementation already includes communication ingestion, mail workflows,
Telegram and WhatsApp foundations, graph projection, documents, projects,
persons compatibility, organizations, calendar, tasks, AI runtime, settings and
vault support.

The gaps below are about target-model alignment, not lack of useful
implementation.

## Product Alignment Gaps

| Gap | Current evidence | Target direction | Plan type |
|---|---|---|---|
| Communications still read as email-heavy in code and docs. | `backend/src/domains/communications`, `docs/integrations/mail/`, many `/api/v1/communications/*` routes backed by email modules. | Communications is the domain; email is one channel. | Documentation first, implementation naming later. |
| Persona model is compatibility-based. | `persons`, `person_id`, `person_roles`, `person_personas`, `person_promises`, `/api/v1/persons/*`. Owner Persona, PersonaType, role-to-Relationship and interaction-context-to-Preference adapters have baselines. | Persona, Owner Persona, PersonaType and first-class Relationships. | ADR and migration plan before schema/API rename. |
| Relationships are fragmented. | Graph edges, organization contact links, task relations, project link reviews, relationship events and person roles coexist. First-class Relationship persistence, graph projection for all current Relationship entity kinds, guarded entity/global review routes, manual/API person role adapters, manual/API and email-sync organization contact link adapters, manual task relation adapters, project link review adapters and Personas workspace review have a baseline. | Relationship is first-class with type, confidence, provenance, trust and validity. | Continue remaining relationship-shaped compatibility adapter work and cross-domain review workflow placement. |
| Polygraph engine is partially implemented. | Migration `0062`, `backend/src/engines/consistency.rs`, `backend/src/engines/consistency/`, `backend/src/app/handlers/consistency.rs` and `backend/src/application/consistency_review.rs` provide structured direct-contradiction detection, deterministic structured and limited natural-language `location` / `status` claim extraction from Communication/Document/Event evidence text, reviewable observations, guarded backend review routes, Knowledge workspace review UI and projected email/Telegram/WhatsApp message/imported Document/meeting-note/call-transcript refresh against active `person_facts`. Broad natural-language extraction and broader provider evidence are incomplete. | Cross-domain engine for contradiction observations and review items. | Expand ingestion refresh to broader provider evidence, then add reviewed-outcome semantics. |
| Decisions and Obligations are partial. | Migrations `0063`, `0064`, `0065`, `0066` and `0067` plus `backend/src/domains/obligations/` and `backend/src/domains/decisions/` provide source-backed persistence, accepted graph projection and task-candidate classification for obligation-derived candidates. `backend/src/engines/obligation/` provides a first obligation candidate detector, `backend/src/domains/decisions/extraction/` provides a first explicit-decision candidate detector, message and document task candidate refresh use Obligation detection for explicit commitments/requests, confirmed `obligation_task` candidates materialize source-backed Obligations linked to Tasks, reset/reject review on those candidates synchronizes durable Obligation state, email sync and Telegram/WhatsApp fixture ingestion refresh reviewable Decision and obligation-derived task candidates for projected Communications, compatibility `person_promises` materialize source-backed `user_confirmed` Obligations, explicit message/imported-document Decision candidates persist as source-backed `suggested` Decisions, project link reviews materialize source-backed `user_confirmed` Decisions, meeting outcomes create reviewable Decisions or Obligations for `decision`, `promise`, `task` and `follow_up` outcomes, accepted Obligations/Decisions have guarded backend entity/global list/review routes, and the Tasks workspace has a global suggested review panel. Candidate-to-domain routing still needs broader workflow coverage. | Durable Decisions and Obligations with evidence and review. | Expand ingestion wiring, review workflows and compatibility adapters. |
| Engine ownership is partly embedded in domains. | Health/watchtower, intelligence, enrichment and timeline-like modules appear in domain folders. | Engines are reusable mechanisms; domains own durable truth. | Engine spec wave before refactoring. |
| Notes are ambiguous. | Frontend has Notes page; foundation treats Notes as document-like artifacts. | Notes remain lightweight document artifacts unless a future ADR promotes them. | Documentation clarification; no implementation change yet. |
| UI vocabulary exposes compatibility names. | Frontend pages include Persons, Notes, Timeline and domain-specific health/watchtower concepts. | UI should surface Personal Memory System concepts without hiding compatibility state. | UI vocabulary plan after product docs. |

## Refactoring And Delivery Plans To Create

### 1. Communications Normalization Plan

Goal: align Mail, Telegram, WhatsApp, calls and meetings under the
Communications product model.

Required scope:

- document channel-specific source boundaries;
- preserve provider-specific implementation modules;
- define canonical Communication lifecycle;
- identify which current mail-specific routes are compatibility names;
- avoid code renames until API/schema compatibility is planned.

### 2. Persona Migration Plan

Goal: move from `persons` compatibility toward the Persona target model.

Required scope:

- Owner Persona semantics;
- `PersonaType` values: `human`, `ai_agent`, `organization_proxy`, `system`;
- target identity trace model;
- `/persons` compatibility strategy;
- route/schema compatibility for remaining Persona root cache columns;
- migration safety and graph impact.

### 3. Relationship Model Plan

Goal: define first-class Relationship records across Personas, Organizations,
Projects, Documents, Communications, Tasks, Events, Decisions and Obligations.

Required scope:

- relationship type taxonomy;
- source and target entity references;
- confidence and provenance;
- trust and strength scores;
- validity period;
- review state for inferred relationships;
- integration with graph projection.

### 4. Polygraph Engine Plan

Goal: introduce Consistency / Contradiction Engine behavior.

Required scope:

- contradiction taxonomy;
- accepted memory inputs;
- new evidence inputs;
- `ContradictionObservation` target shape;
- review workflow;
- effect on Risk and Trust signals;
- source citation requirements;
- UI surface for contradiction review.

### 5. Decisions And Obligations Plan

Goal: separate durable Decisions and Obligations from Tasks, Follow-Ups,
Promises and meeting outcomes.

Required scope:

- Decision evidence and rationale model;
- Obligation evidence and lifecycle;
- Task creation from Obligations;
- Follow-Up as prompt, not always task;
- meeting outcome mapping;
- communication-to-obligation workflow.

### 6. Engine Boundary Plan

Goal: keep domain truth separate from reusable intelligence mechanisms.

Required scope:

- Memory Engine;
- Timeline Engine;
- Trust Engine;
- Search Engine;
- Enrichment Engine;
- Obligation Engine;
- Risk Engine;
- Consistency / Contradiction Engine;
- which current modules are domain-owned and which are engine-like.

### 7. UI Vocabulary Plan

Goal: align desktop surfaces with the Personal Memory System model.

Required scope:

- Personas vs Persons labeling;
- Notes as capture/document artifacts;
- Timeline as engine view;
- Health/watchtower as attention/risk views;
- Communications as the shared entry point;
- product navigation around Context, Memory and Action.

## Documentation Execution Order

1. Complete Product Spine.
2. Create Communications, Personas, Relationships and Knowledge domain specs.
3. Create Obligations, Tasks and Decisions specs.
4. Create Projects, Documents, Organizations and Events specs.
5. Create engine specs, including Polygraph.
6. Create workflow specs.
7. Only then write implementation migration plans for code/schema/API changes.

Wave 2 adds the active domain catalog under `docs/domains/` and creates missing
canonical domain documents for Communications, Organizations, Projects,
Calendar/Events, Decisions, Obligations, Agents and Notes. These documents are
documentation alignment only; they do not authorize code, route or schema
changes without a follow-up implementation plan and ADR where needed.

Wave 3 adds the active engine catalog under `docs/engines/` and creates detailed
specs for Memory, Timeline, Trust, Search, Enrichment, Obligation, Risk and
Consistency / Contradiction. The current code still has several domain-local
engine-like modules; this is a migration gap, not a target boundary.

Wave 4 adds the workflow catalog under `docs/workflows/` for
communication-to-knowledge, communication-to-obligation, meeting-to-decisions,
document-to-context, contradiction-review, dossier-generation and
agent-assisted-recall. These workflows coordinate domains and engines; they do
not define new APIs or authorize implementation changes by themselves.

Wave 5 adds `docs/refactoring/implementation-alignment-plan.md`, which maps the
current backend routes, domain modules, migrations and frontend surfaces to the
target model and splits future code work into safe refactoring slices.

## Current Non-Goals

- No code changes.
- No route renames.
- No schema migrations.
- No generated API design.
- No rewriting historical ADRs.

## Validation Expectation

Every future refactoring plan must include:

- implementation evidence inspected;
- target model reference;
- affected docs;
- affected modules, migrations and frontend surfaces if code work is proposed;
- migration and rollback strategy if persisted data changes;
- validation commands scoped to the actual change.
````

### `docs/refactoring/ui-states-inventory.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/refactoring/ui-states-inventory.md`
- Size bytes / Размер в байтах: `8899`
- Included characters / Включено символов: `7531`
- Truncated / Обрезано: `no`

```markdown
# Инвентаризация UI States и Component Size

> Создано: 2026-06-14 в рамках Phase 1 (Foundation & Safety Net)
> Цель: Задокументировать какие компоненты имеют Loading/Empty/Error/Skeleton states и превышают лимиты по размеру

## 1. Component Size Inventory

Правило: компоненты >500 строк подлежат рефакторингу.

### Превышают лимит (требуют рефакторинга в Phase 5)

| Компонент | Строк | Файл | Статус |
|-----------|-------|------|--------|
| `CommunicationsPage.vue` | 891 | [`frontend/src/domains/communications/views/CommunicationsPage.vue`](../../frontend/src/domains/communications/views/CommunicationsPage.vue) | **GOD COMPONENT** |
| `CommunicationsConversationList.vue` | ~250+ (предположительно) | [`frontend/src/domains/communications/components/CommunicationsConversationList.vue`](../../frontend/src/domains/communications/components/CommunicationsConversationList.vue) | Проверить |
| `CommunicationsContextInspector.vue` | ~200+ | [`frontend/src/domains/communications/components/CommunicationsContextInspector.vue`](../../frontend/src/domains/communications/components/CommunicationsContextInspector.vue) | Проверить |

### CommunicationsPage.vue — разбивка по ответственности (891 lines)

| Раздел | Строки | % | Описание |
|--------|--------|---|----------|
| Imports (script setup) | 1-86 | 9.7% | 85 импортов из 10+ модулей |
| State declarations | 90-94 | 0.6% | refs для UI состояния |
| TanStack Query hooks | 96-150 | 6.2% | 7 useQuery + 3 useMutation |
| Computed properties | 152-163 | 1.3% | 8 computed |
| Watchers | 166-189 | 2.7% | 7 watch для синхронизации Query→Store |
| Message interaction handlers | 191-450 | 29.2% | 20+ handler functions |
| `onMounted` | ~450-470 | 2.2% | Инициализация |
| Template | ~470-891 | 47.3% | HTML template |
| `<style>` | (встроенный) | - | Scoped styles |

**Вывод:** CommunicationsPage.vue — классический God Component. Содержит 7 TanStack Query hooks, 3 mutation hooks, 20+ обработчиков, watchers для синхронизации Query→Store (вместо прямого использования TanStack Query), и ~420 строк шаблона. Требует разбивки на:
- Отдельные composables для Query-логики (уже есть в `queries/`)
- Page layout component
- Отдельные компоненты для ActionBar, MailListSection, ViewerPanel
- Прямое использование TanStack Query в дочерних компонентах вместо watch→store

## 2. UI States Inventory

Легенда:
- ✅ = реализован
- ❌ = отсутствует
- ⚠️ = частично реализован
- N/A = не применимо

### 2.1 Communications Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| `CommunicationsPage.vue` | ❌ (isMailListLoading не используется в template) | ✅ (CommunicationsEmptyPage) | ❌ | ❌ | ❌ |
| `CommunicationsConversationList.vue` | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| `MailList.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `MailViewer.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `ComposeDrawer.vue` | ⚠️ | N/A | ⚠️ | ❌ | ❌ |
| `DraftStrip.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `HealthStrip.vue` | ⚠️ | ❌ | ❌ | ❌ | ❌ |
| `CommunicationsContextInspector.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |
| `CommunicationsContextRail.vue` | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.2 Personas Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Personas page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Identity section | ❌ | ❌ | ❌ | ❌ | ❌ |
| Intelligence section | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.3 Calendar Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Calendar page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Event list | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.4 Tasks Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Tasks page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Task detail | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.5 Knowledge Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Knowledge page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Graph view | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.6 Settings UI Package

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Settings page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Account setup | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.7 Telegram Integration Surface

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| Telegram page | ❌ | ❌ | ❌ | ❌ | ❌ |
| Chat list | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.8 WhatsApp Domain

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| WhatsApp page | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.9 Shared UI Components (базовые)

| Компонент | Loading | Empty | Error | Skeleton | Success/Transition |
|-----------|---------|-------|-------|----------|-------------------|
| `Skeleton.vue` | ✅ | N/A | N/A | ✅ | N/A |
| `Toast.vue` | N/A | N/A | ✅ | N/A | ✅ |
| `Button.vue` | ⚠️ (disabled state) | N/A | N/A | N/A | ✅ |

## 3. Итог по UI States

**Критический вывод:** Ни один domain-level компонент не имеет полного набора Loading/Empty/Error/Skeleton состояний. Только базовые UI компоненты (`Skeleton.vue`, `Toast.vue`) реализуют отдельные состояния.

**Приоритет для Phase 5 (God Component Refactoring):**
1. CommunicationsPage.vue — разбить на компоненты, добавить Skeleton/Error/Empty/Loading
2. Добавить Skeleton.vue во все списки (MailList, ConversationList, TaskList, EventList)
3. Добавить отображение ошибок (через Toast.vue + inline error banners)
4. Empty states для всех списков
5. Анимации переходов (FadeTransition, SlideTransition уже есть в shared)

## 4. Missing Stores Inventory

| Store | Файл | Статус |
|-------|------|--------|
| Personas | `frontend/src/domains/personas/stores/` | ❌ Отсутствует |
| WhatsApp | `frontend/src/integrations/whatsapp/stores/` | ❌ Отсутствует |
| Organizations | `frontend/src/domains/organizations/stores/` | ❌ Отсутствует |
| Documents | `frontend/src/domains/documents/stores/` | ❌ Отсутствует |
| Notes | `frontend/src/domains/notes/stores/` | ❌ Отсутствует |
| Communications | `frontend/src/domains/communications/stores/communications.ts` | ✅ Существует |
| Telegram | `frontend/src/integrations/telegram/stores/telegram.ts` | ✅ Существует |
| Knowledge | `frontend/src/domains/knowledge/stores/knowledge.ts` | ✅ Существует |
| Review | `frontend/src/domains/review/stores/review.ts` | ✅ Существует |
| Tasks | `frontend/src/domains/tasks/stores/tasks.ts` | ✅ Существует |
| Calendar | `frontend/src/domains/calendar/stores/calendar.ts` | ✅ Существует |

## 5. Cross-Domain Import Dependencies

| Source | Target | Файл |
|--------|--------|------|
| `personas/api/personas.ts` | Organizations API | [`frontend/src/domains/personas/api/personas.ts`](../../frontend/src/domains/personas/api/personas.ts) |
| `review/stores/review.ts` | `personas/api/personas` | [`frontend/src/domains/review/stores/review.ts`](../../frontend/src/domains/review/stores/review.ts) |
| `review/stores/review.ts` | `tasks/api/tasks` | same file |
| `review/stores/review.ts` | `knowledge/api/knowledge` | same file |
| `organizations/queries/` | `personas/api/personas` | [`frontend/src/domains/organizations/queries/useOrganizationsQuery.ts`](../../frontend/src/domains/organizations/queries/useOrganizationsQuery.ts) |
```

### `docs/research/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/research/README.md`
- Size bytes / Размер в байтах: `292`
- Included characters / Включено символов: `292`
- Truncated / Обрезано: `no`

```markdown
# Research

Status: documentation package aligned to the current repository structure.

Research documents hold open questions and investigation notes. Decisions from
research must graduate into product docs, architecture docs or ADRs.

## Navigation

- [Open Questions](./open-questions.md)
```

### `docs/research/open-questions.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/research/open-questions.md`
- Size bytes / Размер в байтах: `2640`
- Included characters / Включено символов: `2640`
- Truncated / Обрезано: `no`

```markdown
# Research and Open Questions

Status: active research backlog.

This file records unresolved technical questions. It is not a canonical product
model. When terminology conflicts with foundation, domain, engine or workflow
docs, follow the canonical docs and update this backlog.

## Provider Integrations

- Which email access mode should be first: IMAP/SMTP, Gmail API or provider-specific OAuth? Answered by ADR-0041 and ADR-0055: initial provider shapes are Gmail API/OAuth, iCloud IMAP and generic IMAP, with SMTP/write operations enabled for user-initiated actions and read-only retained only for automated integration tests.
- What Telegram API constraints affect long-term archival and sending?
- What is the reliable WhatsApp integration path for a local-first personal product? Answered for V5 foundation by ADR-0051: use a user-visible `whatsapp_web` companion boundary; keep WhatsApp Business Platform Cloud API as a separate future provider shape.
- How should SMS be handled on desktop without unsafe phone bridge assumptions? (V5)

## Storage

- Should event payloads use JSONB, typed tables or both?
- Which graph representation in PostgreSQL gives the best balance of query power and operational simplicity? Answered for V2 by ADR-0045: relational PostgreSQL graph projection tables (`graph_nodes`, `graph_edges`, `graph_evidence`) are used as rebuildable derived state.
- Which vector index is best for local-first deployment with Rust integration?
- What backup format gives reliable restore across machines?

## AI

- Which Ollama models are strong enough for extraction, classification and summarization?
- What local embedding model gives acceptable multilingual retrieval quality?
- How should extraction quality be evaluated over private data without leaking it?

## UI

- How much graph visualization is useful before it becomes noise?
- Which workflows need split-pane navigation first?
- What is the right density for a personal productivity desktop app?

## Security and Privacy

- Which OS-backed secret store should be used per platform?
- What plugin sandbox runtime isolation model is realistic for Tauri plus Rust after ADR-0052 defines capability manifests and scoped data views?
- What confirmation policies are needed for sending messages and deleting data? Partially implemented by ADR-0052 and the V4 Telegram send policy audit slice: high-risk sends require explicit confirmation unless scoped automation authorizes them, and allowed/rejected dry-run decisions are audited without secrets or private content. Full live-send, delete, export, secret-access and plugin confirmation runtime remains open.
```

### `docs/reviews/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/reviews/README.md`
- Size bytes / Размер в байтах: `330`
- Included characters / Включено символов: `330`
- Truncated / Обрезано: `no`

```markdown
# Reviews

Status: documentation package aligned to the current repository structure.

Review documents are historical traceability records unless a current ADR,
architecture document or product spec explicitly promotes them.

## Navigation

- [Backend Architecture Review 2026-06-06](./backend-architecture-review-2026-06-06.md)
```

### `docs/reviews/backend-architecture-review-2026-06-06.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/reviews/backend-architecture-review-2026-06-06.md`
- Size bytes / Размер в байтах: `18203`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

````markdown
# Backend Architecture Review - 2026-06-06

Status: Historical review.

This review captures backend architecture observations from 2026-06-06. It is
useful for traceability, but it is not the current implementation map. Current
product/domain alignment lives in
`../refactoring/implementation-alignment-plan.md`; current architecture
principles live in `../foundation/` and `../architecture/`.

## Scope

Rust backend (`backend/`) as of current inspected HEAD `a68d908`.

This review covers:

- backend module structure;
- API handler composition;
- coupling between the web layer and domain stores;
- repeated capability checks;
- query parsing;
- error conversion patterns;
- near-term refactoring order.

This review does not cover frontend, Tauri, Docker infrastructure, provider protocol correctness, or database schema design beyond how backend code composes those boundaries.

## Verification Notes

The original draft was directionally useful, but several factual points needed correction.

Verified with:

```sh
git rev-parse --short HEAD
find backend/src -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/src/bin -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/tests -type f -name '*.rs' -print0 | xargs -0 wc -l
find backend/migrations -maxdepth 1 -type f -name '*.sql' -print | sort | tail -n 5
rg -n "verify_local_api_capability\(|fn .*_store\(|parse_.*query|impl From<.*> for ApiError" backend/src/lib.rs
```

Important corrections:

- Backend source currently has **24,441 lines** under `backend/src`, including binary targets.
- Top-level backend library modules have **23,623 lines**.
- Tests have **15,369 lines** under `backend/tests`.
- There are **37 top-level library modules** and **5 binary targets**.
- Migrations currently run through **0024**, not 0023.
- `backend/src/lib.rs` is **3,784 lines**.
- Binary targets are **small** at present:
  - `makosh_email_sync_dev.rs` - 302 lines
  - `makosh_email_fixture_export.rs` - 224 lines
  - `makosh_email_fixture_dev.rs` - 148 lines
  - `makosh_graph_project.rs` - 79 lines
  - `makosh_document_process.rs` - 65 lines
- The original concern that `makosh_email_sync_dev` is a 10k-line production-sized binary is not true in the current tree.

## Current State

The backend has a strong domain-module foundation:

- `event_log` remains the append-only event spine.
- projection helpers keep cursor-based replay semantics isolated.
- stores consistently use `XxxStore { pool: PgPool }`.
- secret handling is separated through secret references, resolver boundaries and encrypted vault storage.
- application settings follow ADR-0054: declared non-secret keys, typed JSONB values and startup repair.

The main architectural issue is not the domain modules. The issue is the **web/application composition layer** in `backend/src/lib.rs`.

`lib.rs` currently contains:

- 37 public module declarations;
- all route registration;
- `AppState`;
- all handler functions;
- request and response DTOs;
- store factory helpers;
- local API capability verification;
- query parsing helpers;
- API error response mapping;
- CORS and tracing setup.

This makes `lib.rs` the main merge-conflict and change-amplification point. Every new backend feature tends to touch it in several unrelated places.

## Architecture Assessment

### What Works

#### 1. Domain stores are cohesive

Most domain modules expose a single public store or service entry point. The pattern is predictable:

```rust
pub struct XxxStore {
    pool: PgPool,
}

impl XxxStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

That is a good fit for this project. Do not replace it with a broad repository abstraction.

#### 2. Event sourcing remains clean

The event log and projection cursor code preserve the current ADR direction:

- events are canonical facts;
- projections are rebuildable;
- projection cursor updates happen after successful handling;
- search and AI-derived state are downstream, not source of truth.

#### 3. Secret boundaries are explicit

The code keeps provider credentials out of ordinary settings and account config. This matches ADR-0053 and ADR-0054.

#### 4. The backend is already testable

The project has substantial backend tests and API tests using `tower::ServiceExt::oneshot`. That means route-module extraction can be protected by existing tests rather than done blind.

## Problems And Solutions

### P1 - `backend/src/lib.rs` is the composition monolith

Severity: High

Current risk:

- unrelated route changes collide in one file;
- request/response DTOs are far from their handlers;
- adding a new API surface requires touching route registration, handler functions, store factories and error conversion in one place;
- route-level tests cannot easily target a small module boundary.

Solution:

Extract route modules gradually, not as one large rewrite.

Recommended structure:

```text
backend/src/
  routes/
    mod.rs
    health.rs
    settings.rs
    graph.rs
    projects.rs
    tasks.rs
    documents.rs
    communications.rs
    ai.rs
    telegram.rs
    whatsapp.rs
    email_accounts.rs
    events.rs
    audit.rs
```

Each route module should own:

- route registration for its surface;
- request DTOs;
- response DTOs;
- handlers;
- route-local query parsers.

Keep shared cross-cutting pieces in `lib.rs` or a small `api` module at first:

- `AppState`;
- `ApiError`;
- local API capability extractors;
- shared response helpers;
- shared query parsing helpers.

Target shape:

```rust
mod routes;

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    let state = AppState {
        config,
        database,
        account_setup: AccountSetupState::default(),
    };

    Router::new()
        .merge(routes::health::routes())
        .merge(routes::settings::routes())
        .merge(routes::graph::routes())
        .merge(routes::events::routes())
        .with_state(state)
        .layer(local_frontend_cors_layer())
}
```

Do not extract every route module at once. Start with `settings`, because it is cohesive and already has focused tests.

### P1 - Capability verification is repeated in handlers

Severity: High

`verify_local_api_capability(&state.config, &headers)?` is repeated across many handlers. Some handlers need the actor, some only need proof that local API capability was verified.

This is not just duplication. It makes it easy to add a new route and forget the guard.

Solution:

Use Axum extractors:

```rust
#[derive(Clone, Debug)]
struct LocalApiActor {
    actor_id: String,
}

struct LocalApiVerified;
```

Implement extraction against `AppState`:

```rust
impl FromRequestParts<AppState> for LocalApiActor {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        verify_local_api_capability(&state.config, &parts.headers)
    }
}

impl FromRequestParts<AppState> for LocalApiVerified {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        verify_local_api_capability(&state.config, &parts.headers)?;
        Ok(Self)
    }
}
```

Handlers then express authorization in the signature:

```rust
async fn put_application_setting(
    actor: LocalApiActor,
    State(state): State<AppState>,
    Path(setting_key): Path<String>,
    Json(request): Json<ApplicationSettingUpdateRequest>,
) -> Result<Json<ApplicationSetting>, ApiError> {
    let updated = settings_store(&state)?
        .update_setting_value(&setting_key, &request.value, &actor.actor_id)
        .await?;

    Ok(Json(updated))
}
```

This should be implemented before route-module extraction. It makes later route moves safer because missing auth becomes visible in handler signatures.

### P2 - Store factory helpers repeat database-pool extraction

Severity: Medium

There are many helpers with the same shape:

```rust
fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApplicationSettingsStore::new(pool.clone()))
}
```

Solution:

Do the smallest useful refactor first:

```rust
impl AppState {
    fn pool(&self) -> Result<PgPool, ApiError> {
        self.database
            .pool()
            .cloned()
            .ok_or(ApiError::DatabaseNotConfigured)
    }
}
```

Then helpers become:

```rust
fn settings_store(state: &AppState) -> Result<ApplicationSettingsStore, ApiError> {
    Ok(ApplicationSettingsStore::new(state.pool()?))
}
```

Do not introduce a generic `FromPool` trait as the first step. It requires touching every store module and adds indirection before the bigger route split. A simple `AppState::pool()` removes the repeated failure branch while keeping the existing explicit store helpers.

After route modules exist, reconsider whether store helpers should remain functions or become `AppState` methods.

### P2 - Query parsing is duplicated

Severity: Medium

There are nine query parsing helpers in `lib.rs`, several of which parse `limit` with similar logic:

- `parse_communication_messages_query`
- `parse_graph_neighborhood_query`
- `parse_graph_nodes_query`
- `parse_graph_search_query`
- `parse_projects_query`
- `parse_project_link_candidates_query`
- `parse_task_candidates_query`
- `parse_document_processing_jobs_query`
- `parse_person_identity_candidates_query`

Solution:

Add a small internal helper that parses typed query parameters and lets each route decide its error code.

Example:

```rust
fn query_param<'a>(raw_query: Option<&'a str>, name: &str) -> Option<String> {
    let raw = raw_query?;
    form_urlencoded::parse(raw.as_bytes())
        .find_map(|(key, value)| (key.as_ref() == name).then(|| value.into_owned()))
}

fn parse_limit_param(
    raw_query: Option<&str>,
    min: usize,
    max: usize,
    invalid: &'static str,
) -> Result<Option<usize>, &'static str> {
    let Some(raw_limit) = query_param(raw_query, "limit") else {
        return Ok(None);
    };

    let limit = raw_limit.parse::<usize>().map_err(|_| invalid)?;
    if limit < min || limit > max {
        return Err(invalid);
    }

    Ok(Some(limit))
}
```

Route-specific parsers still return route-specific `ApiError` variants:

```rust
fn parse_projects_query(raw_query: Option<&str>) -> Result<ProjectsQuery, ApiError> {
    Ok(ProjectsQuery {
        limit: parse_limit_param(raw_query, 1, 100, "limit must be between 1 and 100")
            .map_err(ApiError::InvalidProjectQuery)?
            .unwrap_or(25),
    })
}
```

This avoids hiding HTTP error semantics behind a generic parser.

### P2 - `ApiError` is too broad but should not be abstracted too early

Severity: Medium

The original draft proposed an `IntoApiError` trait and blanket `From<T> for ApiError`. That is a possible end state, but it is not the safest first refactor.

Current `ApiError` has domain-specific behavior that matters:

- some errors are logged as server errors;
- some validation errors become 400;
- some missing records become 404;
- event conflicts become 409;
- local API capability errors include `WWW-Authenticate` behavior.

Solution:

Keep `ApiError` centralized during the first route split. Only extract smaller helpers inside `IntoResponse`, for example:

```rust
fn internal_error(code: &'static str, message: &'static str) -> (StatusCode, &'static str, String, bool) {
    (StatusCode::INTERNAL_SERVER_ERROR, code, message.to_owned(), false)
}
```

After route modules are stable, consider one of these:

1. Keep a single `ApiError` enum but move `From` impls to `api/errors.rs`.
2. Add small domain-to-HTTP traits only for domains that have repeated NotFound/Invalid patterns.
3. Avoid blanket `From<T>` if it makes sensitive error mapping less explicit.

### P3 - Binary target size is not a current problem
````
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/roadmap/README.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/README.md`
- Size bytes / Размер в байтах: `577`
- Included characters / Включено символов: `577`
- Truncated / Обрезано: `no`

```markdown
# Roadmap

Status: documentation package aligned to the current repository structure.

Roadmap documents are versioned planning and closure records. Current product
direction lives in `docs/product/`.

## Navigation

- [Product Roadmap](./product-roadmap.md)
- [V1 Closure Checklist](./v1-closure-checklist.md)
- [V2 Closure Checklist](./v2-closure-checklist.md)
- [V2 Graph Core Checklist](./v2-graph-core-checklist.md)
- [V3 Closure Checklist](./v3-closure-checklist.md)
- [V4 Closure Checklist](./v4-closure-checklist.md)
- [V5 Closure Checklist](./v5-closure-checklist.md)
```

### `docs/roadmap/product-roadmap.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/product-roadmap.md`
- Size bytes / Размер в байтах: `5004`
- Included characters / Включено символов: `5004`
- Truncated / Обрезано: `no`

```markdown
# Product Roadmap

This roadmap uses current foundation terminology. Older implementation
milestones that mention person/contact storage should be read as compatibility
records feeding the Persona model.

## Version 0.1 - Architectural Foundation

Goals:

- establish documentation, ADRs and domain boundaries
- define event, graph, storage, search and agent architecture
- create implementation-ready repository structure

Key functions:

- no runtime product functions
- documentation and design assets only

Architectural changes:

- monorepo skeleton
- initial ADR set
- architecture map

Risks:

- overdesign without later validation
- missing provider-specific constraints

Dependencies:

- review of provider APIs
- storage proof-of-concepts in later phases

## Version 1.0 - Local Memory Core

Goals:

- establish local backend, storage and desktop shell
- ingest first communication source
- build event log, projections and full text search

Key functions:

- local app setup
- PostgreSQL persistence
- event ingestion pipeline
- fixture and read-only provider email import
- local account setup for Gmail, iCloud and raw IMAP
- basic Persona-compatible identity projection
- full text search
- document import for Markdown/PDF

Architectural changes:

- Rust backend foundation
- SvelteKit/Tauri shell
- Tantivy index
- event envelope implementation

Risks:

- production secret backup/recovery and optional OS keychain hardening
- projection replay complexity
- local install and migration UX

Dependencies:

- Rust service architecture
- database migration strategy
- secret storage decision

## Version 2.0 - Knowledge Graph and Documents

Goals:

- make graph-backed memory central
- support richer documents and identity resolution
- connect Communications, Personas, Projects and Documents

Key functions:

- first graph core projection from Persona-compatible identity records, messages and documents
- graph relationships with provenance
- Persona identity merge/split
- document OCR and extraction
- project timeline views
- task candidates from messages and documents

Architectural changes:

- graph schema
- document artifact pipeline
- projection replay tools
- confidence and review workflows

Risks:

- false entity merges
- OCR quality variation
- graph UI complexity

Dependencies:

- document processing engine
- entity extraction evaluation
- graph query patterns

Closure tracking:

- [V2 Closure Checklist](v2-closure-checklist.md)

## Version 3.0 - AI Native Workflows

Goals:

- integrate local agents into daily workflows
- support source-backed analysis and action suggestions
- make AI available inside communication, document, task and graph surfaces

Key functions:

- HESTIA coordinator
- MAKOSH communication agent
- MNEMOSYNE memory agent
- ATHENA analytics agent
- source-backed AI search answers
- task extraction review
- meeting preparation

Architectural changes:

- agent runtime
- tool permission model
- Ollama provider
- embedding provider
- prompt provenance logging

Risks:

- prompt injection
- hallucinated links
- latency on local models

Dependencies:

- local AI model evaluation
- permission model
- graph/search retrieval planner

Closure tracking:

- [V3 Closure Checklist](v3-closure-checklist.md)

## Version 4.0 - Automation, Plugins and Channel Foundation

Goals:

- expand Telegram provider depth and controlled automation
- introduce plugin host
- harden privacy, security and backup

Key functions:

- Telegram integration
- plugin manifest and capability model
- backup/restore
- automation policies
- advanced spam and relevance scoring

Architectural changes:

- plugin runtime
- provider abstraction hardening
- backup verifier
- policy engine

Risks:

- provider API instability
- plugin security
- automation side effects

Dependencies:

- Telegram adapter research
- capability sandbox design
- backup encryption model

Closure tracking:

- [V4 Closure Checklist](v4-closure-checklist.md)

## Version 5.0 - Long-Term Personal Knowledge OS

Goals:

- mature Макошь into a durable personal knowledge operating system
- support deep memory analytics and explainable recall across years
- make replacement of models and indexes routine

Key functions:

- WhatsApp Web companion integration
- optional WhatsApp Business Platform provider research
- optional SMS integration
- cross-year analytics
- decision history
- relationship evolution
- advanced project memory
- structured exports
- index/model replacement tooling
- mature observability and evaluation

Architectural changes:

- long-horizon retention policies
- advanced graph analytics
- model/index migration workflows
- comprehensive evaluation suites

Risks:

- accumulated data quality debt
- performance at multi-year scale
- UX complexity

Dependencies:

- production-scale datasets
- WhatsApp Web companion runtime validation
- search and graph benchmarking
- long-term backup/restore testing

Closure tracking:

- [V5 Closure Checklist](v5-closure-checklist.md)
```

### `docs/roadmap/v1-closure-checklist.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/roadmap/v1-closure-checklist.md`
- Size bytes / Размер в байтах: `2377`
- Included characters / Включено символов: `2377`
- Truncated / Обрезано: `no`

```markdown
# V1 Closure Checklist

## Release Goal

Version 1.0 is complete when a user can run Макошь locally, add
Gmail/iCloud/IMAP accounts, import email fixture data or read-only provider
email batches, inspect canonical messages and Persona-compatible identity
records, search local memory, import Markdown/PDF files into the document
boundary, and open a desktop-first shell connected to backend V1 status.

## In Scope

- Local Rust backend with PostgreSQL migrations and readiness checks.
- Event log, projection cursors and audited local API access.
- Email provider account metadata for `gmail`, `icloud` and `imap`.
- Account-scoped credential references and runtime credential resolution boundary.
- Fixture-based first email import path that preserves raw provider records.
- Read-only Gmail API and iCloud/raw IMAP provider networking that emits raw provider records.
- Encrypted local secret vault and desktop account setup wizards for Gmail, iCloud and raw IMAP.
- Canonical message projection from raw email records.
- Basic Persona-compatible identity projection from message participants.
- Tantivy search boundary covered by message and document record tests.
- Document import boundary for Markdown text and PDF metadata.
- Desktop/laptop SvelteKit/Tauri status shell connected to `GET /api/v1/status`.

## Out of Scope For V1

- Native OS keychain resolver.
- Outbound email sending or mailbox mutation.
- Full MIME parsing beyond raw provider payload preservation.
- Mobile UI design, implementation or validation.
- OCR, entity linking and AI summaries.
- Backup/restore.
- Plugin runtime.

## Acceptance Gate Status

- [x] `make validate` passes from a clean checkout with Docker available.
- [x] Fixture email import preserves raw provider records idempotently.
- [x] Read-only Gmail API and iCloud/raw IMAP provider networking is covered by local network tests and live PostgreSQL batch persistence.
- [x] Account setup stores Gmail OAuth and IMAP credentials in the encrypted vault without plaintext PostgreSQL leakage.
- [x] Canonical messages projection is covered by live PostgreSQL tests.
- [x] Persona-compatible identity projection is covered by live PostgreSQL tests.
- [x] Tantivy search boundary is covered by message/document record tests.
- [x] Document import stores Markdown text and PDF metadata.
- [x] Desktop shell shows backend V1 status.
```
