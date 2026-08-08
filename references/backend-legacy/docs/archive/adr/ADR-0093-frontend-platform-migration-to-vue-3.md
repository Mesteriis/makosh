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

domains/personas/views/
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
