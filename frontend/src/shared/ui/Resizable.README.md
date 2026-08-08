# Resizable

## Description
Native CSS resize container with Макошь surface styling.

## When to use
Use for local preview panes, inspectors and test surfaces that can be manually resized.

## When not to use
Do not use for complex persisted pane layout; that belongs to an owning app shell.

## Accessibility
Native resize affordances are browser-provided. Content inside remains responsible for its semantics.

## Keyboard
Resizable itself has no keyboard shortcut contract.

## Examples
`<Resizable axis="both">...</Resizable>`

## Anti Patterns
Do not persist dimensions in shared UI.
