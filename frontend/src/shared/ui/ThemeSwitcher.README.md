# ThemeSwitcher

## Description
Controlled theme selector for Макошь UI theme family and light/dark mode.

## When to use
Use in Storybook, settings or shells that own theme state.

## When not to use
Do not persist theme settings from this component.

## Accessibility
Uses separate radiogroups for theme family and theme mode.

## Keyboard
Buttons use native focus and activation behavior.

## Examples
`<ThemeSwitcher v-model="theme" />`

## Anti Patterns
Do not mutate global document theme directly inside this component.
