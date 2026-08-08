# Surface Primitives Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the approved Макошь surface primitives pack and expose it in Storybook under `Макошь UI/General/Surface` with Russian, English, and Spanish story copy.

**Architecture:** All components stay in `frontend/src/shared/ui` as presentation-only shared UI primitives. Styles live in `frontend/src/shared/ui/styles/surfaces.css` and are imported by the shared UI stylesheet index. Storybook examples live in `frontend/stories/ui` and use the existing Storybook locale globals.

**Tech Stack:** Vue 3 SFCs, TypeScript, Vite, Storybook Vue 3, Vitest boundary tests, Playwright visual tests, pnpm.

---

## Source Context

Approved design spec:

- `docs/superpowers/specs/2026-07-02-surface-primitives-design.md`

Existing implementation to preserve:

- `frontend/src/shared/ui/Surface.vue`
- `frontend/src/shared/ui/Paper.vue`
- `frontend/src/shared/ui/Panel.vue`
- `frontend/src/shared/ui/Card.vue`
- `frontend/src/shared/ui/CardHeader.vue`
- `frontend/src/shared/ui/CardTitle.vue`
- `frontend/src/shared/ui/CardDescription.vue`
- `frontend/src/shared/ui/CardContent.vue`
- `frontend/src/shared/ui/CardFooter.vue`
- `frontend/src/shared/ui/Divider.vue`
- `frontend/src/shared/ui/Popover.vue`
- `frontend/src/shared/ui/DropdownMenu.vue`
- `frontend/src/shared/ui/index.ts`
- `frontend/src/shared/ui/styles/index.css`
- `frontend/src/shared/ui/styles/primitives.css`
- `frontend/src/shared/ui/styles/data-display.css`
- `frontend/src/shared/ui/primitives.boundary.test.ts`
- `frontend/src/shared/ui/storybookCoverage.boundary.test.ts`
- `frontend/stories/ui/general-story-copy.ts`
- `frontend/stories/ui/storybook-i18n.ts`

The working tree is already dirty. Do not revert unrelated changes. Keep implementation scoped to files listed in this plan unless a validation failure proves another directly related file must be changed.

## File Structure

Create:

- `frontend/src/shared/ui/Section.vue`
- `frontend/src/shared/ui/Section.README.md`
- `frontend/src/shared/ui/Accordion.vue`
- `frontend/src/shared/ui/Accordion.README.md`
- `frontend/src/shared/ui/Accordion.types.ts`
- `frontend/src/shared/ui/Callout.vue`
- `frontend/src/shared/ui/Callout.README.md`
- `frontend/src/shared/ui/Well.vue`
- `frontend/src/shared/ui/Well.README.md`
- `frontend/src/shared/ui/Fieldset.vue`
- `frontend/src/shared/ui/Fieldset.README.md`
- `frontend/src/shared/ui/ToolbarSection.vue`
- `frontend/src/shared/ui/ToolbarSection.README.md`
- `frontend/src/shared/ui/StatCard.vue`
- `frontend/src/shared/ui/StatCard.README.md`
- `frontend/src/shared/ui/ActionCard.vue`
- `frontend/src/shared/ui/ActionCard.README.md`
- `frontend/src/shared/ui/surface.boundary.test.ts`
- `frontend/src/shared/ui/styles/surfaces.css`
- `frontend/stories/ui/GeneralSurface.stories.ts`

Update:

- `frontend/src/shared/ui/Surface.vue`
- `frontend/src/shared/ui/Surface.README.md`
- `frontend/src/shared/ui/Paper.vue`
- `frontend/src/shared/ui/Paper.README.md`
- `frontend/src/shared/ui/Panel.vue`
- `frontend/src/shared/ui/Panel.README.md`
- `frontend/src/shared/ui/Card.vue`
- `frontend/src/shared/ui/Card.README.md`
- `frontend/src/shared/ui/index.ts`
- `frontend/src/shared/ui/styles/index.css`
- `frontend/src/shared/ui/styles/primitives.css`
- `frontend/src/shared/ui/styles/data-display.css`
- `frontend/src/shared/ui/storybookCoverage.boundary.test.ts`
- `frontend/stories/ui/general-story-copy.ts`

Do not update generated files manually.

## Component Contracts

### Surface

Use `Surface` as the base generic wrapper.

Props:

- `as`: string, default `section`
- `tone`: `default | muted | raised | deep`, default `default`
- `padding`: `none | sm | md | lg`, default `md`
- `radius`: `none | sm | md | lg`, default `md`
- `bordered`: boolean, default `true`
- `clip`: boolean, default `false`
- `class`: string

Class contract:

- Always include `makosh-surface`
- Include modifier classes for tone, padding, radius, bordered, and clip
- Do not emit `hh-surface`

### Paper and Panel

Align `Paper` and `Panel` with the same surface props:

- `as`
- `tone`
- `padding`
- `radius`
- `bordered`
- `clip`
- `class`

`Paper` remains document-like and slightly more elevated by CSS defaults. `Panel` remains the lightweight grouped UI surface.

### Card

Props:

- `as`: string, default `article`
- `variant`: `default | muted | raised | interactive`, default `default`
- `density`: `compact | comfortable`, default `comfortable`
- `selected`: boolean, default `false`
- `disabled`: boolean, default `false`
- `clip`: boolean, default `false`
- `class`: string

Class contract:

- Always include `makosh-card`
- Include `makosh-card--interactive` only through `variant="interactive"`
- Include `makosh-card--clip` only when `clip` is true
- Existing `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, and `CardFooter` remain supported

### Section

Generic page or panel section. It is not a card.

Props:

- `as`: string, default `section`
- `tone`: `plain | muted | bordered`, default `plain`
- `padding`: `none | sm | md | lg`, default `md`
- `class`: string

Slots:

- `header`
- `actions`
- default
- `footer`

### Accordion

Generic disclosure stack.

Create `Accordion.types.ts` with:

```ts
export interface AccordionItem {
	id: string
	title: string
	description?: string
	disabled?: boolean
}
```

Props:

- `items`: `AccordionItem[]`
- `modelValue`: `string[]`, default empty array
- `multiple`: boolean, default `false`
- `collapsible`: boolean, default `true`
- `class`: string

Emits:

- `update:modelValue`

Slots:

- `item`, scoped with the current item

Behavior:

- Single mode opens one item at a time
- Multiple mode allows several open ids
- Disabled items do not toggle
- Collapsible false keeps one item open when possible

### Callout

Informational surface with tone and optional icon.

Props:

- `tone`: `neutral | info | success | warning | danger`, default `neutral`
- `icon`: string
- `class`: string

Slots:

- `title`
- default
- `actions`

### Well

Inset low-emphasis surface for secondary content.

Props:

- `as`: string, default `div`
- `tone`: `default | muted | inset`, default `default`
- `padding`: `sm | md | lg`, default `md`
- `class`: string

### Fieldset

Semantic grouping for form/control clusters.

Props:

- `disabled`: boolean, default `false`
- `class`: string

Slots:

- `legend`
- `description`
- default

### ToolbarSection

Labelled section inside toolbars and dense control rows.

Props:

- `orientation`: `horizontal | vertical`, default `horizontal`
- `class`: string

Slots:

- `label`
- default

### StatCard

Generic metric card with no domain fields.

Props:

- `label`: string
- `value`: string or number
- `description`: string
- `trend`: string
- `tone`: `neutral | accent | success | warning | danger`, default `neutral`
- `icon`: string
- `class`: string

### ActionCard

Interactive generic card for local actions.

Props:

- `as`: string, default `button`
- `title`: string
- `description`: string
- `icon`: string
- `selected`: boolean, default `false`
- `disabled`: boolean, default `false`
- `class`: string

Emits:

- `click`

Implementation note:

- For `as="button"`, pass `type="button"` and `disabled`
- For non-button tags, apply `aria-disabled` when disabled and suppress click emission

## CSS Contract

Move surface/card styling out of `data-display.css` into `surfaces.css`.

`surfaces.css` must define:

- `.makosh-surface`
- `.makosh-paper`
- `.makosh-panel`
- `.makosh-card`
- `.makosh-card-header`
- `.makosh-card-title`
- `.makosh-card-description`
- `.makosh-card-content`
- `.makosh-card-footer`
- `.makosh-section`
- `.makosh-accordion`
- `.makosh-callout`
- `.makosh-well`
- `.makosh-fieldset`
- `.makosh-toolbar-section`
- `.makosh-stat-card`
- `.makosh-action-card`

Overflow rule:

- Default surface classes must not set `overflow: hidden`
- Only explicit `--clip` classes may set `overflow: hidden`
- Popover and menu portal content must remain above cards and sections through existing overlay z-index rules

Token rule:

- Use existing `--h-*` tokens
- Do not introduce a one-off color palette
- Keep radius at `var(--h-radius-md)` or below for default card-like elements unless existing tokens require another value

## Storybook Contract

Create `frontend/stories/ui/GeneralSurface.stories.ts`.

Meta:

```ts
const meta = {
	title: 'Макошь UI/General/Surface',
	component: Surface
} satisfies Meta<typeof Surface>
```

Stories:

- `Overview`
- `Cards`
- `Sections`
- `Accordion`
- `CalloutsAndWells`
- `FieldsetAndToolbar`
- `OverlaySafety`

Story requirements:

- Import components from `@/shared/ui`
- Import locale helpers from `./storybook-i18n`
- Use copy from `general-story-copy.ts`
- Show all new components at least once
- Show `Surface`, `Paper`, `Panel`, `Card`, and card subcomponents
- Keep examples generic and UI-only
- Use Russian, English, and Spanish copy through the existing Storybook locale global
- `OverlaySafety` must render an open `Popover` inside a card or section so clipping issues are visible in Storybook

Update `general-story-copy.ts`:

- Add `controls.surface`
- Add a `surfaces` copy object with labels for all seven stories
- Add matching translations for `en`, `ru`, and `es`

Update `storybookCoverage.boundary.test.ts`:

- Add `Макошь UI/General/Surface` to `requiredGeneralTitles`
- Keep the existing requirement that every story imports `./storybook-i18n`

## Implementation Tasks

- [ ] Task 1: Add failing surface boundary coverage
  - Create `frontend/src/shared/ui/surface.boundary.test.ts`
  - Assert that every surface pack component has a `.vue` file, `.README.md`, and barrel export
  - Include existing components: `Surface`, `Paper`, `Panel`, `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter`, `Divider`
  - Include new components: `Section`, `Accordion`, `Callout`, `Well`, `Fieldset`, `ToolbarSection`, `StatCard`, `ActionCard`
  - Assert all surface pack component source files do not import `@/domains`, `@/integrations`, `@/platform`, stores, routers, or network APIs
  - Assert `GeneralSurface.stories.ts` imports all surface pack components
  - Add CSS assertions that `surfaces.css` contains `--clip` classes and no default `.makosh-surface` or `.makosh-card` block sets `overflow: hidden`
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 2: Normalize existing surface primitives
  - Update `Surface.vue` to emit `makosh-surface` classes and the approved props
  - Update `Paper.vue` and `Panel.vue` to share the approved surface-like props and modifier classes
  - Update `Card.vue` to support `as`, `variant`, `density`, `selected`, `disabled`, and `clip`
  - Add `Surface.README.md`
  - Add `Card.README.md`
  - Update `Paper.README.md` and `Panel.README.md` with the overlay clipping rule
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/surface.boundary.test.ts src/shared/ui/storybookCoverage.boundary.test.ts
```

- [ ] Task 3: Move surface styles into `surfaces.css`
  - Create `frontend/src/shared/ui/styles/surfaces.css`
  - Move card and old surface styling from `data-display.css` into `surfaces.css`
  - Move shared `Panel` and `Paper` styling from `primitives.css` into `surfaces.css`
  - Update `frontend/src/shared/ui/styles/index.css` to import `surfaces.css` after `primitives.css`
  - Leave non-surface data display styles in `data-display.css`
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 4: Implement structural surfaces
  - Add `Section.vue` and `Section.README.md`
  - Add `Callout.vue` and `Callout.README.md`
  - Add `Well.vue` and `Well.README.md`
  - Add `Fieldset.vue` and `Fieldset.README.md`
  - Add `ToolbarSection.vue` and `ToolbarSection.README.md`
  - Use existing `Icon.vue` only where icons are required
  - Keep all component state local and presentation-only
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 5: Implement interactive surface primitives
  - Add `Accordion.types.ts`
  - Add `Accordion.vue` and `Accordion.README.md`
  - Add `StatCard.vue` and `StatCard.README.md`
  - Add `ActionCard.vue` and `ActionCard.README.md`
  - Keep `Accordion` deterministic with controlled `modelValue`
  - Keep `ActionCard` disabled behavior explicit for button and non-button render targets
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 6: Export the surface pack
  - Update `frontend/src/shared/ui/index.ts`
  - Export all new components near existing surface/card exports
  - Export `AccordionItem` from `Accordion.types.ts`
  - Ensure `storybookCoverage.boundary.test.ts` still passes the all-components-exported check
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/storybookCoverage.boundary.test.ts src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 7: Add localized Storybook surface stories
  - Add `frontend/stories/ui/GeneralSurface.stories.ts`
  - Add the `surfaces` copy object to `frontend/stories/ui/general-story-copy.ts`
  - Add `controls.surface` in `en`, `ru`, and `es`
  - Add `Макошь UI/General/Surface` to `requiredGeneralTitles` in `storybookCoverage.boundary.test.ts`
  - The `OverlaySafety` story must render an open `Popover` inside a `Card` or `Section`
  - Run:

```sh
cd frontend && pnpm exec vitest run src/shared/ui/storybookCoverage.boundary.test.ts src/shared/ui/surface.boundary.test.ts
```

- [ ] Task 8: Run focused static validation
  - Run:

```sh
cd frontend && pnpm lint:ox src/shared/ui/Surface.vue src/shared/ui/Paper.vue src/shared/ui/Panel.vue src/shared/ui/Card.vue src/shared/ui/Section.vue src/shared/ui/Accordion.vue src/shared/ui/Callout.vue src/shared/ui/Well.vue src/shared/ui/Fieldset.vue src/shared/ui/ToolbarSection.vue src/shared/ui/StatCard.vue src/shared/ui/ActionCard.vue src/shared/ui/index.ts src/shared/ui/surface.boundary.test.ts src/shared/ui/storybookCoverage.boundary.test.ts stories/ui/GeneralSurface.stories.ts stories/ui/general-story-copy.ts
cd frontend && pnpm typecheck
```

- [ ] Task 9: Validate Storybook build and interaction smoke
  - Run:

```sh
cd frontend && pnpm storybook:build
cd frontend && MAKOSH_STORYBOOK_HOST=127.0.0.1 MAKOSH_STORYBOOK_PORT=6008 pnpm storybook:serve
cd frontend && pnpm exec test-storybook --url http://127.0.0.1:6008
```

  - Stop the Storybook static server when validation finishes
  - Open the in-app browser to:

```text
http://127.0.0.1:6008/?path=/story/makosh-ui-general-surface--overview&globals=theme:light;locale:ru
```

- [ ] Task 10: Update and compare visual baselines
  - Run after Storybook renders correctly:

```sh
cd frontend && MAKOSH_STORYBOOK_PORT=6007 pnpm test:visual:update
cd frontend && MAKOSH_STORYBOOK_PORT=6007 pnpm test:visual
```

  - Review generated snapshots for the new `makosh-ui-general-surface` stories
  - Confirm there is no text overlap at 320, 375, 768, 1024, 1440, 1920, and 5120 widths
  - Confirm the `OverlaySafety` popover is visible outside the surface bounds

## Review Checklist

- [ ] `Surface`, `Paper`, `Panel`, and `Card` have compatible prop naming
- [ ] `Section` is visibly not a card
- [ ] Default surfaces do not clip overlays
- [ ] `clip` is opt-in and visually testable
- [ ] Storybook hierarchy contains `Макошь UI/General/Surface`
- [ ] All stories use `ru`, `en`, and `es` copy through the locale global
- [ ] No domain, provider, store, router, or network imports are introduced
- [ ] README files explain intended use and misuse
- [ ] Boundary tests protect exports, docs, Storybook coverage, and UI-only ownership
- [ ] Visual snapshots cover the new story group

## Risks

- Visual snapshot updates can be large because the current visual suite screenshots every Storybook story across themes, locales, and widths.
- `Card` API expansion can affect existing card usage if class names are removed instead of extended.
- Existing dirty worktree contains many unrelated changes, so implementation must avoid cleanup outside the surface pack.
- Native and portal overlay behavior must be verified in Storybook; CSS review alone is not enough.

## Definition of Done

- The approved surface primitives exist and are exported from `@/shared/ui`
- Existing `Surface`, `Paper`, `Panel`, and `Card` are normalized without removing their intended roles
- `Макошь UI/General/Surface` appears in Storybook with the seven required stories
- Storybook examples are localized for `ru`, `en`, and `es`
- Overlay safety is visible in Storybook through an open popover inside a surface
- Boundary tests pass for exports, docs, Storybook coverage, and UI-only ownership
- Typecheck passes
- Storybook builds successfully
- Visual snapshots are updated and compare cleanly
