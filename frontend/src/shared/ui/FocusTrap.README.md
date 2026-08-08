# FocusTrap

## Description
Макошь wrapper around Reka FocusScope.

## When to use
Use for custom overlay primitives that need contained keyboard focus.

## When not to use
Do not wrap content already managed by Dialog, AlertDialog or Drawer.

## Accessibility
Supports trapped and looped focus through Reka FocusScope.

## Keyboard
Tab and Shift+Tab loop when `loop` is enabled. Focus cannot escape when `trapped` is enabled.

## Examples
`<FocusTrap><button>First</button><button>Last</button></FocusTrap>`

## Anti Patterns
Do not trap focus in non-modal page regions.
