# Макошь UI Tokens

## Token layers

Макошь UI использует два слоя токенов.

### Legacy compatibility tokens

Существующие токены:

```text
--hh-*
```

Они сохраняются для текущих экранов и shell-логики.

### New UI Kit tokens

Новые токены:

```text
--h-*
```

Они используются компонентами `shared/ui`.

## Core tokens

### Color

```text
--h-color-bg
--h-color-bg-muted
--h-color-surface
--h-color-surface-raised
--h-color-surface-overlay
--h-color-surface-hover
--h-color-surface-active
--h-color-border
--h-color-border-strong
--h-color-text
--h-color-text-strong
--h-color-text-muted
--h-color-text-soft
--h-color-accent
--h-color-accent-strong
--h-color-accent-muted
--h-color-accent-contrast
--h-color-danger
--h-color-danger-muted
--h-color-success
--h-color-success-muted
--h-color-warning
--h-color-warning-muted
--h-color-info
--h-color-info-muted
```

### Radius

```text
--h-radius-xs
--h-radius-sm
--h-radius-md
--h-radius-lg
--h-radius-xl
--h-radius-pill
```

### Spacing

```text
--h-space-1
--h-space-2
--h-space-3
--h-space-4
--h-space-5
--h-space-6
```

### Controls

```text
--h-control-height-sm
--h-control-height-md
--h-control-height-lg
--h-control-padding-x
```

### Motion

```text
--h-motion-fast
--h-motion-default
```

### Elevation

```text
--h-shadow-xs
--h-shadow-sm
--h-shadow-md
--h-shadow-lg
```

## Theme mapping

Themes live in:

```text
frontend/src/shared/ui/styles/themes.css
```

Base tokens live in:

```text
frontend/src/shared/ui/styles/tokens.css
```

Component styles consume only `--h-*` tokens.

## Design direction

### Light

Main production theme.

```text
white surfaces
soft grey borders
blue corporate accent
minimal shadows
high readability
```

### Dark

Neutral dark theme.

```text
near-black background
blue accent
low contrast borders
no neon glow
```

### Макошь

Signature theme.

```text
very dark background
emerald accent
context/intelligence feeling
used carefully, not as visual confetti
```

## Rule

Never hardcode component colors in Vue files.

Allowed:

```css
background: var(--h-color-surface);
```

Forbidden:

```vue
<div style="background: #fff"></div>
```

Inline style attributes are already blocked by the frontend style guard. Humanity briefly achieved progress.
