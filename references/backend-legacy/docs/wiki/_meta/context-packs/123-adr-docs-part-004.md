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

- Chunk ID / ID чанка: `123-adr-docs-part-004`
- Group / Группа: `docs`
- Role / Роль: `adr`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `decisions/adr-index.md`

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

### `docs/adr/ADR-0077-i18n-russian-english.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0077-i18n-russian-english.md`
- Size bytes / Размер в байтах: `2146`
- Included characters / Включено символов: `2130`
- Truncated / Обрезано: `no`

```markdown
# ADR-0077: i18n — Russian and English Interface

Status: Accepted
Date: 2026-06-08
Deciders: Alex (makosh maintainer)

## Context

Макошь is a personal knowledge system. The primary user is Russian-speaking, but technical collaborators and future extensibility benefit from English as a secondary language. Hardcoding strings in one language creates maintenance debt and makes switching impractical.

## Decision

The Макошь desktop interface supports **two languages: Russian (ru) and English (en)** via a lightweight i18n system.

**Mechanism:**
- JSON translation dictionaries under `frontend/src/lib/i18n/` (`en.json`, `ru.json`)
- A Svelte writable store `currentLocale` (defaults to `en`)
- A pure `t(locale, key)` function for translations
- English strings serve as translation keys; `en.json` is an empty `{}`

**Language switch:**
- A toggle in the user menu (⌘ menu) allows switching between Russian and English
- The HTML `lang` attribute is set to `ru` (primary user language)

**Scope:**
- User-visible UI text only: navigation labels, widget titles, zone titles, page headings, form labels, buttons, wizard steps, status messages
- Non-user-visible identifiers (API keys, setting keys, CSS class names, TypeScript identifiers, code comments) remain in English

## Consequences

- All new UI text must go through the `t()` / `_()` translation function
- `ru.json` must be kept in sync when English strings are added or changed
- Translation coverage is progressive: unwrapped strings display as English (the fallback key)
- The i18n system is intentionally minimal — no external library dependency

## Alternatives Considered

**Russian-only UI (ADR-0077 draft).** Rejected — English fallback supports collaboration and future extensibility without blocking.

**Full i18n library (svelte-i18n, @sveltekit-i18n, typesafe-i18n).** Rejected — adds dependency weight without proportional benefit for a two-language personal system.

**No i18n, English only.** Rejected — primary user is Russian-speaking.

## References

- ADR-0026 — desktop-first responsive UI
- ADR-0031 — temporary desktop-only UI scope
```

### `docs/adr/ADR-0078-frontend-component-decomposition.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0078-frontend-component-decomposition.md`
- Size bytes / Размер в байтах: `6639`
- Included characters / Включено символов: `6597`
- Truncated / Обрезано: `no`

````markdown
# ADR-0078: Frontend Component Decomposition — SPA Pages and Widgets

Status: Superseded by ADR-0093
Date: 2026-06-09
Deciders: Alex (makosh maintainer)

## Superseded

This ADR is superseded by [ADR-0093](ADR-0093-frontend-platform-migration-to-vue-3.md).
The frontend platform has migrated from SvelteKit to Vue 3. The Domain-Driven
Design folder structure defined in ADR-0093 replaces the SvelteKit-specific
component decomposition described here.

## Context

`frontend/src/routes/+page.svelte` contains 8954 lines: all 14+ application views,
settings, shell (sidebar, topbar, notifications), account setup wizards, layout
editor, compose drawer, draft strip, health strip, and all business logic. There
are zero reusable Svelte components. This creates unmanageable cognitive load,
makes independent work on different views impossible, and blocks testing.

The application is a desktop SPA (Tauri shell, static adapter, `prerender=true`,
`ssr=false`) with client-side state-based routing via `currentView` variable. All
navigation happens without page reloads.

## Decision

### Routing: SPA state-based (unchanged)

Keep the single `+page.svelte` route and `currentView` state variable. Do NOT
convert to SvelteKit filesystem routing (`routes/home/+page.svelte` etc.).
Rationale:

- Matches desktop app UX (no URL changes, no page transitions)
- Preserves `prerender=true, ssr=false` build config unchanged
- Shell (sidebar, topbar) stays mounted across view switches
- Lower risk — no build pipeline or adapter config changes

### Architecture: Pages as Svelte components

Each application view becomes a Svelte component in `$lib/pages/<name>/`:

```
src/lib/pages/
  <name>/
    <Name>Page.svelte      # Main page: layout, data fetching, state
    widgets/               # Widget components for this page
      <Widget1>.svelte
      <Widget2>.svelte
```

### Shell decomposition

Extract the shell from `+page.svelte` into reusable components:

```
src/lib/components/
  shell/
    Sidebar.svelte               # Navigation sidebar with groups
    Topbar.svelte                # Top bar with notifications, user menu
    NotificationsDrawer.svelte    # Notification panel
  vault/
    VaultOnboarding.svelte       # Initial vault setup wizard
  shared/
    WidgetFrame.svelte           # Zone + widget container
    WidgetEditChrome.svelte      # Layout edit controls overlay
    LayoutEditControls.svelte    # Add/cancel/save layout editing bar
    WidgetSettingsDrawer.svelte  # Widget configuration panel
    AddWidgetDrawer.svelte       # Add widget catalog
    ComposeDrawer.svelte         # Email compose panel
    DraftStrip.svelte            # Recent drafts bar
    HealthStrip.svelte           # Mailbox health indicator
```

### State management

| Layer | Mechanism | Scope |
|-------|-----------|-------|
| Locale | `$lib/i18n` writable store | Global (existing, unchanged) |
| Navigation | New `$lib/stores/navigation.ts` writable | Global — `currentView`, `activeCommunicationSection` |
| Theme | New `$lib/stores/theme.ts` writable | Global — shell background, accent, opacity |
| Notifications | New `$lib/stores/notifications.ts` writable | Global — notification items, drawer state |
| Sidebar | New `$lib/stores/sidebar.ts` writable | Global — persisted sidebar settings, draft, resolved entries |
| Layout editor | New `$lib/stores/layoutEditor.ts` writable | Global — constructor mode, widget draft, widget drawers |
| Settings | New `$lib/stores/settings.ts` writable | Global — application settings, provider accounts, settings tabs |
| Page data | `$state` inside page component | Page-local — loaded data, form state |
| Account setup | `$lib/stores/accountWizard.ts` writable | App-level — modal target/open state shared across views |
| Compose/draft/health | `$state` inside each page component | Page-local — repeated pattern per page |

### Migration strategy

**Incremental extraction**: Extract one page at a time, verify after each step.
No big-bang rewrite.

1. **Foundation** (stores, shell, shared components) — zero visual change
2. **Pages** one by one — each page removed from god file, replaced by component import
3. **Cleanup** — remove dead code, deduplicate repeated blocks

Page extraction order (simplest first):

| Order | Page | Rationale |
|-------|------|-----------|
| 1 | settings | Well-structured tabbed forms, ~260 lines template |
| 2 | timeline | Simple list, ~90 lines |
| 3 | organizations | List + detail, ~70 lines |
| 4 | tasks | Table + rail, ~80 lines |
| 5 | home | Dashboard, ~120 lines |
| 6 | documents | Cards + list + rail, ~100 lines |
| 7 | notes | List + rail, ~80 lines |
| 8 | calendar | Complex but self-contained, ~180 lines |
| 9 | persons | List + detail + rail, ~180 lines |
| 10 | projects | Hero + tabs + rail, ~250 lines |
| 11 | knowledge | Graph canvas + rail, ~310 lines |
| 12 | agents | Cards + detail + forms, ~120 lines |
| 13 | communications | 3-pane + sub-sections, ~220 lines |
| 14 | telegram | Chat + complex forms, ~280 lines |
| 15 | whatsapp | Sessions + messages, ~140 lines |
| 16 | mail | 3-pane, ~210 lines |

### CSS

CSS is split along the same ownership boundaries as components:

- shell CSS lives beside shell components;
- shared panel/widget CSS lives beside shared components;
- page CSS lives under `$lib/pages/`;
- global reset, tokens and cross-cutting shell classes remain in `$lib/styles/`.

Page components keep existing semantic CSS class names where practical.

## Consequences

- Each page becomes independently readable and editable
- Widgets can be developed and tested in isolation
- Shell components are shared, eliminating repeated blocks (compose, draft, health)
- `+page.svelte` shrinks from the original god file toward a thin SPA router
- `+layout.svelte` owns shell assembly and cross-cutting shell controls
- All existing tests (`test:layout`) continue to pass without changes
- No build config changes, no new dependencies
- CSS follows component/page ownership and no longer remains monolithic

## Alternatives Considered

**SvelteKit filesystem routing.** Rejected — requires adapter config changes,
breaks the desktop SPA UX model, increases risk without proportional benefit
for a Tauri desktop app.

**Big-bang rewrite.** Rejected — incremental extraction allows validation after
each step and keeps the app functional throughout.

## References

- ADR-0003 — SvelteKit Frontend
- ADR-0004 — Tauri Desktop Shell
- ADR-0026 — Desktop First Responsive UI
- ADR-0031 — Temporary Desktop Only UI Scope
- ADR-0077 — i18n Russian and English Interface
````

### `docs/adr/ADR-0079-script-logic-decomposition.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0079-script-logic-decomposition.md`
- Size bytes / Размер в байтах: `7209`
- Included characters / Включено символов: `7147`
- Truncated / Обрезано: `no`

````markdown
# ADR-0079: Script Logic Decomposition — Services, Smart Pages and Config

Status: Superseded by ADR-0093
Date: 2026-06-09
Deciders: Alex (makosh maintainer)

## Superseded

This ADR is superseded by [ADR-0093](ADR-0093-frontend-platform-migration-to-vue-3.md).
The frontend platform has migrated from SvelteKit to Vue 3. While the general
pattern of service extraction, config modules and page-owned data loading
remains valid, the Svelte-specific implementation details ($state, $effect,
writable stores) are replaced by Vue 3 Composition API, Pinia and TanStack
Query as defined in ADR-0093.

## Context

After ADR-0078 (component decomposition), `+page.svelte` shrank from 8954 to
~5380 lines. The template is now fully decomposed into 86 Svelte components.
However, the script block still contains ~4900 lines: 78 `$state` variables,
66 `$derived` values, 70 async data-fetching functions, and 160 helper functions.

All page components are "dumb" — they receive data and callbacks as props from
`+page.svelte`. This keeps `+page.svelte` as the single owner of all application
state, making it the bottleneck for any change.

## Decision

### 1. Extract `lib/config.ts`

Move `apiBaseUrl` and `apiSecret` from `+page.svelte` to a shared config module:

```ts
// lib/config.ts
export const apiBaseUrl = import.meta.env.VITE_MAKOSH_API_BASE_URL ?? 'http://127.0.0.1:8080';
export const apiSecret = import.meta.env.VITE_MAKOSH_LOCAL_API_SECRET ?? 'change-me-local-api-secret';
```

All service modules and components import from here instead of receiving these
values as props or defining them locally.

### 2. Extract service modules (`lib/services/`)

Group related data-fetching functions into domain service modules. Each service
imports `apiBaseUrl`/`apiSecret` from config and API functions from `$lib/api`.

```
src/lib/services/
  vault.ts          — loadV1Status, createVault, unlockVault, exportRecovery
  graph.ts          — loadGraphSummary, runGraphSearch, selectGraphNode, loadNeighborhood
  communications.ts — loadCommunications, loadDrafts, loadMailboxHealth, workflow
  projects.ts       — loadProjects, loadProjectDetail
  persons.ts        — loadPersons, loadIdentityCandidates, setIdentityReview
  tasks.ts          — loadTaskReviewState, setTaskCandidateReview
  calendar.ts       — loadCalendar, searchCalendar, handleCreateEvent, prepareEvent
  documents.ts      — loadDocumentProcessingJobs, retryFailedJob
  ai.ts             — loadAiWorkspace, submitAiAnswer, refreshTasksFromAi
  telegram.ts       — loadTelegramWorkspace, telegram auth, fixture ingestion
  whatsapp.ts       — loadWhatsappWebWorkspace, fixture ingestion
  settings.ts       — loadSettingsWorkspace, saveSetting, saveTheme, saveSidebar, saveLocale
  accounts.ts       — account setup wizards (mail, calendar, telegram, whatsapp)
```

Each service exports plain async functions. The functions manage their own
loading/error state if needed, or return results for the caller to manage.

### 3. "Smart" page components

Each page component becomes responsible for loading its own data via `$effect`:

```svelte
<!-- HomePage.svelte -->
<script>
  import { loadCommunications } from '$lib/services/communications';
  let messages = $state([]);
  let loading = $state(false);

  $effect(() => {
    loading = true;
    loadCommunications().then(data => {
      messages = data;
      loading = false;
    });
  });
</script>
```

Page components no longer receive data as props from `+page.svelte` (except
cross-cutting concerns like layout editing state). They own their data lifecycle.

### 4. Move helper functions to appropriate lib files

Helper functions that are pure and domain-specific move to their respective
service or lib files:

- `graph*` functions → `lib/services/graph.ts` (or `lib/layout/graph-helpers.ts` if pure)
- `communication*` helpers → `lib/services/communications.ts`
- `project*` helpers → `lib/services/projects.ts`
- `account*` helpers → `lib/services/accounts.ts`
- `format*`, `sender*`, `messageTime` → `lib/formatting.ts`
- `setting*` helpers → `lib/services/settings.ts`
- `sidebar*` helpers (already partially in `lib/layout/sidebar-navigation.ts`)
- `widget*` helpers → `lib/layout/widget-helpers.ts`
- `vault*` helpers → `lib/services/vault.ts`

### 5. `+page.svelte` as thin orchestrator

After extraction, `+page.svelte` retains only:
- View routing (which page component to render)
- Shared drawer hosts that must stay mounted at the SPA route level

Cross-cutting shell state is owned outside `+page.svelte`:

- `+layout.svelte` assembles shell UI and drawers
- `$lib/stores/layoutEditor.ts` owns constructor mode and widget editor state
- `$lib/stores/sidebar.ts` owns persisted sidebar state and resolved shell entries
- `$lib/stores/notifications.ts` owns drawer state, notification count and target selection
- `$lib/stores/vault.ts` with `$lib/services/vault.ts` owns onboarding state and vault actions
- `$lib/stores/settings.ts` owns settings workspace data, provider account lists and settings tab state
- `$lib/stores/accountWizard.ts` owns account setup modal target/open state

### Migration strategy

**Incremental by domain**: Extract one service at a time, update the
corresponding page component, verify.

1. `lib/config.ts` — no behavior change
2. `lib/services/vault.ts` + update VaultOnboarding
3. `lib/services/graph.ts` + update KnowledgePage
4. `lib/services/communications.ts` + update CommunicationsPage
5. Continue through all domains
6. Move cross-cutting shell state into shared stores
7. Move settings workspace/account setup state out of `+page.svelte`
8. Clean up dead code from `+page.svelte`

## Consequences

- Each page component is independently understandable and testable
- Data fetching is co-located with the UI that displays it
- `+page.svelte` becomes a true thin orchestrator
- Services can be reused across pages (e.g., loadCommunications used by both
  CommunicationsPage and HomePage)
- Debugging is easier — each page's data flow is self-contained
- Type safety is maintained by importing API types from `$lib/api`

## Risks

- Some data is shared across pages (e.g., `communicationMessages` used by
  HomePage, CommunicationsPage, TimelinePage). Solution: each page loads
  independently, or we create a simple cache layer in the service.
- Multiple concurrent API calls on page navigation. Solution: SvelteKit's
  client-side navigation keeps components mounted; `$effect` cleanup cancels
  in-flight requests.

## Alternatives Considered

**Keep current "dumb page" pattern.** Rejected — `+page.svelte` remains the
~5000-line bottleneck. Every new feature requires touching the god file.

**Full state management library (e.g., Svelte stores for everything).**
Rejected — adds complexity without proportional benefit. Per-page `$state` is
simpler and sufficient for a desktop SPA.

**Redux/Zustand-style centralized store.** Rejected — overkill for a personal
desktop application. Svelte 5 runes provide sufficient reactivity.

## References

- ADR-0078 — Frontend Component Decomposition
- ADR-0003 — SvelteKit Frontend
- ADR-0026 — Desktop First Responsive UI
````

### `docs/adr/ADR-0080-mail-background-sync-progress-local-trash.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0080-mail-background-sync-progress-local-trash.md`
- Size bytes / Размер в байтах: `5250`
- Included characters / Включено символов: `5220`
- Truncated / Обрезано: `no`

```markdown
# ADR-0080 Mail Background Sync, Progress and Local Trash

Status: Accepted
Date: 2026-06-10
Deciders: Alex (makosh maintainer)

## Context

Макошь mail already has provider account setup, raw/blob preservation,
message projection and provider SMTP send for iCloud/generic IMAP. The mail
workbench now needs continuous account-scoped ingestion instead of only manual or
fixture-driven imports.

The owner also clarified deletion semantics: deleting mail in the Макошь UI must
not delete, move, trash or expunge the original provider message. Макошь
should hide the item from active workbench views while preserving local raw/blob
content and metadata for replay, AI analysis and analytics.

Mail sync also feeds the knowledge system. Sender and recipient identities should
be projected to persons, organizations and relationship events as mail arrives.

## Decision

Add account-scoped mail sync settings and durable run history.

- Every `gmail`, `icloud` and `imap` provider account has effective defaults:
  `sync_enabled = true`, `batch_size = 5`, `poll_interval_seconds = 300`.
- Background scheduling is per account and prevents overlapping active runs for
  the same account.
- Manual "check now" uses the same account-scoped service and records a durable
  run.
- Run status records only sanitized metadata: account id, trigger, phase,
  progress, counts, checkpoint presence and sanitized error code/message.
- Status and audit records must never contain mail bodies, raw MIME, attachment
  bytes, tokens, passwords or plaintext secret references.

Provider read behavior:

- IMAP/iCloud full backfill starts without `last_seen_uid`, fetches batches in
  ascending UID order, persists checkpoint data, and loops until an empty batch.
- IMAP/iCloud incremental sync starts after stored `last_seen_uid`.
- If IMAP UID validity changes, UID progress is reset and the mailbox is read
  again from the start.
- Gmail full backfill pages through `users.messages.list` and raw message reads.
- Gmail incremental sync uses Gmail history when a `history_id` checkpoint is
  available. If history is expired, the run records a recoverable full-resync
  condition and restarts full listing.

Local deletion behavior:

- `communication_messages.local_state` is separate from workflow state.
- Active UI lists, counts, threads, search indexing and resource summaries use
  `local_state = active` by default.
- UI delete sets `local_state = trash`, `local_state_changed_at` and
  `local_state_reason`.
- Restore sets `local_state = active`.
- Local trash is not auto-emptied.
- Provider delete operations are not used for UI delete. The legacy
  `/imap-delete` route maps to local trash and does not send IMAP `STORE`,
  `EXPUNGE`, provider move or provider trash commands.
- Reprojection preserves an existing `trash` state.

Knowledge projection behavior:

- Mail projection creates or updates `persons` and active email identities from
  normalized addr-spec values.
- Mail projection creates durable `communication_message_participants` links.
- Mail projection creates idempotent relationship events for sender/recipient
  interactions.
- Mail projection creates non-public-domain organizations, organization domains
  and organization contact links with mail sync/message provenance.
- Graph projection is refreshed after successful mail batches. Task/note
  extraction remains candidate or user-controlled and does not auto-activate
  tasks.

Frontend behavior:

- The mail workbench exposes a desktop-only account selector with per-account
  sync state and thin progress indicators.
- The selected account exposes sync settings and manual check-now controls.
- All Accounts shows aggregate sync progress.
- The workbench has a Trash filter/folder. Normal inbox/thread/search/resource
  views exclude trash by default.
- Message detail exposes Delete for active messages and Restore for trash
  messages.
- All new visible strings use i18n keys; Russian translations are maintained.
- Validation remains desktop-only while ADR-0031 is active.

## Consequences

- Mail sync is durable, inspectable and configurable per account.
- Provider credentials remain account scoped and resolved only at runtime.
- UI delete is safe for provider mailboxes and reversible locally.
- Local trash keeps data available for replay, AI and analytics.
- Sync can be extended later with provider-specific adapters, outbox queues,
  attachment scanning or mobile behavior without changing the local-state
  contract.

## References

- ADR-0031 — Temporary Desktop Only UI Scope
- ADR-0041 — Email Provider Ingestion Foundation
- ADR-0046 — Persistent Dev Mail Cache and Blob Storage
- ADR-0052 — Capability-Based Provider Writes
- ADR-0055 — Full Email Provider Networking
- ADR-0060 — Person Timeline and Graph Integration
- ADR-0061 — Organization as First-Class Entity
- ADR-0062 — Organization Identity and Resolution
- ADR-0066 — Organization Graph Integration
- ADR-0070 — Tasks First-Class Domain
- ADR-0071 — Task Context Evidence Provenance
- ADR-0074 — Person Multi-Channel Identity Model
- ADR-0076 — Host Vault on macOS
- ADR-0077 — i18n Russian and English Interface
- ADR-0078 — Frontend Component Decomposition
```

### `docs/adr/ADR-0081-opt-in-omniroute-ai-runtime.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0081-opt-in-omniroute-ai-runtime.md`
- Size bytes / Размер в байтах: `2096`
- Included characters / Включено символов: `2096`
- Truncated / Обрезано: `no`

```markdown
# ADR-0081 Opt-In OmniRoute AI Runtime

Status: Proposed

## Context

ADR-0009 selected Ollama as the initial local AI runtime boundary and requires remote models to be opt-in and policy controlled if added later.
ADR-0049 implemented V3 AI with Ollama as the only provider, but the local infrastructure now exposes a dedicated OpenAI-compatible OmniRoute gateway for this workstation.

Макошь still handles private communications and documents. Remote or routed model calls must not become implicit defaults.

## Decision

Add an opt-in AI runtime provider named `omniroute` alongside the existing default `ollama` provider.

Rules:

- `ollama` remains the default provider.
- `omniroute` is enabled only by explicit runtime setting or environment override.
- OmniRoute uses an OpenAI-compatible API boundary.
- Non-secret provider settings may live in `application_settings`.
- OmniRoute API keys remain outside `application_settings`; the initial implementation reads `MAKOSH_OMNIROUTE_API_KEY` from process environment.
- The AI run event payload records provider name and model IDs, not API keys or private prompt/document bodies.
- Existing semantic embedding dimension validation remains enforced; changing embedding models still requires compatibility with `halfvec(2560)` unless a future ADR changes the derived index shape.

## Consequences

Positive:

- Макошь can use the owner-managed OmniRoute gateway without hardcoding a cloud provider.
- Local Ollama remains the safe default for private data.
- Provider replacement is isolated behind a runtime client boundary.

Negative:

- Opting into OmniRoute can send private prompts and retrieved context to upstream providers selected by OmniRoute routing.
- Live smoke validation requires an API key that must not be printed or committed.
- Embedding model changes remain constrained by the existing semantic index dimension.

## Non-Goals

- No new database migration.
- No storage of OmniRoute API keys in PostgreSQL settings.
- No graph/schema expansion.
- No audio, image generation, or transcription support in this slice.
```

### `docs/adr/ADR-0082-ai-settings-control-center.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0082-ai-settings-control-center.md`
- Size bytes / Размер в байтах: `3018`
- Included characters / Включено символов: `3018`
- Truncated / Обрезано: `no`

```markdown
# ADR-0082 AI Settings Control Center

Status: Proposed

## Context

ADR-0049 introduced the V3 local AI runtime with Ollama defaults. ADR-0081 added explicit OmniRoute support, but the current settings surface still exposes AI as generic `application_settings` rows. That does not scale to built-in runtime management, CLI-backed local agents, remote API providers, per-capability model routing, or editable prompt templates.

Макошь handles private communications and documents. AI provider configuration must preserve the local-first posture, keep secrets in the host vault from ADR-0076, and make remote-context consent explicit.

## Decision

Add an AI Control Center domain surfaced from Settings as a first-class `AI` section.

Rules:

- `application_settings` remains the allowlisted non-secret fallback surface, but AI provider accounts, model inventory, routing and prompt studio state live in AI domain tables.
- AI provider accounts support `built_in`, `cli` and `api` provider kinds.
- API provider secrets are stored only through host-vault secret references. Environment-backed OmniRoute remains a legacy/bootstrap fallback.
- Remote/API providers require explicit provider-level consent before they can be used for private-context workflows.
- CLI agents are provider bridges only. They may execute only allowlisted fixed command/argument presets and must not become autonomous workflow actors in this slice.
- Built-in Ollama runtime management is desktop/macOS-first. Макошь may install/start/update the runtime automatically, but model downloads require explicit user confirmation.
- Model routing uses stable capability slots instead of one global chat model. Embedding routes must keep the current 2560-dimension constraint until a future ADR changes the semantic index shape.
- Prompt templates are versioned. System prompts are seeded/read-only, while user prompts and active versions are stored as domain records.
- Prompt evaluation runs may persist model output and metadata, but audit/event payloads must not store raw private source text, API keys or provider secret values.

## Consequences

Positive:

- AI configuration becomes understandable as a product area instead of a generic settings list.
- Provider setup, model catalog, routing and prompt templates can evolve independently.
- Secrets remain behind the host-vault resolver boundary.
- AI runs can record provider/model/prompt provenance without leaking credentials.

Negative:

- A new migration and API surface are required.
- Runtime management introduces OS/process concerns that must stay behind allowlisted adapters.
- Remote provider consent becomes part of user workflow before some models can be selected.

Risk handling:

- Seed safe local Ollama defaults.
- Treat remote/API provider state as unavailable until consent and a vault-backed credential are present.
- Keep CLI command presets static and validated.
- Add regression tests for secret-like payload rejection and no private text in event/audit metadata.
```

### `docs/adr/ADR-0083-telegram-live-user-client-runtime.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0083-telegram-live-user-client-runtime.md`
- Size bytes / Размер в байтах: `3758`
- Included characters / Включено символов: `3758`
- Truncated / Обрезано: `no`

```markdown
# ADR-0083 Telegram Live User Client Runtime

Status: Proposed

## Context

ADR-0050 made Telegram a first-class communication channel, but the implemented
V4 surface is still a foundation: fixture accounts, QR authorization setup,
policy dry-runs, call metadata and transcript storage. A usable Telegram client
needs a live `telegram_user` runtime that can keep TDLib state warm, sync chats,
sync selected history, send user-confirmed messages and fetch media on demand.

This must not turn Telegram into the source of truth. Макошь remains
local-first and event-backed: Telegram provider data is preserved as source
evidence, while canonical messages, graph links and task candidates remain local
projections.

## Decision

Implement the first live Telegram user-client slice around an account-scoped
backend TDLib runtime boundary.

- `telegram_user` accounts use a Rust backend runtime manager with one actor per
  account. The actor owns TDLib receive-loop state and serializes commands for
  chat sync, selected-chat history sync, manual sends and media downloads.
- CI and local smoke tests may use a fixture runtime through the same API shape.
  Fixture runtime support must remain available even when native TDLib is not
  installed.
- `telegram_bot` accounts remain setup-compatible, but Bot API live runtime is a
  later slice.
- All chat metadata may be synced for configured user accounts. Message history
  and media are synced deeply only for selected or pinned chats.
- Manual sends are `provider_write` actions. The UI click is explicit user
  confirmation, but backend policy/audit remains authoritative.
- Automated live sends remain blocked. Existing policy dry-runs continue to use
  the ADR-0052 automation policy model.
- Raw provider records are append-only and idempotent. Canonical
  `communication_messages` remain projections from source records.
- TDLib local state and downloaded media bytes stay under ignored local data
  paths. PostgreSQL stores metadata, provenance, checkpoints, attachment records,
  hashes and local blob references only.
- On-demand Telegram media uses the existing communication attachment and safety
  scanner boundary. A provider-neutral facade may wrap the current mail-named
  blob store; table renaming is not part of this slice.
- Live calls, desktop audio capture and real speech-to-text remain blocked
  capabilities until separate runtime, permission and validation work exists.

## Consequences

Positive:

- Telegram can become a usable desktop workbench without making provider state
  canonical.
- CI can validate command, projection, audit and media behavior without live
  Telegram credentials.
- Manual sends get the same backend authority and audit posture as future
  automation and destructive actions.

Negative:

- TDLib update handling becomes long-lived runtime infrastructure instead of a
  short QR-login helper.
- Runtime status, degraded states and account lifecycle must be visible in UI.
- Media sync requires size, storage and scanner discipline from the first slice.

Risk handling:

- Do not report live TDLib send/sync capability as `available` unless the native
  runtime, account authorization, command path and opt-in smoke validation exist.
- Do not store Telegram API hashes, bot tokens, session encryption keys, message
  bodies or media bytes in audit records.
- Do not auto-download all media by default.
- Do not allow AI or automation to choose destinations, accounts, templates or
  live-send authority.

## Non-Goals

- Telegram Bot API live runtime.
- Automated live Telegram sends.
- Message edit, delete, forward, pin or reaction parity.
- Video calls, group calls or screen sharing.
- Hidden recording or cloud transcription by default.
- Mobile UI.
```

### `docs/adr/ADR-0084-persona-intelligence-system.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0084-persona-intelligence-system.md`
- Size bytes / Размер в байтах: `4892`
- Included characters / Включено символов: `4892`
- Truncated / Обрезано: `no`

````markdown
# ADR-0084 Persona Intelligence System

Status: Proposed

Supersedes:

- ADR-0019 Contact Identity Resolution
- ADR-0059 Person Communication DNA and Personas

Clarifies:

- ADR-0057 Person Memory and Provenance
- ADR-0058 Person Enrichment Engine
- ADR-0060 Person Timeline and Graph Integration
- ADR-0074 Person Multi-Channel Identity Model

## Context

Макошь is a local-first Personal Memory System. The persons domain was
previously documented as a partially renamed contact system: contacts became
persons, but the model still used CRM-shaped concepts such as contact merge,
roles, nested personas, favorites, watchlists, health status, fingerprints,
analytics and investigator flows.

The domain direction has changed. Макошь does not treat people as contacts.
Макошь treats subjects as Personas.

A Persona is a durable digital representation of a subject that can accumulate
identity, relationships, communication context, memory, timeline, knowledge and
a generated dossier.

## Decision

Use Persona Intelligence as the target architecture for the persons domain.

The root domain entity is:

```yaml
Persona:
  id:
  is_self:
  persona_type:

  identity:
  communication:
  memory:
  timeline:
  relationships:
  dossier:
```

Exactly one Persona represents the owner:

```yaml
Persona:
  is_self: true
```

There is no separate `UserProfile` or Self domain. Local agents act through the
Owner Persona when operating for the system owner.

Supported Persona types:

```yaml
PersonaType:
  human
  ai_agent
  organization_proxy
  system
```

Relationships are first-class records:

```yaml
Relationship:
  source_persona:
  target_persona:
  type:
  trust_score:
  strength_score:
```

Trust and relationship strength must not be stored only as fields on a Persona.
Roles, organization links, relationship health and attention state are modeled
as Relationships, Timeline events, memory records or read models.

Persona memory contains facts, knowledge, preferences, memory cards and
conflicts with provenance, confidence and verification metadata. AI output may
produce observations and candidates, but it is not source of truth without
reviewed, cited storage.

Each Persona has a generated Dossier read model:

```yaml
Dossier:
  summary:
  interests:
  projects:
  organizations:
  skills:
  communication_patterns:
  ai_observations:
```

`fingerprint`, `communication profile`, `trust`, `analytics` and `investigator`
are consolidated under Persona Intelligence.

Identity Resolution operates on digital traces of a Persona:

- email;
- phone;
- Telegram;
- WhatsApp;
- GitHub;
- LinkedIn;
- documents;
- messages;
- provider-specific handles.

Ambiguous identity resolution remains reviewable. This preserves the safety
property from ADR-0019 while replacing its Contact framing.

ADR-0074 remains the implementation compatibility contract for existing
`person_id` values and `/persons` routes until a separate schema/API migration
ADR is accepted. This ADR changes the domain model and terminology; it does not
silently require a database migration.

## Consequences

Positive:

- The domain aligns with Макошь as a Personal Memory System.
- People, agents, organization proxies and system actors can exist in one graph.
- Relationships become queryable, provenance-backed records.
- The Owner Persona gives agents a clear subject boundary.
- Dossiers become derived read models with citations instead of manually edited
  contact summaries.
- Identity resolution can unify communication and document traces without
  pretending they are address-book fields.

Negative:

- Current `persons` schema and `/persons` API names become compatibility
  details.
- `person_personas` conflicts with the new Persona meaning. Compatibility
  writes now materialize interaction-context values into Persona Preferences,
  but the nested Persona table and route names remain deprecated compatibility
  surfaces until a schema/API migration ADR retires them.
- Existing health, watchlist, role and trust fields must be reclassified before
  deeper implementation work.
- UI and backend code will need a future migration plan to avoid breaking
  current projections.

## Non-Goals

- Immediate schema migration from `persons` to `personas`.
- Immediate route migration from `/persons` to `/personas`.
- Removing current compatibility tables or endpoints.
- Fine-tuning models on private Persona data.
- Turning public enrichment into active OSINT or scraping beyond approved
  provider boundaries.

## Required Follow-Up

- Design a schema/API migration ADR if implementation moves from compatibility
  `persons` storage to Persona-native storage.
- Add first-class Relationship records.
- Add Owner Persona uniqueness semantics.
- Add target PersonaType validation.
- Reframe existing intelligence, analytics and investigator code as Persona
  Intelligence services and read models.
````

### `docs/adr/ADR-0085-communication-spine-and-contradiction-engine.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0085-communication-spine-and-contradiction-engine.md`
- Size bytes / Размер в байтах: `3503`
- Included characters / Включено символов: `3503`
- Truncated / Обрезано: `no`

````markdown
# ADR-0085 Communication Spine and Consistency / Contradiction Engine

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0041 Email Provider Ingestion Foundation
- ADR-0055 Full Email Provider Networking
- ADR-0084 Persona Intelligence System

## Context

Макошь is a local-first Personal Memory System. The product model treats
Communications as the primary ingestion spine: messages, meetings, calls and
provider events enter Макошь as source evidence and become knowledge, memory,
relationships, obligations, tasks, decisions and project context.

The repository still contains email-heavy implementation boundaries because
email was implemented first. Telegram, WhatsApp, calls and meetings already
exist as adjacent surfaces. Documentation needs one canonical model that
explains how all interaction evidence enters the system.

The user also approved a Polygraph concept: when a new message, document or
event contradicts remembered knowledge, Макошь should detect the conflict and
surface it for review.

## Decision

Treat Communications as the primary ingestion spine for the Personal Memory
System.

The canonical flow is:

```text
Communication
  -> Source Evidence
  -> Extracted Knowledge
  -> Memory
  -> Relationships
  -> Context
  -> Obligations / Tasks / Decisions / Projects
```

Email, Telegram, WhatsApp, calls, meetings and future providers are channels
feeding the Communications model. Provider-specific behavior remains at adapter
and source-record boundaries.

Add the Consistency / Contradiction Engine as a shared engine. Its user-facing
alias is Polygraph.

The engine compares new evidence with accepted memory and knowledge. It creates
source-backed contradiction observations and review items. It must not:

- decide that a Persona is lying;
- silently overwrite accepted memory;
- mutate domain state without review or explicit policy;
- hide source references.

Required contradiction output includes:

```yaml
ContradictionObservation:
  old_source:
  new_source:
  affected_entities:
  conflict_type:
  old_claim:
  new_claim:
  confidence:
  severity:
  review_state:
```

## Consequences

Positive:

- Communications become the common entry point for memory, knowledge and action.
- Email-specific functionality can be documented as a channel, not as the whole
  product.
- Contradictions become explicit reviewable observations instead of silent
  memory drift.
- The engine boundary prevents every domain from inventing its own local
  polygraph logic.

Negative:

- Existing `mail` module naming remains a compatibility detail until a future
  implementation migration is planned.
- No dedicated backend module, table or review workflow exists yet for the
  Consistency / Contradiction Engine.
- Existing domain-local intelligence and health modules must be audited before
  shared engine extraction.

## Non-Goals

- Immediate code rename from Mail to Communications.
- Immediate schema migration.
- Immediate public API design.
- Automatic contradiction resolution.
- Using contradiction detection as a punitive trust judgment.

## Required Follow-Up

- Keep detailed engine behavior in `docs/engines/consistency/README.md`.
- Keep communication ingestion behavior in `docs/domains/communications/README.md`.
- Add implementation ADRs before introducing persistence, route groups or
  automated contradiction resolution.
- Start future implementation with reviewable contradiction observations.
````

### `docs/adr/ADR-0086-first-class-relationship-persistence.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0086-first-class-relationship-persistence.md`
- Size bytes / Размер в байтах: `5208`
- Included characters / Включено символов: `5208`
- Truncated / Обрезано: `no`

````markdown
# ADR-0086 First-Class Relationship Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0045 Graph Core Projection
- ADR-0084 Persona Intelligence System
- ADR-0085 Communication Spine and Consistency / Contradiction Engine

## Context

Макошь is relationship-first. Current implementation stores relationship-shaped
data in several places:

- `graph_edges` as graph projection records;
- `relationship_events` as Persona timeline records;
- `person_roles` and `person_personas` as historical contact-era structures;
- organization, project and task link tables;
- trust and health fields on Persona and Organization read models.

This fragments the source of truth. It also conflicts with ADR-0084, which
requires Relationship records with source Persona, target Persona,
relationship type, trust score and strength score.

The graph remains essential, but graph edges are optimized for traversal and
projection. They should not be the only durable model for reviewed
relationships.

## Decision

Introduce first-class Relationship persistence.

The initial implementation creates a compatibility-safe `relationships` table
and a backend `relationships` domain store. The table stores a relationship as:

```yaml
Relationship:
  relationship_id:
  source_entity_kind:
  source_entity_id:
  target_entity_kind:
  target_entity_id:
  relationship_type:
  trust_score:
  strength_score:
  confidence:
  review_state:
  valid_from:
  valid_to:
  metadata:
```

Persona-to-Persona relationships are the first supported source path:

```yaml
source_entity_kind: persona
target_entity_kind: persona
```

This preserves the ADR-0084 model while leaving room for later relationships
between Organizations, Projects, Communications, Documents, Tasks, Decisions
and Obligations.

Each relationship must have evidence:

```yaml
RelationshipEvidence:
  relationship_id:
  source_kind:
  source_id:
  excerpt:
  metadata:
```

AI output may propose relationships, but accepted durable relationships remain
source-backed. Suggested relationships are stored with review state and
provenance; they are not silent truth.

`graph_edges` remain a derived graph traversal surface. The implementation
projects Relationship records between graph-supported entities as generic
`entity_relationship` graph edges, while preserving the Relationship record as
source of truth. The current supported projection endpoints match the current
`RelationshipEntityKind` set: Persona, Organization, Project, Communication,
Document, Task, Event, Decision, Obligation and Knowledge.

## Consequences

Positive:

- Relationship becomes a durable domain concept instead of a scattered field.
- Trust and strength scores have a clear owner.
- Persona Intelligence can depend on relationship records without treating
  Personas as CRM contacts.
- The graph can remain rebuildable from source relationships and evidence.
- Future Polygraph, Trust and Risk outputs can point to relationship records.

Negative:

- Existing relationship-like tables remain as compatibility or read-model
  surfaces until migration plans retire them.
- There is temporary duplication between `relationships`, graph edges and
  timeline events.
- The first desktop review UI is still surfaced inside the Personas workspace;
  broader cross-domain workflow placement and remaining compatibility adapters
  still need explicit follow-up work.

## Non-Goals

- Renaming `/persons` routes.
- Removing `graph_edges`.
- Removing `relationship_events`, `person_roles` or organization/project link
  tables.
- Automatically deriving trust from contradictions.

## Implementation Status

The backend now includes guarded routes for listing Relationship records by
entity and changing review state:

- `GET /api/v1/relationships?entity_kind=&entity_id=&limit=`
- `GET /api/v1/relationships?review_state=&limit=`
- `PUT /api/v1/relationships/{relationship_id}/review`

Review updates re-project active graph-supported Relationship edges so the
graph projection follows the Relationship source of truth.

The desktop frontend now includes a Personas workspace review panel for global
suggested Relationships. It uses the guarded global review list route, keeps
entity-scoped formatting when a Persona is selected and sends explicit owner
`user_confirmed` / `user_rejected` review state.

Manual/API `person_roles` now materialize source-backed `has_role`
Relationships from Persona to role Knowledge anchors. Removing a role demotes
the same Relationship to `user_rejected`.

Manual/API and email-sync `organization_contact_links`, manual `task_relations`
and explicit project link reviews now also materialize source-backed
Relationship records behind their compatibility surfaces.

## Required Follow-Up

- Move or duplicate Relationship review into a broader cross-domain review
  inbox when the workflow shell is defined.
- Reclassify remaining relationship-shaped compatibility and read-model
  surfaces into Relationship records.
- Feed reviewed Relationship records into Trust, Risk, Timeline and Dossier
  projections.
- Update implementation alignment docs as each compatibility surface is
  retired.
````

### `docs/adr/ADR-0087-contradiction-observation-persistence.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0087-contradiction-observation-persistence.md`
- Size bytes / Размер в байтах: `5212`
- Included characters / Включено символов: `5212`
- Truncated / Обрезано: `no`

````markdown
# ADR-0087 Contradiction Observation Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0023 Rebuildable Projections
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0086 First-Class Relationship Persistence

## Context

Макошь is a Personal Memory System. New Communications, Documents, Events,
Decisions and Obligations can contradict accepted Memory and Knowledge.

ADR-0085 introduced the Consistency / Contradiction Engine, user-facing alias
Polygraph. The repository currently has no durable backend representation for
Polygraph observations. Without persistence, contradictions cannot be reviewed,
linked to source evidence or fed into Memory, Trust, Risk and Relationship
semantics.

## Decision

Introduce `ContradictionObservation` persistence as the first implementation
slice of the Consistency / Contradiction Engine.

The engine stores reviewable observations:

```yaml
ContradictionObservation:
  observation_id:
  old_source_kind:
  old_source_id:
  new_source_kind:
  new_source_id:
  affected_entities:
  conflict_type:
  old_claim:
  new_claim:
  confidence:
  severity:
  review_state:
  metadata:
```

Initial review states are:

```yaml
ContradictionReviewState:
  suggested
  user_confirmed
  user_rejected
```

Initial severities are:

```yaml
ContradictionSeverity:
  low
  medium
  high
  critical
```

The first detection path operates on structured claims produced by upstream
extraction or deterministic tests:

```yaml
AcceptedClaim:
  subject_id:
  claim_type:
  value:
  source_kind:
  source_id:

NewEvidenceClaim:
  subject_id:
  claim_type:
  value:
  source_kind:
  source_id:
```

When a new claim has the same subject and claim type but a different normalized
value, the engine creates a `direct_contradiction` observation.

The engine must not:

- overwrite accepted Memory or Knowledge;
- change source records;
- mark a Persona as dishonest;
- adjust Relationship trust automatically;
- resolve the conflict without owner review or an explicit future policy.

## Consequences

Positive:

- Polygraph becomes a concrete backend engine baseline.
- Contradictions become source-backed and reviewable.
- Memory and Knowledge remain protected from silent mutation.
- Future Trust, Risk and Relationship engines can consume reviewed outcomes.

Negative:

- The detector still handles only direct contradictions and a small deterministic
  extraction baseline.
- Provider-wide ingestion and broader natural-language extraction from
  Communications and Documents remain outside this slice.

## Non-Goals

- Natural-language contradiction detection.
- Review UI.
- Automatic memory update.
- Automatic trust, risk or relationship score changes.
- Punitive judgments about Personas.

## Implementation Status

The backend now includes guarded routes for listing open contradiction
observations and changing review state:

- `GET /api/v1/contradictions`
- `PUT /api/v1/contradictions/{observation_id}/review`

These routes record API audit events and do not automatically overwrite Memory,
Trust, Risk or Relationship state.

`backend/src/engines/consistency.rs` also includes a deterministic extraction
baseline for simple structured Communication and Document evidence lines such
as `status: blocked` or `location=Madrid`, plus limited natural-language
patterns for `location` and `status` claims, such as `I am now in Madrid` or
`status is blocked`. This converts evidence text into `NewEvidenceClaim` values
and reuses the same direct-contradiction detector.

`ContradictionObservationStore::refresh_deterministic_observations` now
provides the first backend ingestion bridge for projected Communication
messages, imported Documents, meeting notes and call transcripts. It treats
active `person_facts` as accepted Memory claims, matches a Persona through the
compatibility `persons.email_address` field, active Telegram/WhatsApp
`person_identities`, `event_participants.person_id` or active Telegram call
identity, compares projected email message subject/body evidence by sender,
compares projected Telegram and WhatsApp message evidence by provider
`sender_id`, compares Document title/extracted-text evidence when the Document
text references the Persona email, compares meeting-note content for linked
event participants and compares successful call transcript text for linked
Telegram identities. It stores reviewable contradiction observations and does
not overwrite `person_facts`, Trust, Risk or Relationships.

The desktop frontend now includes a Knowledge workspace Polygraph review panel
that lists open contradiction observations through `GET /api/v1/contradictions`
and submits explicit owner review state through
`PUT /api/v1/contradictions/{observation_id}/review`.

## Required Follow-Up

- Expand ingestion wiring beyond projected email/Telegram/WhatsApp messages,
  imported Documents, meeting notes and call transcripts to broader provider
  evidence.
- Expand natural-language claim extraction beyond deterministic `location` and
  `status` patterns behind explicit review policy.
- Link reviewed outcomes to Memory, Trust, Risk and Relationship semantics.
````

### `docs/adr/ADR-0088-obligation-persistence.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0088-obligation-persistence.md`
- Size bytes / Размер в байтах: `5636`
- Included characters / Включено символов: `5636`
- Truncated / Обрезано: `no`

````markdown
# ADR-0088 Obligation Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0015 Command Query Separation
- ADR-0020 Task Candidate Lifecycle
- ADR-0070 Tasks First-Class Domain
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0087 Contradiction Observation Persistence

## Context

Макошь is a Personal Memory System. Communications, meetings, calls and
documents often contain commitments, duties and promises. The documentation
distinguishes three concepts:

- an Obligation is a commitment or duty backed by evidence;
- a Task is an actionable unit with status lifecycle;
- a Follow-Up is a prompt to revisit something.

Current implementation represents adjacent behavior through task candidates,
meeting outcomes, person promises and follow-up status. That is not enough for
the target model because it collapses the reason something matters into the
action that may be created from it.

## Decision

Introduce first-class Obligation persistence.

The initial implementation creates durable, source-backed obligation records:

```yaml
Obligation:
  obligation_id:
  obligated_entity_kind:
  obligated_entity_id:
  beneficiary_entity_kind:
  beneficiary_entity_id:
  statement:
  status:
  review_state:
  due_at:
  condition:
  risk_state:
  confidence:
  metadata:
```

Every durable Obligation must have evidence:

```yaml
ObligationEvidence:
  obligation_id:
  source_kind:
  source_id:
  quote:
  confidence:
  metadata:
```

Initial statuses:

```yaml
ObligationStatus:
  open
  fulfilled
  waived
  disputed
  canceled
```

Initial review states:

```yaml
ObligationReviewState:
  suggested
  user_confirmed
  user_rejected
```

Initial risk states:

```yaml
ObligationRiskState:
  none
  watch
  at_risk
  breached
```

Obligations may link to Tasks, but a confirmed Obligation must not
automatically create a Task. Task creation requires a separate user action,
policy or candidate review flow.

## Consequences

Positive:

- Макошь can remember commitments without forcing them into task lifecycle.
- Tasks can cite Obligations as reasons instead of becoming the source of truth
  for commitments.
- Consistency / Contradiction Engine can point at obligation status conflicts.
- Risk and Timeline engines can consume obligations later.

Negative:

- Existing person promises, meeting outcomes and task candidates remain
  compatibility or source surfaces. Initial adapters exist for person promises,
  selected meeting outcomes and obligation-derived task candidates, while
  broader routing remains follow-up work.
- The first desktop UI is scoped to the Tasks workspace; non-task-candidate
  Obligation review routing beyond accepted Obligations remains follow-up work.
- Obligation extraction remains limited and candidate-first. Explicit message
  and document task-candidate refresh paths now exist, while full automatic
  extraction across every provider stream remains follow-up work.

## Non-Goals

- Public `/obligations` API routes.
- Cross-domain workflow placement outside the Tasks workspace.
- Automatic task creation.
- Automatic obligation extraction from every message.
- Removing task candidates, meeting outcomes or person promises.

## Required Follow-Up

- Add candidate-to-Obligation review routing.
- Connect broader Communication, meeting and document extraction to obligation
  candidates beyond the explicit task-candidate paths.
- Expand adapters beyond the initial person promise and meeting outcome
  compatibility baselines.
- Expand reviewed Obligation links to additional compatibility sources.
- Feed obligation conflicts into the Consistency / Contradiction Engine.

## Implementation Status

The backend now has guarded accepted-Obligation list/review routes:

- `GET /api/v1/obligations?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/obligations?review_state=&limit=`;
- `PUT /api/v1/obligations/{obligation_id}/review`.

These routes update accepted Obligation review state only. They do not create
Tasks or create accepted Obligations from candidates.

The desktop frontend now includes a Tasks workspace review panel for global
suggested Obligations and Decisions, with optional entity-scoped filtering. It
uses the guarded list/review routes and sends only explicit owner
`user_confirmed` / `user_rejected` review state. It does not create Tasks or
convert candidates into accepted Obligations.

Migration `0066` and `ObligationStore` project accepted Obligations into graph
for supported obligated and beneficiary entity kinds. The projection creates
`obligation` graph nodes, source-backed `entity_relationship` edges and
`obligation` graph evidence while preserving the Obligation domain as the
source of truth.

Migration `0067` adds explicit task-candidate classification metadata.
`TaskCandidateStore` now creates `obligation_task` candidates from the
Obligation Engine for explicit message and document commitments. When the
candidate is user-confirmed, it creates or updates a source-backed
`user_confirmed` Obligation, preserves source evidence and links the created
Task through `obligation_task_links.link_kind = fulfillment_task`. Generic task
candidates remain task-only.

`PersonPromiseStore::create` now materializes compatibility `person_promises`
records into source-backed `user_confirmed` Obligations with `raw_record`
evidence. It does not create Tasks.

`MeetingOutcomeStore::add` now materializes meeting `promise`, `task` and
`follow_up` outcomes into source-backed `suggested` Obligations and stores the
created Obligation id in `meeting_outcomes.linked_entity_id`. It does not create
Tasks.
````

### `docs/adr/ADR-0089-decision-persistence.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0089-decision-persistence.md`
- Size bytes / Размер в байтах: `6118`
- Included characters / Включено символов: `6118`
- Truncated / Обрезано: `no`

````markdown
# ADR-0089 Decision Persistence

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0008 Knowledge Graph First
- ADR-0015 Command Query Separation
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0087 Contradiction Observation Persistence
- ADR-0088 Obligation Persistence

## Context

Макошь is a Personal Memory System. It must remember not only what happened,
but why a direction was chosen.

Current implementation has decision-shaped data in several places:

- meeting outcomes with `outcome_type = 'decision'`;
- project link review decisions;
- capability policy decisions;
- communication and document evidence that can imply decisions.

These are useful source or workflow surfaces, but none is the durable Decisions
domain described by the product model.

## Decision

Introduce first-class Decision persistence.

The initial implementation creates durable, source-backed Decision records:

```yaml
Decision:
  decision_id:
  title:
  status:
  rationale:
  alternatives:
  decided_by_entity_kind:
  decided_by_entity_id:
  decided_at:
  review_state:
  confidence:
  metadata:
```

Every durable Decision must have evidence:

```yaml
DecisionEvidence:
  decision_id:
  source_kind:
  source_id:
  quote:
  confidence:
  metadata:
```

Decisions also link to impacted entities:

```yaml
DecisionImpactedEntity:
  decision_id:
  entity_kind:
  entity_id:
  impact_type:
  metadata:
```

Initial statuses:

```yaml
DecisionStatus:
  active
  superseded
  reversed
  deprecated
```

Initial review states:

```yaml
DecisionReviewState:
  suggested
  user_confirmed
  user_rejected
```

A meeting outcome, project review or AI extraction may propose a Decision, but
it is not the Decision source of truth until stored as a source-backed Decision
record. Decision persistence does not automatically create Tasks, Projects or
Obligations.

## Consequences

Positive:

- Макошь can answer why a project, communication thread or workflow moved in a
  particular direction.
- Decisions become evidence-backed and reviewable instead of being hidden in
  meeting text, task notes or project state.
- Polygraph can point to conflicting decisions as reviewable contradictions.
- Projects, Documents, Communications, Events, Personas, Organizations, Tasks
  and Obligations can link to Decisions without owning decision truth.

Negative:

- Existing meeting outcomes and review decision tables remain compatibility or
  source surfaces. Initial adapters exist for meeting outcomes and project link
  review decisions, while broader routing remains follow-up work.
- The first desktop UI is scoped to the Tasks workspace; meeting/provider-wide
  candidate-to-Decision review flows are still follow-up work.
- Provider-wide Decision extraction from Communications and Meetings remains
  outside the first persistence slice.

## Non-Goals

- Cross-domain workflow placement outside the Tasks workspace.
- Automatic decision extraction.
- Automatic project status changes.
- Automatic task or obligation creation.
- Removing meeting outcomes or project link review decisions.

## Required Follow-Up

- Add candidate-to-Decision review routing.
- Connect meeting and provider-wide communication extraction to Decision
  candidates.
- Expand adapters beyond the initial meeting outcome and project link review
  baselines.
- Expand accepted Decision graph projection and project reviewed Decisions into
  timeline and dossier views.
- Feed conflicting Decisions into the Consistency / Contradiction Engine.

## Implementation Status

The backend now has guarded accepted-Decision list/review routes:

- `GET /api/v1/decisions?entity_kind=&entity_id=&limit=`;
- `GET /api/v1/decisions?review_state=&limit=`;
- `PUT /api/v1/decisions/{decision_id}/review`.

These routes update accepted Decision review state only. They do not create
Tasks, Projects, Obligations or accepted Decisions from candidates.

The desktop frontend now includes a Tasks workspace review panel for global
suggested Decisions and Obligations, with optional entity-scoped filtering. It
uses the guarded list/review routes and sends only explicit owner
`user_confirmed` / `user_rejected` review state. It does not create Tasks,
Projects or Obligations.

`backend/src/domains/decisions/extraction/` adds a deterministic candidate
detector for explicit Communication and Document evidence, for example
`Decision: Use local-first storage because private context must work offline`.
The detector produces reviewable Decision drafts and evidence references; it
does not persist accepted Decisions or mutate Projects, Tasks or Obligations.

Migration `0065` and `DecisionStore` project accepted Decisions into graph for
supported impacted entity kinds. The projection creates `decision` graph nodes,
source-backed `entity_relationship` edges and `decision` graph evidence while
preserving the Decision domain as the source of truth.

`DecisionStore::refresh_deterministic_candidates` now provides the first backend
candidate-to-Decision persistence path for explicit Communication messages and
imported Documents. It stores detected candidates as source-backed `suggested`
Decisions impacted by the source Communication or Document, preserves
`user_confirmed` and `user_rejected` review state across repeat refreshes, and
relies on the existing guarded Decision review route for confirmation. It does
not create Tasks, Projects or Obligations.

`MeetingOutcomeStore::add` now materializes meeting `decision` outcomes into
source-backed `suggested` Decisions impacted by the meeting Event and stores the
created Decision id in `meeting_outcomes.linked_entity_id`. It does not create
Tasks, Projects or Obligations.

`ProjectLinkReviewStore::set_review_state` and projection replay now
materialize explicit `user_confirmed` / `user_rejected` project link review
events into source-backed `user_confirmed` Decisions impacted by the Project and
the reviewed Communication or Document. This records the owner decision behind
the compatibility project-link surface without changing Project, Task or
Obligation state.
````

### `docs/adr/ADR-0090-persona-native-compatibility-api-bridge.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0090-persona-native-compatibility-api-bridge.md`
- Size bytes / Размер в байтах: `2707`
- Included characters / Включено символов: `2707`
- Truncated / Обрезано: `no`

````markdown
# ADR-0090 Persona-Native Compatibility API Bridge

Status: Proposed

Clarifies:

- ADR-0084 Persona Intelligence System

## Context

ADR-0084 defines Persona as the target domain entity, while current durable
storage and much of the compatibility API still use `persons`, `person_id` and
`/api/v1/persons/*`.

Макошь needs Persona-native read and write surfaces so new UI and agent flows
can speak the target language. A physical schema rename from `persons` to
`personas` is still a separate migration decision because existing routes,
tables, projections and tests depend on compatibility names.

## Decision

Expose a Persona-native compatibility API bridge under `/api/v1/personas/*`.

The bridge may read and write the current compatibility projection, but its
public contract uses Persona terminology and target-model shapes.

Allowed in this bridge:

- read Persona list/detail models from compatibility storage;
- update owner-editable Persona identity fields such as display name;
- set the single Owner Persona through Persona-native request fields;
- return the same Persona read model after writes;
- keep legacy identifiers in an explicit `compatibility` section.

Not allowed in this bridge:

- rename PostgreSQL tables, columns or migrations from `persons` to `personas`;
- remove `/api/v1/persons/*` compatibility routes;
- change Persona identity or `persona_type` without explicit validation rules;
- create separate Self/UserProfile storage;
- auto-merge identity traces without review.

The initial write bridge is intentionally narrow. It updates:

```yaml
PersonaUpdate:
  identity:
    display_name:
  is_self:
```

`is_self: true` sets the requested Persona as the only Owner Persona. `is_self:
false` is not a supported way to remove the Owner Persona; ownership must move
to another Persona instead.

## Consequences

Positive:

- New code can use Persona terminology without waiting for physical schema
  migration.
- Compatibility state remains stable for existing workflows.
- Owner Persona semantics are available through the target API language.
- The future schema migration has a clearer API contract to preserve.

Negative:

- The backend still contains compatibility names internally.
- The bridge must be maintained until the physical schema/API migration ADR is
  accepted and implemented.
- Some target-model fields remain read-only until their source-of-truth
  boundaries are defined.

## Follow-Up

- Design a physical schema migration ADR before renaming tables or columns.
- Expand write support only for fields with clear source-of-truth ownership and
  review semantics.
- Keep compatibility gaps visible in `docs/refactoring/implementation-alignment-plan.md`.
````

### `docs/adr/ADR-0091-telegram-production-client-capability-model.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0091-telegram-production-client-capability-model.md`
- Size bytes / Размер в байтах: `15599`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```markdown
# ADR-0091 Telegram Production Client Capability Model

Status: Proposed

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0018 Provider Adapter Boundary
- ADR-0026 Desktop First Responsive UI
- ADR-0031 Temporary Desktop Only UI Scope
- ADR-0046 Persistent Dev Mail Cache and Blob Storage
- ADR-0050 V4 Telegram Client, Policy Automation and Call Intelligence
- ADR-0052 Capability Runtime and Action Confirmation Policy
- ADR-0076 Host Vault on macOS
- ADR-0083 Telegram Live User Client Runtime
- ADR-0085 Communication Spine and Consistency / Contradiction Engine

Clarified by:

- ADR-0094 Telegram Base Domain Completion Boundary

## Context

Макошь already has a Telegram foundation: provider account records,
fixture accounts, QR/live-blocked account setup, a TDLib-oriented runtime
manager, chat/history sync endpoints, manual send routing, media download
facade, policy dry-runs, call metadata, transcript storage and a desktop
Telegram page.

The production target is larger. The owner expects Telegram account lifecycle,
multi-account operation, proxies, chat management, messages, soft delete,
message history, media, voice/video messages, calls, channels, groups, forums,
search, drafts, notifications, address book data, media gallery, offline mode,
exports and desktop UX.

This must not turn Макошь into a generic Telegram clone. Telegram remains a
Communication channel feeding source evidence into a local-first Personal Memory
System. Provider behavior is preserved at adapter and source-record boundaries;
canonical Communication, Memory, Relationship, Obligation, Decision, Task,
Document and Context behavior remains Макошь-owned.

External protocol references used by this ADR:

- Telegram documents SOCKS5 and MTProto proxy support for Telegram clients:
  <https://core.telegram.org/proxy>
- TDLib documents `addProxy` as a network proxy function that can be called
  before authorization:
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1add_proxy.html>
- TDLib documents SOCKS5 and MTProto proxy type objects:
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1proxy_type_socks5.html>
  and
  <https://core.telegram.org/tdlib/docs/classtd_1_1td__api_1_1proxy_type_mtproto.html>

## Decision

Implement production Telegram as a capability-gated Communication channel, not
as a separate messenger domain.

### Capability States

Every Telegram operation must be represented in a backend capability contract.
The UI may hide, disable or explain actions, but the backend remains the source
of authority.

Capability states:

- `available`: implementation, storage, policy, audit and validation exist.
- `blocked`: the feature is architecturally allowed but missing a required
  adapter, permission, secret, runtime dependency or validation gate.
- `degraded`: the feature was available but the current account/runtime cannot
  execute it reliably.
- `planned`: the feature is intentionally deferred to a named initiative and is
  not part of base Telegram channel capability completion.
- `unsupported`: the feature is intentionally out of current scope or conflicts
  with Макошь policy.

Capability action classes follow ADR-0052:

- `read`;
- `local_write`;
- `provider_write`;
- `destructive`;
- `export`;
- `secret_access`;
- `automation`.

Provider writes, destructive actions, sensitive exports and secret access require
explicit owner confirmation unless a future scoped automation policy permits a
narrow operation. AI and automation must not choose account, destination,
template, delete scope, export scope, call state or admin authority from
retrieved content.

### Accounts And Sessions

Telegram supports multiple `telegram_user` and `telegram_bot` accounts.

Rules:

- Account records store non-secret metadata only.
- Credential lookup uses `account_id + secret_purpose`; provider kind alone must
  never select credentials.
- New credential payloads live in the host vault per ADR-0076.
- Each live user account has an account-scoped runtime actor and account-scoped
  TDLib state path.
- Multiple account actors may run at the same time. Cross-account identity
  linking is a Persona/Relationship problem, not a Telegram account problem.
- Add account, authorization, logout, session import and session export are
  lifecycle commands and must emit auditable events.
- Account removal disables the account and stops its runtime. Existing local
  source evidence is retained unless a separate explicit destructive purge
  capability is implemented.
- Logout may revoke or discard provider session state only after explicit owner
  confirmation and must not delete canonical Communication evidence.
- Session import/export bundles are sensitive. They must be encrypted,
  account-scoped, manifest-backed and require host-vault unlock. They must not
  contain plaintext API hashes, bot tokens, session encryption keys or proxy
  secrets.

### Proxies

Telegram user-account runtime may support SOCKS5 and MTProto proxy profiles.

Rules:

- Proxy profiles are account-scoped or runtime-scoped non-default configuration.
- Proxy secrets and SOCKS5 passwords are secret payloads, not application
  settings.
- Proxy host, port, kind, label and non-secret status may be stored as metadata.
- A proxy change requires runtime restart or TDLib proxy command execution
  through the account actor.
- Proxy status, last error and active profile must be visible in the UI.
- Proxy testing must not leak credentials into logs, audit records or telemetry.

### Source Evidence And Local Truth

Telegram source data must enter Макошь through append-only raw source records and
canonical events.

Rules:

- Raw provider records are immutable source evidence.
- Canonical `communication_messages` are projections from source records.
- Search indexes, media galleries, unread views, folders, pinned views and
  desktop notifications are derived or provider-overlay state.
- Message bodies, media bytes, call audio and document bytes must not be written
  to audit records.
- Media bytes live in local blob storage, not PostgreSQL. PostgreSQL stores
  metadata, hashes, scanner state and local references.
- Attachment metadata must pass through the attachment safety scanner boundary.
  A no-op scanner records `not_scanned`; it must not mark Telegram attachments
  as `clean`.

### Chats

Chats, groups, channels, forums, archived chats, pinned chats, folders and hidden
views are account-scoped projections over provider state plus local owner
overlays.

Rules:

- Listing, sorting, filtering and searching chats are read operations.
- Mark read/unread, archive/unarchive, pin/unpin and folder changes are provider
  writes when mirrored to Telegram; local-only overlays are local writes and must
  be labeled as local-only.
- Hide is a local owner overlay unless a provider-backed equivalent is
  explicitly implemented.
- Delete chat is destructive. It must create a local tombstone and require
  explicit confirmation before any provider-side deletion request.
- Forum topics are first-class thread projections under a chat. Topic create,
  close, delete, pin and reply actions are provider writes or destructive
  actions according to their effect.

### Messages

Message send, edit, delete, reply, forward, reaction, pin, saved-message,
export, jump-to-message and reply-thread actions must be modeled as explicit
commands.

Rules:

- Drafting is local. Sending is a provider write.
- Reply, forward, edit, reaction and pin commands must preserve target message
  identifiers, account, chat, actor and preview hash in audit metadata. Message
  body content must not be stored in audit records.
- Multi-select is UI state until a bulk command is submitted. Bulk commands must
  expand to per-message command records so failures are explainable.
- Copy to clipboard is local UI behavior and should not create durable state
  unless the copied item is saved or exported.
- Save message creates a local source-backed saved item, not a Telegram source
  mutation unless Telegram Saved Messages write support is explicitly selected.
- Jump-to-message and open-reply-thread are read/navigation operations.

### Soft Delete And Message History

Макошь never physically deletes local Telegram message evidence as the default
delete behavior.

Rules:

- Local delete creates a tombstone event.
- Tombstones record reason, actor class, observed time, target message and
  source event.
- Supported reason classes include `deleted_by_owner`,
  `deleted_by_counterparty`, `deleted_by_provider`, `moderation_removed`,
  `account_removed`, `retention_policy` and `unknown`.
- UI must show deletion reason, deletion time and deletion history where known.
- Provider-side delete requests are destructive commands and must still leave
  the local tombstone and historical source evidence.
- Message edit history is versioned from observed updates. Макошь must not claim
  to reconstruct provider edit versions that were never observed locally.
- Version diff views compare local observed versions and must cite source
  records or events.

### Attachments And Media

Telegram media is modeled as Communication attachments and local blob objects.

Supported target content classes:

- images;
- video;
- documents;
- PDF;
- DOCX;
- XLSX;
- ZIP;
- audio;
- voice messages;
- video messages;
- contact cards;
- locations;
- GIF;
- stickers;
- links and link previews.

Rules:

- Download is a provider read plus local blob write.
- Open and preview read local cached bytes where available and otherwise request
  an explicit download.
- Save local and save to Documents are local writes; save to Documents creates or
  links a Document domain artifact.
- Share and export are export actions and require policy/audit handling.
- Auto-download defaults must be conservative by size, chat trust and media
  class.
- Media gallery views are derived from attachment metadata and local blob state.

### Voice Messages, Video Messages And Calls

Voice messages and video messages are message attachments with capture, playback,
download and send workflows. Recording requires desktop permission handling and a
visible recording state.

Audio and video calls are separate from message sync and require their own live
runtime validation before becoming available.

Rules:

- Playback speed and replay are local UI state.
- Voice/video message send is a provider write with attachment upload.
- Call history can be source evidence.
- Accept, decline and redial are provider writes.
- Audio/video call start, accept and device selection require a native desktop
  permission and media-device boundary.
- Hidden recording is unsupported.
- Transcription remains local by default and must be account/chat/policy scoped.
- Video calls, group calls and screen sharing require separate ADR-backed runtime
  work before being marked `available`.

### Channels, Groups And Forums

Channels, groups and forums are Telegram chat types with additional provider
permissions.

Rules:

- Subscribe/join, unsubscribe/leave, create group, delete group, invite/remove
  participants and topic moderation are provider writes or destructive actions.
- Participant lists and searches are reads but may be rate-limited by provider
  visibility and permissions.
- Channel publication reading and search are read operations.
- Grouping, archiving and pinning are overlays or provider writes depending on
  whether they are synchronized to Telegram.
- Admin actions must check account permission state before command submission.

### Search, Drafts, Notifications And Offline

Global Telegram search participates in the Макошь Search Engine. Search results
must distinguish local cache hits from provider search results.

Rules:

- Search supports messages, files, images, video, links, Personas, groups,
  channels, date ranges, sender and attachment type as target filters.
- Search indexes are derived and rebuildable.
- Drafts are local-first records with autosave, restore and dele
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `docs/adr/ADR-0092-mail-provider-capability-tiers.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0092-mail-provider-capability-tiers.md`
- Size bytes / Размер в байтах: `4008`
- Included characters / Включено символов: `4008`
- Truncated / Обрезано: `no`

```markdown
# ADR-0092 Mail Provider Capability Tiers

Status: Proposed
Date: 2026-06-13

Clarifies:

- ADR-0041 Email Provider Ingestion Foundation
- ADR-0055 Full Email Provider Networking
- ADR-0076 Host Vault on macOS
- ADR-0080 Mail Background Sync, Progress and Local Trash

## Context

Макошь mail has a provider-neutral storage boundary, Gmail OAuth setup,
iCloud/generic IMAP account setup, SMTP sending for IMAP-backed accounts,
background sync status, local trash and message projections.

The requested working mail scope is wider than the current provider model:
POP3, Exchange, Microsoft 365, Fastmail, Mail.ru, Yandex and Proton have
different protocol semantics. Treating all of them as identical `imap`
accounts would hide real capability differences:

- POP3 has no durable server folders, labels or flag mutation contract.
- Microsoft 365 and Exchange Online should prefer Microsoft Graph OAuth over
  IMAP basic credentials.
- Legacy/on-prem Exchange may require EWS or a local bridge.
- Proton support should normally use Proton Mail Bridge locally; Макошь must not
  attempt to handle Proton account passwords directly.
- Fastmail, Mail.ru and Yandex can work as IMAP/SMTP presets before they need
  first-class provider kinds.

## Decision

Mail providers are modeled as capability tiers. A provider account exposes
capabilities instead of implying that every account can read, send, mutate
folders, mutate flags, sync labels, use OAuth, or expose provider-native
threads.

Initial tiers:

| Tier | Examples | Storage provider kind | Notes |
|---|---|---|---|
| Native API | Gmail, Microsoft 365 | provider-specific | Uses OAuth and provider APIs for read/write when implemented. |
| Standards IMAP/SMTP | iCloud, Fastmail, Mail.ru, Yandex, generic IMAP | `icloud` or `imap` | Uses IMAP for read/sync and SMTP for send. Provider presets are UI/config helpers, not new domain kinds by default. |
| POP3/SMTP | legacy mailboxes | future `pop3` ADR/migration | Ingestion-only mailbox semantics; no provider folder or flag mutation contract. |
| Exchange legacy | on-prem Exchange | future adapter | Requires EWS or a local bridge; not modeled as generic IMAP unless explicitly configured that way. |
| Proton Bridge | Proton Mail | `imap` with bridge metadata | Connects only to a user-run local Proton Bridge IMAP/SMTP endpoint. |

Rules:

- Provider account records continue to store only non-secret metadata and
  adapter configuration.
- Credential lookup remains account-scoped by `account_id` and secret purpose.
- Runtime/UI capability checks must decide whether actions such as send, move,
  copy, label mutation, server delete and folder sync are available.
- UI must not present unavailable provider operations as working actions.
- Local trash remains the default delete behavior from ADR-0080 and must not be
  silently converted into provider delete.
- Adding a durable provider kind outside `gmail`, `icloud` and `imap` requires a
  schema migration and explicit tests for capability reporting.

## Consequences

Positive:

- The account UI can support common providers through presets without inventing
  false native semantics.
- Provider-specific write behavior stays explicit and testable.
- POP3, Microsoft Graph, EWS and Proton Bridge can be added incrementally
  without breaking existing Gmail/IMAP accounts.

Negative:

- Some requested provider names initially map to presets or unsupported
  capability rows rather than full native adapters.
- Capability reporting becomes a required part of account management.
- The current provider account table check constraint must be expanded before
  adding durable non-Gmail/IMAP provider kinds.

## Follow-Up

- Add account management API that returns provider capabilities and sanitized
  account config.
- Add provider presets for Fastmail, Mail.ru, Yandex, Microsoft and Proton
  Bridge in the frontend account wizard.
- Design separate migrations before introducing `pop3`, `microsoft_graph` or
  `exchange_ews` provider kinds.
```

### `docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0093-frontend-platform-migration-to-vue-3.md`
- Size bytes / Размер в байтах: `10212`
- Included characters / Включено символов: `10080`
- Truncated / Обрезано: `no`

````markdown
# ADR-0093: Frontend Platform Migration to Vue 3

Status: Accepted

## Date

2026-06-14

## Deciders

Alex (makosh maintainer)

## Supersedes

- ADR-0003 — SvelteKit Frontend
- ADR-0078 — Frontend Component Decomposition — SPA Pages and Widgets
- ADR-0079 — Script Logic Decomposition — Services, Smart Pages and Config

---

## Context

Макошь evolves from a small desktop application into a long-term Personal
Memory System that integrates:

- Mail
- Telegram
- Personas
- Organizations
- Tasks
- Calendar
- Documents
- Notes
- Knowledge Graph
- AI Agents

The project is built as a desktop-first application using:

- Rust
- Tauri

Макошь is not a traditional website and does not require:

- SEO
- Public SSR pages
- Search engine indexing
- Edge rendering

The frontend must support:

- Large datasets
- Real-time updates
- Virtualized lists
- Complex workspaces
- Multi-panel layouts
- Knowledge graph visualization
- Outlook-like mail experience
- Telegram-like messaging experience
- Obsidian-like knowledge workflows

The current SvelteKit architecture introduces additional complexity while
providing limited value in a desktop-only environment:

- SvelteKit's routing model (`load()`, server actions, filesystem routes) adds
  abstraction overhead for a SPA that never needs SSR.
- The Svelte 5 runes migration (`$state`, `$derived`, `$effect`) requires a
  significant conceptual shift with limited ecosystem support.
- Svelte's smaller ecosystem means fewer mature libraries for complex desktop
  UI patterns (virtualized tables, rich text editors, graph visualization).
- Component libraries (shadcn-svelte, etc.) lag behind their Vue/React
  counterparts in stability and feature completeness.
- Developer familiarity and long-term maintenance risk are concerns given the
  planned scope of the project.

---

## Decision

The frontend platform will migrate from:

```text
SvelteKit
```

to:

```text
Vue 3 + TypeScript + Vite + Tauri 2
```

SvelteKit-specific concepts are removed from the architecture:

- `load()` functions
- server actions
- SSR routing model
- Kit-specific data loading patterns
- Runes (`$state`, `$derived`, `$effect`)
- Svelte stores (`writable`, `derived`)

The frontend becomes a pure client application communicating with Rust services
through:

- HTTP API
- Server-Sent Events (SSE)
- WebSocket
- Tauri Commands

---

## Selected Technology Stack

### Core

| Technology | Purpose |
|------------|---------|
| Vue 3 | UI framework (Composition API, `<script setup>`) |
| TypeScript | Type safety |
| Vite | Build tool (already in project) |
| Tauri 2 | Desktop shell (already in project) |

### State Management

| Library | Scope |
|---------|-------|
| Pinia | UI state, layout state, user preferences, workspace state, temporary client state |
| TanStack Query | Server state, caching, synchronization, invalidation, background refresh |

**Pinia** is used only for transient client state:

- active tab
- selected mail
- current workspace
- sidebar state
- theme preferences

**TanStack Query** owns all server-derived state:

- query keys follow domain structure (`['mail', 'list', accountId]`)
- mutations invalidate related queries
- background refetch replaces manual refresh logic
- stale-while-revalidate eliminates loading spinners

Direct API calls inside components are **prohibited**.

Allowed:

```ts
useMailListQuery()
useTelegramChatQuery()
usePersonQuery()
```

Forbidden:

```ts
await axios.get(...)
```

inside components (except in query definitions).

### Data Grid

| Library | Usage |
|---------|-------|
| `@tanstack/vue-table` | Mail, tasks, documents, search results, personas, organizations |

### Virtualization

| Library | Requirement |
|---------|-------------|
| `@tanstack/vue-virtual` | Required for all potentially large collections: mail lists, Telegram chats, documents, timeline views |

### Rich Text Editor

| Library | Usage |
|---------|-------|
| TipTap | Notes, documents, email composer, AI-generated content |

### Knowledge Graph

| Library | Usage |
|---------|-------|
| Vue Flow | Foundation for graph visualization |

### Dates

| Library | Role |
|---------|------|
| `date-fns` | Standard project date library |

### UI Foundation

| Library | Role |
|---------|------|
| Tailwind CSS | Utility-first CSS framework |
| shadcn-vue | Accessible component primitives (become project-owned code) |

Макошь UI is not tied to any third-party design system. shadcn-vue components
are copied into the project and customized as needed.

### Animations

| Library | Role |
|---------|------|
| Motion (preferred) | Page transitions, workspace transitions, drawer animations, docking animations |
| GSAP (optional) | Complex timeline animations |

---

## Frontend Architecture

The frontend follows Domain-Driven Design.

### Forbidden top-level structure

```text
src/components/
src/pages/
src/stores/
```

### Required top-level structure

```text
src/
├── app/           # Application shell, routing, layout
├── domains/       # Domain-specific code
├── widgets/       # Cross-domain workspace compositions
├── shared/        # Reusable UI components and utilities
└── platform/      # Framework bindings, API client, auth, SSE/WebSocket
```

### Domain Structure

Each domain is self-contained:

```text
domains/
└── mail/
    ├── api/           # API functions for this domain
    ├── queries/       # TanStack Query keys, hooks, mutations
    ├── stores/        # Pinia stores for transient UI state
    ├── views/         # Domain-specific view components
    ├── routes/        # Route definitions (if this domain has routes)
    ├── types/         # TypeScript types for this domain
    └── components/    # Domain-specific components
```

---

## Component Hierarchy

### Level 1 — UI primitives

```text
shared/ui/
  Button.vue
  Input.vue
  Dialog.vue
  Dropdown.vue
```

### Level 2 — Domain components

```text
domains/communications/components/
  MailListItem.vue
  MailAttachmentChip.vue

domains/telegram/components/
  TelegramMessageBubble.vue
```

### Level 3 — Features

```text
domains/communications/views/
  MailList.vue
  MailViewer.vue

domains/telegram/views/
  TelegramChat.vue

domains/persons/views/
  PersonProfile.vue
```

### Level 4 — Widgets / Workspaces

```text
widgets/
  MailWorkspace.vue
  TelegramWorkspace.vue
  TimelineWorkspace.vue
```

### Level 5 — Routes

```text
app/routes/
  MailPage.vue
  TelegramPage.vue
  SettingsPage.vue
```

---

## Migration Path

### Phase 1 — Foundation (parallel to existing SvelteKit app)

1. Initialize Vue 3 + Vite project in `frontend/` alongside current code.
2. Set up Tailwind CSS, shadcn-vue, Pinia, TanStack Query.
3. Build shared UI primitives (`shared/ui/`).
4. Port `platform/` layer (API client, auth header, SSE/WebSocket client).
5. Port i18n system (dictionary-based, compatible with existing `ru.json`).
6. Port theme system (CSS custom properties → Tailwind + Vue reactivity).

### Phase 2 — Domain by domain

7. Port one domain at a time, starting with simplest (settings, then home
   dashboard, then increasingly complex domains).
8. Each domain includes types, API functions, queries, store, views, components.
9. After each domain, verify the Vue app can render it correctly.
10. Old SvelteKit code remains until all domains are ported.

### Phase 3 — Cutover

11. Once all domains are ported, remove SvelteKit code and dependencies.
12. Update Tauri configuration to point to the new build output.
13. Run full validation gate.

---

## Rules

### Shared Component Rule

Components must not be moved to `shared/` prematurely. A component enters
shared only after proven reuse across at least two domains.

### God Component Rule

| Threshold | Action |
|-----------|--------|
| 300+ lines | Warning — consider decomposition |
| 500+ lines | Architecture review required |
| 700+ lines | Architecture violation |

### Server State Rule

All server-derived state must go through TanStack Query. Pinia stores must
not cache server data — only transient UI state.

### Real-Time First Rule

Polling is discouraged. Preferred:

- SSE for server-to-client event streams
- WebSocket for bidirectional communication
- Event-driven updates via TanStack Query invalidation

---

## Consequences

### Benefits

- Better ecosystem maturity and library availability.
- Larger component ecosystem (shadcn-vue, Vue Flow, TipTap all have strong Vue
  support).
- Better support for complex desktop application patterns (virtualized tables,
  real-time updates, graph visualization).
- Alignment with Domain-Driven Design — frontend structure mirrors backend
  domain boundaries.
- TanStack Query eliminates manual server state management (loading/error
  states, caching, background refresh, stale detection).
- Type safety across the stack — no `as T` casts, no `any` API responses.
- Better developer familiarity and long-term maintainability.

### Trade-offs

- Significant migration effort — all existing Svelte components must be
  rewritten.
- The app is non-functional during the migration window (until Phase 3
  cutover).
- Team must standardize on Vue 3 Composition API patterns.
- Some UI components will need complete redesign for the new framework.
- Risk of regressions during domain-by-domain porting.

### Risk Mitigation

- Migration is phased — each domain is ported and verified independently.
- Shared infrastructure (i18n dictionary, API client contract, theme tokens)
  ports directly without semantic change.
- Test coverage for critical domains (communications, AI settings) provides
  a regression safety net.
- The existing SvelteKit app remains functional throughout Phases 1-2.

---

## References

- ADR-0003 — SvelteKit Frontend (superseded)
- ADR-0004 — Tauri Desktop Shell (unchanged)
- ADR-0026 — Desktop First Responsive UI (unchanged)
- ADR-0031 — Temporary Desktop Only UI Scope (unchanged)
- ADR-0077 — i18n Russian and English Interface (unchanged)
- ADR-0078 — Frontend Component Decomposition (superseded)
- ADR-0079 — Script Logic Decomposition (superseded)
````

### `docs/adr/ADR-0094-telegram-base-domain-completion-boundary.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0094-telegram-base-domain-completion-boundary.md`
- Size bytes / Размер в байтах: `4417`
- Included characters / Включено символов: `4417`
- Truncated / Обрезано: `no`

```markdown
# ADR-0094 Telegram Base Domain Completion Boundary

Status: Superseded by ADR-0097
Date: 2026-06-18

Superseded note: ADR-0097 replaces the "Telegram Channel" operating-surface
framing. Telegram remains a Communication Channel capability set and integration
adapter; it is not a product/backend/frontend domain.

Clarifies:

- ADR-0052 Capability Runtime and Action Confirmation Policy
- ADR-0083 Telegram Live User Client Runtime
- ADR-0091 Telegram Production Client Capability Model
- ADR-0093 Frontend Platform Migration to Vue 3

## Context

Telegram has reached the base Communication Channel scope for Макошь
Communications. It provides source evidence, provider commands, communication
projections, realtime events, identity traces, timeline evidence and media
evidence for other Макошь systems.

Telegram must not become a Memory Engine, Knowledge Engine, Persona Engine,
Organization Engine, Project Engine, Obligation Engine or Decision Engine.
Those systems consume Telegram evidence through existing Макошь boundaries.

Several requested Telegram-adjacent features remain valuable, but they require
separate runtime, permissions, security, media-device or AI review work. Keeping
them inside the base Telegram completion scope would hide unfinished
architecture behind a broad domain label.

## Decision

The base Telegram channel capability set is completed and moves to maintenance
once the implementation, tests and documentation agree that P0
provider-command, lifecycle, reply/forward, topic, dialog, search and media
parity are closed for the supported scope.

Capability states now include:

- `available`: implementation, storage, policy, audit and validation exist.
- `blocked`: architecturally allowed but blocked by a missing runtime,
  dependency, permission, credential or validation gate in the current account.
- `degraded`: implemented but currently running with reduced provider/runtime
  confidence.
- `planned`: intentionally deferred to a named initiative and not part of base
  Telegram completion.
- `unsupported`: intentionally outside Макошь policy or incompatible with the
  current Telegram account/runtime.

The following capabilities are `planned`, not base-domain gaps:

- Bot Runtime;
- Voice Recording;
- Voice Send;
- Video Recording;
- Live Calls;
- Session Export;
- Session Import;
- MTProxy;
- SOCKS5;
- AI Summary;
- Translation;
- Bilingual Reply;
- AI Review Flows.

Rules:

- `planned` capabilities must be visible in the backend capability contract and
  frontend capability matrix.
- `planned` does not authorize implementation inside a Telegram product domain.
- Future work for Bot Runtime, Voice, Calls and AI Layer must start as separate
  initiatives with their own ADR or ADR update before implementation.
- Provider writes continue to use the durable outbox.
- Destructive actions continue to require audit.
- ACK from TDLib or another provider adapter is not success. Provider-write
  commands complete only after provider-observed state or an explicit provider
  result snapshot that carries the durable evidence needed by the projection.
- Telegram UI must use projected/sanitized evidence, not raw TDLib payloads
  directly.
- TanStack Query owns Telegram server state in the frontend; component-level
  fetch remains forbidden.

## Consequences

Positive:

- Base Telegram can be treated as a maintained Communication Channel instead of
  a permanently open feature bucket.
- Deferred features remain discoverable to users and tests through capability
  status without being mislabeled as broken or unsupported.
- Cross-domain systems keep consuming Telegram evidence without Telegram owning
  Memory, Knowledge, Persona, Organization, Project, Obligation or Decision
  behavior.

Negative:

- Some feature requests need a new initiative even if they are Telegram-branded
  in the UI.
- Capability consumers must handle five states instead of four.

## Validation

Completion requires:

- no confirmed `BROKEN` or `REGRESSION` Telegram capability;
- no base-domain P0 gap in Telegram gap analysis;
- no Telegram implementation, test or frontend source file over 700 lines;
- provider writes through outbox;
- destructive actions through audit;
- realtime events through the shared event bus;
- no runtime polling where a realtime provider path exists;
- no component-level Telegram fetch;
- Telegram documentation aligned with the implemented code.
```

### `docs/adr/ADR-0095-event-driven-domain-communication-and-dlq.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0095-event-driven-domain-communication-and-dlq.md`
- Size bytes / Размер в байтах: `3521`
- Included characters / Включено символов: `3521`
- Truncated / Обрезано: `no`

````markdown
# ADR-0095 Event-Driven Domain Communication and DLQ

Status: Accepted

## Context

Макошь is moving to a layered architecture where integrations, domains, engines
and UI modules do not call each other's handlers or domain services directly.
Communication channels such as Email, Telegram and WhatsApp should publish
source evidence and provider state. Domain owners should react through durable
events and explicit promotion/review flows.

The repository already has an append-only `event_log`, canonical event
envelopes and projection cursors. That is enough for replayable projections, but
not enough for reliable cross-domain communication. Inter-domain consumers need
their own checkpoints, retry state and dead-letter handling so one poisoned event
does not silently skip, loop forever or block unrelated consumers.

## Decision

All cross-domain communication must use the event system.

Rules:

- API handlers call only the command/application service of their own bounded
  context.
- A domain must not import another domain's handlers, stores or services for
  synchronous business behavior.
- Integrations must not call Personas, Tasks, Documents, Projects,
  Organizations, Knowledge or other business domains directly.
- Cross-domain intent is expressed as a versioned event.
- The owning domain consumes that event and decides whether to create or update
  its own durable state.
- Event consumers are at-least-once and must be idempotent.
- Consumer cursors move only after successful handling or after the event is
  durably moved to the consumer's dead-letter queue.
- Retry and DLQ state are per consumer. One consumer failing must not prevent
  other consumers from processing the same event.

The event platform adds:

```text
event_consumers
event_consumer_failures
event_consumer_processed_events
event_dead_letters
```

`event_consumers` stores each durable consumer position. `event_consumer_failures`
stores retry state and backoff. `event_consumer_processed_events` records the
per-consumer event positions already applied so duplicate delivery and cursor
rewind do not invoke the handler again for the same event. `event_dead_letters`
stores poison events for owner review and manual replay.

Canonical event families include:

```text
communication.*
integration.email.*
integration.telegram.*
integration.whatsapp.*
radar.*
persona.*
task.*
document.*
knowledge.*
relationship.*
```

The first target flow is:

```text
integration.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
  -> radar.promotion.requested
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

## Consequences

Positive:

- Domains become isolated bounded contexts.
- Integrations stop owning business meaning.
- Retry, DLQ and replay semantics are centralized.
- Cross-domain behavior becomes auditable and replayable.
- Architecture linting can enforce boundaries in CI.

Negative:

- Existing synchronous cross-domain imports become legacy debt until refactored.
- Event contracts require version discipline.
- Consumers must be explicitly idempotent.
- Some workflows become eventually consistent instead of synchronous.

## Implementation Notes

Superseded by `ADR-architecture-communication-contract`.

The event/DLQ rules in this ADR remain accepted, but the temporary boundary
baseline is abolished. Existing violations must be removed or moved behind the
communication contract. `scripts/architecture-boundary-baseline.json` must not
exist.
````

### `docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0096-canonical-evidence-review-and-context-packs.md`
- Size bytes / Размер в байтах: `4966`
- Included characters / Включено символов: `4966`
- Truncated / Обрезано: `no`

````markdown
# ADR-0096 Canonical Evidence, Review Inbox and Context Packs

Status: Accepted

## Context

Макошь already has events, source-backed communications, decisions,
obligations, relationships and reviewable candidates. The missing boundary was
the layer between provider/runtime captures and domain truth.

Provider records are not domain objects. An email message, browser capture,
voice memo, PDF import or meeting transcript should first become evidence. Only
later may ingestion or review promote that evidence into Personas,
Organizations, Meetings, Tasks, Decisions, Obligations, Relationships,
Documents, Projects or Knowledge.

ADR-0001 keeps the event log as the system spine. ADR-0095 keeps cross-domain
communication event-driven. This ADR defines the durable evidence and review
ownership model that sits between integrations and domains.

## Decision

Макошь uses this target flow:

```text
External Systems
  -> Integrations
  -> Vault
     (accounts / capabilities / sources / sessions)
  -> Observation Platform
     (canonical evidence)
  -> Ingestion
  -> Domains
  -> Knowledge
  -> Review
     (inbox / promotion / approval / dismissal)
  -> Actions
```

The Observation Platform is the Canonical Evidence Store. It is a platform
layer, not part of Vault and not a business domain.

Core invariants:

- Observation is evidence, not truth.
- Observations are append-only.
- External deletion or mutation creates another observation. It does not mutate
  or delete the previous observation.
- Vault owns provider accounts, capabilities, sources and sessions. Vault does
  not own observations.
- Review is a domain and the main Макошь inbox for reviewable material.
- Radar remains attention vocabulary and read-model language. It is not a
  durable domain.
- Context Packs are engine output under `engines/context_packs/`. They are
  derived and rebuildable from observations, domains, knowledge, relationships
  and prior decisions.
- Do not create `domains/signals`, `domains/events`, `domains/attention` or
  `domains/evidence`.

Canonical observation kinds are registry-backed. Initial kinds include:

```text
COMMUNICATION_MESSAGE
COMMUNICATION_MESSAGE_DELETED
COMMUNICATION_ATTACHMENT
MEETING
MEETING_RECORDING
MEETING_TRANSCRIPT
DOCUMENT
VOICE_RECORDING
BROWSER_CAPTURE
CONTACT_RECORD
CALENDAR_EVENT
```

Review item kinds include:

```text
new_person
new_organization
potential_task
potential_obligation
potential_decision
potential_relationship
potential_project
knowledge_candidate
```

Review lifecycle states are:

```text
new
in_review
approved
promoted
dismissed
archived
```

Event flow:

```text
observation.captured.v1
persona.detected.v1
organization.detected.v1
task.candidate.detected.v1
decision.candidate.detected.v1
obligation.candidate.detected.v1
relationship.candidate.detected.v1
knowledge.candidate.detected.v1
review.item.available.v1
review.item.approved.v1
review.item.promoted.v1
review.item.dismissed.v1
```

Identity resolution and relationship detection are separate engines:

- `engines/identity_resolution` decides whether two subjects represent the same
  entity.
- `engines/relationships` decides whether two entities are linked and how.

## Consequences

Positive:

- Provider records stop being promoted directly into domain truth.
- Manual notes, browser captures, voice recordings and imported documents can
  create observations without Vault.
- Provider deletion is represented as evidence, not destructive data loss.
- Review becomes one concrete inbox instead of scattering promotion state across
  Radar, Tasks, Knowledge and candidates.
- Context packs have a real home without becoming source-of-truth records.
- Architecture guard can reject forbidden evidence/attention/signal domains and
  Vault-owned observations.

Negative:

- Existing communication and provider ingestion paths need gradual migration to
  create observations before domain candidates.
- Domains that already store source references need compatibility bridges until
  evidence links are backfilled.
- Review promotion is eventually consistent when downstream domains consume
  events instead of being synchronously called.

## Implementation Notes

The initial implementation adds:

- `observation_kind_definitions`;
- append-only `observations`;
- `observation_links`;
- `observation_ingestion_runs`;
- `review_items`;
- `review_item_evidence`;
- `context_packs`;
- `context_pack_sources`;
- backend modules for `platform::observations`, `domains::review` and
  `engines::context_packs`;
- lightweight engine contracts for identity resolution and relationship
  candidates;
- architecture guard checks for forbidden domain directories and Vault-owned
  observations.

The first implementation does not rename existing provider/source tables or
force all existing task creation paths through observations. That migration
requires a separate compatibility plan because current domains still expose
legacy source/evidence fields.
````

### `docs/adr/ADR-0097-communications-channel-domains-to-integrations.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0097-communications-channel-domains-to-integrations.md`
- Size bytes / Размер в байтах: `5940`
- Included characters / Включено символов: `5940`
- Truncated / Обрезано: `no`

````markdown
# ADR-0097 Communications Channel Domains To Integrations

Status: Accepted
Date: 2026-06-20

Supersedes:

- ADR-0094 Telegram Base Domain Completion Boundary, for the use of
  "Telegram Channel" as an operating surface or bounded-context label.

Clarifies:

- ADR-0041 Email Provider Ingestion Foundation
- ADR-0051 WhatsApp Web Companion Boundary
- ADR-0055 Full Email Provider Networking
- ADR-0085 Communication Spine and Consistency / Contradiction Engine
- ADR-0091 Telegram Production Client Capability Model
- ADR-0092 Mail Provider Capability Tiers
- ADR-0093 Frontend Platform Migration to Vue 3
- ADR-0095 Event-Driven Domain Communication and DLQ

Route note: ADR-0098 supersedes the intermediate provider-scoped business route
decision from this ADR. Channels remain integrations, but business
Communications APIs are now provider-neutral under `/api/v1/communications/*`;
provider setup/runtime APIs live under `/api/v1/integrations/*`.

## Context

Макошь accumulated channel-shaped implementation surfaces while building email,
Telegram and WhatsApp support. Email started as the first communication
implementation. Telegram later gained a large account/runtime/message UI and was
documented as a completed "base domain". WhatsApp documentation also described a
future channel-specific workbench.

That language is now misleading. A channel is not a domain. A channel is an
integration. A communication is the domain object.

The long-term product boundary is Communications:

```text
Communication -> Source Evidence -> Extracted Knowledge -> Memory -> Context
```

Mail, Telegram and WhatsApp provide source records, account metadata, runtime
state and provider commands. They do not own durable product state such as
messages, conversations, participants, attachments, drafts, outbox, search,
AI/workflow state or provider command envelopes.

## Decision

Макошь has one Communications domain.

Rules:

- `communications` owns account, channel, identity, conversation, participant,
  message, attachment, message version, tombstone, reaction, folder, draft,
  outbox, search, AI/workflow and provider command state.
- Mail, Telegram and WhatsApp are integration adapters.
- Provider/protocol/runtime code lives under `backend/src/integrations`.
- Frontend provider setup/runtime panels live under `frontend/src/integrations`.
- User-facing communication workspace lives at `/communications`.
- Public channel-scoped API routes use:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

- Public legacy route families are removed:

```text
/api/v1/<legacy-provider-root>/*
```

where `<legacy-provider-root>` was `email-accounts`, `telegram` or `whatsapp`.

- Mail, Telegram and WhatsApp must not reappear as backend or frontend product
  domains.
- Frontend route families `/telegram` and `/whatsapp` are removed.
- Provider-specific frontend query keys rooted directly at provider names are
  not domain cache keys. Provider-scoped communication cache keys use
  `['communications', provider, ...]`.
- Realtime domain events patch Communications caches. Integration runtime
  events may patch only integration-owned runtime panels when such panels exist.

## DTO And Naming Contract

Domain DTOs use provider-neutral names:

```text
CommunicationAccount
CommunicationChannel
CommunicationIdentity
CommunicationParticipant
CommunicationConversation
CommunicationMessage
CommunicationAttachment
CommunicationMessageVersion
CommunicationMessageTombstone
CommunicationReaction
CommunicationProviderCommand
```

Provider-specific DTOs such as `TelegramMessage`, `WhatsappMessage`,
`MailMessage` or `EmailMessage` are allowed only inside integration/runtime
modules and integration-scoped tests.

Historical database tables and compatibility Rust names may remain until
explicit migrations remove them. New runtime/domain code must not treat
provider-prefixed tables as the owning domain state.

## Data Contract

The canonical communication table family is:

```text
communication_accounts
communication_channels
communication_identities
communication_conversations
communication_conversation_participants
communication_messages
communication_attachments
communication_message_versions
communication_message_tombstones
communication_message_reactions
communication_message_refs
communication_folders
communication_drafts
communication_outbox
communication_provider_commands
communication_sync_runs
communication_sync_checkpoints
communication_raw_records
communication_raw_payloads
```

Historical provider-prefixed tables may remain for upgrade compatibility and
migration traceability. PostgreSQL stores metadata, provenance, observations,
projections, commands and workflow state. Secrets and session material remain in
the host vault or integration runtime storage according to ADR-0076.

## Consequences

Positive:

- Communications has one owner and one public workspace.
- Channels stop duplicating product-domain logic.
- Provider runtimes can evolve without changing the product domain boundary.
- Realtime, search, AI/workflow state, outbox and provider commands gain a
  shared communication model.

Negative:

- Existing Telegram and mail-heavy code requires large mechanical moves.
- Historical docs and tests that used domain language need cleanup.
- Some provider-specific type names remain temporarily inside integration code
  and tests until the final compatibility cleanup.

## Validation

The repository must enforce:

- no backend mail domain directory;
- no frontend Telegram domain directory;
- no frontend WhatsApp domain directory;
- no public `/api/v1/<legacy-provider-root>/*` route families for
  `email-accounts`, `telegram` or `whatsapp`;
- channel-scoped communication API routes under `/api/v1/communications/*`;
- no user-facing frontend query keys rooted directly at Telegram or WhatsApp;
- canonical communication migration after `0148`.
````

### `docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md`
- Size bytes / Размер в байтах: `4366`
- Included characters / Включено символов: `4366`
- Truncated / Обрезано: `no`

````markdown
# ADR-0098 Provider-Neutral Communications API And Strict Boundaries

Status: Accepted
Date: 2026-06-21

Supersedes:

- ADR-0097 public route decision for channel-scoped
  `/api/v1/communications/{mail,telegram,whatsapp}/*` business routes.

Clarifies:

- ADR-0042 Provider Credential Secret References And Resolver Boundary
- ADR-0076 Host Vault
- ADR-0085 Communication Spine And Consistency / Contradiction Engine
- ADR-0095 Event-Driven Domain Communication And DLQ
- ADR-0097 Communications Channel Domains To Integrations

## Context

ADR-0097 correctly established that Mail, Telegram and WhatsApp are
integrations, not product domains. It still allowed provider-scoped
Communications business routes as an intermediate migration shape:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

That intermediate shape leaves provider identity in the product API and makes it
too easy for integration code, app handlers and frontend modules to keep owning
business message state. The target model is stricter: Communications owns the
business state; providers supply observations, runtime state and command
execution.

## Decision

Макошь business Communications APIs are provider-neutral.

Provider-neutral product routes use:

```text
/api/v1/communications/conversations
/api/v1/communications/messages
/api/v1/communications/messages/{message_id}/...
/api/v1/communications/media
/api/v1/communications/search/...
```

Provider runtime/setup routes use:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

Provider search under `/api/v1/integrations/{provider}/provider-search` is a
runtime/control trigger only. It returns command/status metadata and must not
return projected Communication message, media, conversation or topic items.
Normal user search uses provider-neutral Communications routes such as
`/api/v1/communications/search/messages` and
`/api/v1/communications/search/media`.

The old provider-scoped business route families are removed and are not kept as
compatibility aliases:

```text
/api/v1/integrations/mail/*
/api/v1/integrations/telegram/*
/api/v1/integrations/whatsapp/*
```

Boundary rules are strict:

- `backend/src/integrations/**` must not import `crate::domains::*`.
- `backend/src/domains/**` must not import `crate::vault::*`.
- `backend/src/platform/**` must not contain SQL ownership of business domain
  tables such as `communication_*`, `task_*`, `calendar_*`, `review_*` or
  `graph_*`; platform owns technical event, observation, settings, audit,
  secret and storage primitives only.
- `backend/src/workflows/**` coordinates through domain command/query ports,
  events and platform contracts; it must not import concrete stores, handlers or
  integration clients.
- `backend/src/app/**` handlers validate, authorize, audit and map responses;
  business orchestration lives in application/workflow services.
- `frontend/src/domains/**` and `frontend/src/integrations/**` must not import
  each other directly. Shared types/helpers live in `frontend/src/shared` or
  `frontend/src/platform`; composition lives in app-level modules.

Architecture checks must enforce these rules structurally. New baseline files,
hardcoded per-file allowlists and linter/guard weakening are forbidden.

## Consequences

Positive:

- Communications has one provider-neutral product API.
- Provider runtimes can evolve without leaking into business routes.
- Guards fail on real boundary leaks instead of hiding them as compatibility
  exceptions.

Negative:

- Existing provider-scoped frontend clients and backend route tests must move in
  the same implementation pass.
- Workflows and app handlers need explicit ports/application services before the
  stricter guards can pass.

## Validation

The repository must enforce:

- no backend `integrations -> domains` imports;
- no backend `domains -> vault` imports;
- no platform SQL against business domain tables;
- no workflow concrete store/handler/integration-client imports;
- no app handler store/runtime/workflow orchestration;
- no frontend `domains <-> integrations` imports;
- no provider-scoped Communications business routes;
- no provider-search business read routes under `/api/v1/integrations/*`;
- no guard baseline or hardcoded per-file allowlist for these rules.
````

### `docs/adr/ADR-0099-signal-hub-event-platform.md`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/adr/ADR-0099-signal-hub-event-platform.md`
- Size bytes / Размер в байтах: `5448`
- Included characters / Включено символов: `5448`
- Truncated / Обрезано: `no`

````markdown
# ADR-0099 Signal Hub Event Platform

Status: Accepted
Date: 2026-06-22

Clarifies:

- ADR-0001 Event Sourcing as System Spine
- ADR-0014 Canonical Event Envelope
- ADR-0018 Provider Adapter Boundary
- ADR-0034 Event Replay and Projection Cursors
- ADR-0095 Event-Driven Domain Communication and DLQ
- ADR-0097 Communications Channel Domains To Integrations
- ADR-0098 Provider-Neutral Communications API And Strict Boundaries

## Context

Макошь is growing from a Communications-centered local system into a Personal
Operating System for memory, context and decisions. Email, Telegram and WhatsApp
are only the first external sources. Future sources include GitHub, Browser
capture, RSS, Calendar providers, Filesystem, Home Assistant, voice input and
fixture sources.

The system needs a single place to answer:

```text
What sources exist?
What is connected?
What is enabled?
What is muted?
What is paused?
What is unhealthy?
What can be replayed?
What fixture mode is active?
```

Putting this state inside provider integrations would duplicate policy across
Mail, Telegram, WhatsApp and every future source. Putting it inside
Communications would incorrectly make all signals communication-shaped.

## Decision

Макошь introduces Signal Hub as a first-class system domain.

Signal Hub owns:

- source registry;
- source connections;
- source capabilities;
- source runtime state;
- source health;
- signal policies;
- source profiles;
- replay requests;
- system recovery fixtures;
- fixture source catalog metadata.

Signal Hub does not own:

- provider protocol code;
- provider secrets;
- raw private message bodies;
- Communications state;
- Radar state;
- Tasks, Personas, Documents, Calendar, Knowledge or Graph state.

All new external and synthetic signal sources enter Макошь through Signal Hub
control state and the Event Backbone.

## Event Platform Decision

Макошь designs the event platform from the start for:

- PostgreSQL append-only `event_log` as audit/recovery source of truth;
- NATS JetStream as durable production delivery and fan-out transport;
- in-memory EventBus for deterministic unit tests;
- Axum SSE for browser realtime updates;
- Protobuf + ConnectRPC for typed command/query API contracts.

Redis, Kafka, RabbitMQ and provider-runtime sidecars are not part of this
initial target.

## Canonical Flow

```text
External / Fixture Source
  -> Signal Hub
  -> EventEnvelope
  -> PostgreSQL event_log
  -> NATS JetStream subject
  -> Domain consumer
  -> Projection
  -> SSE UI update
```

Communication example:

```text
signal.telegram.message.observed
  -> communication.message.recorded
  -> radar.signal.detected
  -> review.item.promoted
  -> task.created / persona.identity_trace.recorded / document.import.requested
```

## Signal Controls

Signal Hub must support:

- enable;
- disable;
- global mute;
- selective mute;
- pause;
- resume;
- replay;
- health check;
- fixture mode;
- profile application.

These controls are not test hacks. They are product-level operations for a local
memory system that must be debuggable, recoverable and safe.

## Fixture And Recovery Decision

Signal Hub must provide a schema-agnostic system recovery fixture.

The fixture may contain:

- canonical source codes;
- capability codes;
- profile codes;
- category strings;
- non-secret defaults.

The fixture must not contain:

- UUID values;
- FK values;
- database row IDs;
- secret references;
- provider account IDs;
- graph IDs;
- communication IDs;
- task/document/person IDs.

The loader maps canonical fixture values into the current database schema and is
idempotent. It restores missing system records but does not overwrite user-owned
connections, secrets or runtime sessions.

## Testing Decision

Every real source must have a deterministic fixture source.

Domain and workflow tests must be able to run without live Telegram, WhatsApp,
Mail, GitHub, browser extension, Home Assistant or calendar provider access.

Core testing modes:

```text
Unit: InMemoryEventBus
Domain integration: PostgreSQL + fixture sources
Event transport: PostgreSQL + NATS JetStream test environment
E2E local: Signal Hub UI + SSE + fixtures
```

## Consequences

Positive:

- source control is centralized;
- provider integrations remain adapters;
- non-communication sources are not forced into Communications;
- testing can mute, pause, replay and fixture sources deterministically;
- event delivery is designed for NATS JetStream from the start;
- ConnectRPC contracts prevent REST DTO sprawl.

Negative:

- first implementation is larger than a provider-specific settings page;
- policies and profiles require careful UX;
- event naming must migrate from some provider-specific `integration.*` families
  to canonical `signal.*` families;
- NATS JetStream and ConnectRPC add implementation work immediately.

## Validation

The repository should eventually enforce:

- `domains/signal_hub` exists and owns source control state;
- provider integrations do not own source policy;
- Signal Hub does not mutate Communications or other domain tables;
- source controls emit audit events;
- recovery fixtures contain no IDs or references;
- fixture sources can drive Communications/Radar workflows;
- NATS JetStream transport exists behind EventBus/EventTransport abstractions;
- ConnectRPC contracts exist for Signal Hub command/query APIs;
- SSE updates Signal Hub projections;
- Redis is not introduced as an event substrate.
````
