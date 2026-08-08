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
  personas.ts       — loadPersonas, loadIdentityCandidates, setIdentityReview
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
