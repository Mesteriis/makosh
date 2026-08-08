# Макошь Frontend UI System

## Goal

Макошь UI должен быть чистым, предсказуемым и спокойным. Это рабочий инструмент для памяти, коммуникаций, контекста и решений, а не демонстрация того, как дизайнер нашёл blur и больше не смог остановиться.

Главная цель UI:

```text
помочь пользователю быстро попасть в нужный контекст,
понять состояние,
совершить действие,
не потерять источник и память.
```

## Stack

```text
Vue 3
Tailwind CSS
Reka UI
shadcn-vue style ownership
Макошь UI Kit
Storybook
```

Важно: shadcn-vue здесь означает архитектурный подход, а не зависимость, которую домены импортируют напрямую.

## Layering

```text
frontend/src/shared/ui
  базовые локальные UI-компоненты

frontend/src/shared/ui/styles
  tokens, themes, component classes

frontend/stories/ui
  Storybook stories для изолированного допила компонентов

frontend/src/domains/*
  доменные компоненты, которые используют только shared/ui + query/store
```

## Public import rule

Домены импортируют UI только так:

```ts
import { Button, Dialog, DropdownMenu } from '@/shared/ui'
```

Домены не знают про `reka-ui`, `shadcn-vue`, `storybook`, `floating-ui`, Ant, PrimeVue или любую другую библиотеку. Иначе через полгода компонент `MessageBubble.vue` внезапно начнёт спорить с dropdown-движком, потому что людям зачем-то нужен хаос.

## UI component categories

### Primitive

Низкоуровневый компонент без доменного смысла:

```text
Button
IconButton
Input
Textarea
Select
Switch
Badge
Avatar
Tooltip
Popover
Dialog
Sheet
DropdownMenu
Tabs
Card
```

### Composite

Собранный UX-блок без доменной бизнес-логики:

```text
Command
ThemeProvider
Kbd
```

### Domain component

Компонент конкретного домена:

```text
ConversationItem
RadarItemCard
TaskCandidateRow
PersonaIdentityBlock
```

Он может использовать `shared/ui`, но не может импортировать Reka primitives напрямую.

## No views in UI Kit patch

Этот patch намеренно не добавляет view-экраны. Он добавляет только:

- foundation tokens;
- themes;
- shared UI components;
- Storybook stories.

Новые screens должны появляться после утверждения layout-модели Макошь.

## Themes

UI Kit поддерживает три темы:

| Theme | Purpose |
|---|---|
| `light` | Основная чистая корпоративная тема |
| `dark` | Нейтральная тёмная тема |
| `makosh` | Фирменная emerald-тема Макошь |

Переключение:

```vue
<ThemeProvider theme="light">
  <Button>Action</Button>
</ThemeProvider>
```

или на уровне root:

```html
<body data-ui-theme="light"></body>
```

## Storybook

Storybook используется для разработки UI Kit без запуска backend/Tauri runtime.
Он является UI lab для shared primitives, а не декоративной витриной.

Команды:

```bash
cd frontend
pnpm storybook
pnpm storybook:build
pnpm storybook:test
```

Storybook stories живут в:

```text
frontend/stories/ui
```

Включённый набор:

- docs/autodocs, controls и source через Storybook 10 docs surface;
- `@storybook/addon-a11y` для WCAG-проверок в панели;
- `@storybook/addon-themes` для переключения `light`, `dark`, `makosh`;
- `@storybook/addon-vitest` и `@storybook/test-runner` для story-driven tests;
- `@storybook/addon-coverage` с Vite 8-compatible `vite-plugin-istanbul` override;
- `msw-storybook-addon` и `public/mockServiceWorker.js` для API-mocking в stories;
- `storybook-addon-pseudo-states` для hover/focus/active states;
- `storybook-design-token`, который читает `@tokens` sections из `frontend/src/shared/ui/styles`;
- `@storybook/addon-designs` для Figma/design references, когда появится реальный design URL.

`frontend/public/design-tokens.source.json` является generated output плагина
design tokens и не хранится как source truth.

### Storybook i18n

Storybook имеет toolbar locale с `ru`, `en`, `es`. Тексты stories живут в:

```text
frontend/stories/ui/storybook-i18n.ts
```

Default locale - `ru`, потому что Макошь сейчас проектируется русскоязычным
owner-first интерфейсом. Все shared UI stories должны брать display text из
этого файла, чтобы visual baselines ловили переполнения и layout regressions
для трёх языков.

## Visual regression

Storybook является источником визуальных baseline для UI Kit. Playwright
читает `index.json`, открывает каждую story в `iframe.html` и сравнивает
скриншоты по локалям `ru`, `en`, `es`, темам `light`, `dark`, `makosh` и
ключевым ширинам интерфейса.

Команды:

```bash
cd frontend
pnpm test:visual
pnpm test:visual:update
```

Legacy CI, вызывавший этот gate через Makefile, перенесён в reference и больше
не является активной проверкой. До появления clean-room CI visual validation
запускается только прямыми командами `frontend/package.json`, приведёнными
выше. Обновление baseline остаётся явным локальным действием ревьюера.

## Component rule

Компонент должен быть:

- маленьким;
- стилевым, а не бизнесовым;
- без API calls;
- без TanStack Query;
- без Pinia store;
- без доменной валидации;
- без provider-specific логики.

UI Kit показывает и даёт interaction primitives. Он не думает. Думать у нас уже есть кому, к сожалению, и это не dropdown.
