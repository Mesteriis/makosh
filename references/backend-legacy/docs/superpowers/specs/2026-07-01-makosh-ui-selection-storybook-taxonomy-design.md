# Макошь UI Selection Controls And Storybook Taxonomy Design

Date: 2026-07-01
Status: Draft for user review

## Goal

Rework the Макошь UI Storybook taxonomy so standard UI controls are easy to
find by component name before domain-specific surfaces appear. The first
implementation slice is the selection-controls pack: richer standard select
components for dense Макошь workflows.

The goal is not to add domain logic. These components remain shared UI
primitives and composites under `frontend/src/shared/ui`.

## Current Context

The current UI Kit already includes basic controls:

- `Select`
- `MultiSelect`
- `Combobox`
- `Autocomplete`
- `Tree`
- `TreeItem`

These are visible mostly through broad Storybook groups such as `Form` and
`Navigation`. That makes the catalog harder to scan because the user cannot open
`Макошь UI / General / Select` directly and compare select variants.

## Target Storybook Taxonomy

Storybook should move toward this hierarchy:

```text
Макошь UI
  General
    Button
    Button Group
    Icon Button
    Split Button
    Toggle Group

    Select
    Searchable Select
    Multi Select
    Searchable Multi Select
    Grouped Select
    Tree Select
    Cascader
    Async Select

    Input
    Textarea
    Search Input
    Token Input
    Tag Input

    Checkbox
    Radio
    Switch
    Slider

    Date Picker
    Date Range Picker
    Time Picker

    Menu
    Context Menu
    Command
    Tabs
    Dialog
    Drawer
    Tooltip
    Popover

    Table
    List
    Tree
    Timeline

    Media
    Editor
    Feedback
    Layout
    Utility

  Domain
    Communications
    Review
    Knowledge
    Personas
    Organizations
    Projects
    Documents
    Tasks
    Calendar
    Agents
    Settings

  Foundation
    Tokens
    Themes
    Typography
    Icons
    Spacing
```

The first slice should create `Макошь UI / General / ...` stories for selection
components. Existing broad stories can remain during migration, but new control
work should use the new hierarchy.

## Selection Pack V1

Implement or promote these components:

| Component | Purpose |
|---|---|
| `Select` | Existing single-select baseline with clear direct Storybook entry. |
| `SearchableSelect` | Single-select with search, clear, keyboard navigation and empty state. |
| `MultiSelect` | Existing multi-select baseline, eventually replacing native multiple select UX. |
| `SearchableMultiSelect` | Multi-select with search, chips, select-all, clear-all and empty state. |
| `GroupedSelect` | Single-select grouped by section without hierarchical nesting. |
| `TreeSelect` | Hierarchical selection for nested local option trees. |
| `Cascader` | Column-based hierarchical selection for deep trees where expanded tree UI is too noisy. |
| `AsyncSelect` | UI state wrapper for externally loaded options: loading, error, retry, empty and disabled. |

## Component Boundaries

All components must stay provider-neutral and domain-neutral:

- no API calls;
- no TanStack Query;
- no Pinia stores;
- no domain imports;
- no provider-specific vocabulary;
- no persistence;
- no business validation.

The parent owns data loading and domain meaning. Shared UI components only render
state and emit user intent.

## Shared Option Model

Use a small shared option shape for selection components:

```ts
interface SelectOption {
  value: string
  label: string
  description?: string
  disabled?: boolean
  icon?: string
  tone?: 'default' | 'muted' | 'warning' | 'danger' | 'success'
}

interface SelectGroup {
  id: string
  label: string
  options: SelectOption[]
}

interface TreeSelectOption extends SelectOption {
  children?: TreeSelectOption[]
}
```

This keeps standard controls consistent without introducing domain entities into
the UI Kit.

## Common Props And Events

Selection components should converge on predictable props:

```ts
modelValue
options
placeholder
ariaLabel
disabled
readonly
loading
error
emptyLabel
clearable
searchable
maxVisibleOptions
```

Common events:

```ts
update:modelValue
search
select
clear
open
close
retry
```

`AsyncSelect` should not fetch by itself. It receives `loading`, `error`,
`options` and emits `search` / `retry`.

## Interaction Requirements

Selection controls must support:

- keyboard navigation;
- Escape to close;
- Enter to select;
- Home / End where list navigation is active;
- disabled options;
- clear action where `clearable` is enabled;
- visible empty state;
- visible loading state for async controls;
- error state with retry affordance for async controls;
- stable layout across `ru`, `en`, `es`;
- light, dark and Макошь themes.

## Storybook Requirements

Each component gets its own Storybook entry under `Макошь UI / General`.

Minimum stories:

- default state;
- disabled state;
- empty state;
- long labels;
- keyboard-friendly dense state;
- dark / Макошь theme coverage through globals;
- localized text from `frontend/stories/ui/storybook-i18n.ts`.

Async stories must demonstrate loading, error, empty and loaded states without
real network calls.

## Testing Requirements

For the first implementation slice:

- add or update boundary tests for public exports and no-domain-import rules;
- add focused component tests where interaction logic is non-trivial;
- run `pnpm typecheck`;
- run targeted unit tests;
- run Storybook build;
- run Storybook test runner;
- update and run visual snapshots when Storybook output changes.

## Migration Plan

1. Add shared selection types if needed.
2. Keep existing `Select`, `MultiSelect`, `Combobox` and `Autocomplete`
   compatible.
3. Add new selection components as separate files.
4. Add `Макошь UI / General / ...` stories for the selection pack.
5. Move selection examples out of broad `Form` only when the new entries are
   stable; until then broad stories may link to the new component stories.
6. Update docs inventory after the Storybook hierarchy exists.

## Out Of Scope

- domain-specific entity pickers;
- backend-backed search;
- API clients;
- provider-specific recipients or folder pickers;
- graph/entity relationship selection;
- committing implementation work without explicit user approval.

Those can be built later on top of the standard selection controls.

## Implementation Decision

`SearchableMultiSelect` should be introduced as a separate component first.
The existing native `MultiSelect` remains as a simple baseline until the richer
component is stable in Storybook, tested and visually reviewed. This avoids
breaking current consumers while still moving the UI Kit toward the target UX.
