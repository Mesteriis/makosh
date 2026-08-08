# Portal

## Description
Small Vue Teleport wrapper owned by Макошь UI Kit.

## When to use
Use when shared UI needs to render overlay content outside the local DOM subtree.

## When not to use
Do not use for ordinary layout composition.

## Accessibility
Portal only changes DOM placement. The teleported content owns semantics.

## Keyboard
Portal does not manage focus.

## Examples
`<Portal to="body">...</Portal>`

## Anti Patterns
Do not portal content to hide poor component boundaries.
