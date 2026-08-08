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
