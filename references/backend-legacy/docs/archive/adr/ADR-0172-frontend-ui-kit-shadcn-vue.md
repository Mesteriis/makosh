# ADR-0172: Макошь UI Kit на базе shadcn-vue / Reka UI

Status: Accepted

## Context

Макошь frontend должен быть переписан как самостоятельный UI-слой без бизнес-логики во Vue-компонентах. Доменный UI не должен напрямую зависеть от конкретной внешней компонентной библиотеки.

Нужна база, которая даёт:

- предсказуемый чистый корпоративный интерфейс;
- полноценные overlay primitives: dialog, dropdown, popover, tooltip, sheet, command;
- поддержку light, dark и Макошь signature темы;
- возможность дорабатывать компоненты в Storybook без запуска backend/Tauri runtime;
- локальное владение компонентами, а не vendor lock.

## Decision

Использовать подход shadcn-vue:

```text
Reka UI behavior primitives
↓
Макошь UI Kit local components
↓
shared/ui public API
↓
domain components
```

shadcn-vue не подключается как runtime component dependency. Компоненты живут в репозитории и считаются кодом Макошь.

## Rules

### Разрешено

```ts
import { Button } from '@/shared/ui'
import { Dialog } from '@/shared/ui'
```

### Запрещено в доменных компонентах

```ts
import { DialogRoot } from 'reka-ui'
import Button from 'primevue/button'
import { Button } from 'ant-design-vue'
```

Reka UI допускается только внутри `frontend/src/shared/ui/**`.

## Theme contract

UI Kit использует CSS custom properties с префиксом `--h-*`.

Обязательные темы:

- `light` — основная рабочая тема;
- `dark` — нейтральная тёмная тема;
- `makosh` — фирменный тёмный emerald-интерфейс.

Тема назначается через:

```html
<div data-ui-theme="light"></div>
<div data-ui-theme="dark"></div>
<div data-ui-theme="makosh"></div>
```

## Consequences

Плюсы:

- Макошь владеет компонентами;
- домены не зависят от конкретного vendor API;
- Storybook становится рабочей средой для UI Kit;
- можно менять визуальный язык без переписывания доменных компонентов.

Минусы:

- часть компонентов придётся дорабатывать вручную;
- нельзя бездумно импортировать новые primitives в домены;
- UI Kit становится отдельным продуктовым слоем, который надо поддерживать.
