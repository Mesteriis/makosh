---
chunk_id: 140-other-frontend-part-013
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 24
generated_by: code-wiki-ru
---

# 140-other-frontend-part-013 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `24`

## Резюме

Создать страницу `components/frontend.md` в русской Obsidian‑wiki проекта `makosh`. Страница документирует все шаблонные Vue‑компоненты, находящиеся в `frontend/src/shared/ui/`, а также shell‑обёртки, глобальные дизайн‑токены, классы тем и стили поверхностей. Каждое утверждение о поведении компонента опирается исключительно на встроенный исходный код; внешние знания не добавлены.

## Предложенные страницы

### `components/frontend.md`

```markdown
# Шаблонные компоненты фронтенда (Макошь UI)

## Обзор

Фронтенд-часть проекта `makosh` использует общую библиотеку UI-компонентов,
построенных на примитивах `reka-ui` с собственной системой стилей Макошь.
Компоненты находятся в директории `frontend/src/shared/ui/`, глобальные стили —
в `frontend/src/style.css`, `frontend/src/styles/surfaces.css` и
`frontend/src/styles/theme-classes.css`.

Все компоненты используют CSS-переменные с префиксом `--hh-*` для цветов,
радиусов, отступов и теней. Значения переменных заданы в `style.css` и
переопределяются классами тем из `theme-classes.css`.

## UI-компоненты (`shared/ui`)

### Input (`Input.vue`)

Однострочное текстовое поле с поддержкой состояния ошибки.

**Пропсы:**

| Проп | Тип | По умолчанию | Описание |
|---|---|---|---|
| `modelValue` | `string?` | `''` | Значение поля (v-model) |
| `placeholder` | `string?` | `''` | Placeholder-текст |
| `disabled` | `boolean?` | `false` | Блокировка поля |
| `readonly` | `boolean?` | `false` | Только для чтения |
| `type` | `string?` | `'text'` | HTML-тип поля |
| `error` | `string?` | — | Текст ошибки |
| `class` | `string?` | — | Дополнительный CSS-класс |

**События:** `update:modelValue(value: string)`, `focus(event: FocusEvent)`, `blur(event: FocusEvent)`

**CSS-классы:** `makosh-input-wrapper`, `makosh-input`, `makosh-input--error`, `makosh-input-error`

**Состояния стилей:** hover (не в `disabled`/`readonly`) меняет цвет границы на
`--hh-border-accent`; focus добавляет `box-shadow` с `--hh-focus-ring`; disabled
уменьшает opacity до `0.5` и ставит `cursor: not-allowed`; error меняет цвет
границы и кольца фокуса на `--hh-color-danger`.

---

### Textarea (`Textarea.vue`)

Многострочное текстовое поле с поддержкой состояния ошибки.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `modelValue` | `string?` | `''` |
| `placeholder` | `string?` | `''` |
| `disabled` | `boolean?` | `false` |
| `rows` | `number?` | `3` |
| `error` | `string?` | — |
| `class` | `string?` | — |

**События:** `update:modelValue(value: string)`

**CSS-классы:** `makosh-textarea-wrapper`, `makosh-textarea`, `makosh-textarea--error`, `makosh-textarea-error`

Состояния hover/focus/disabled/error — аналогично `Input`. Свойство `resize: vertical`.

---

### Select (`Select.vue`)

Выпадающий список (single-select) на основе `reka-ui` (`SelectRoot`, `SelectTrigger`,
`SelectValue`, `SelectContent`, `SelectItem`, `SelectItemIndicator`,
`SelectViewport`, `SelectPortal`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `modelValue` | `string?` | `''` |
| `placeholder` | `string?` | `'Select…'` |
| `disabled` | `boolean?` | `false` |
| `error` | `string?` | — |
| `class` | `string?` | — |
| `options` | `Array<{ value: string; label: string }>?` | — |

**События:** `update:modelValue(value: string)`

**CSS-классы:** `makosh-select-wrapper`, `makosh-select-trigger`, `makosh-select-value`,
`makosh-select-chevron`, `makosh-select-content`, `makosh-select-viewport`,
`makosh-select-item`, `makosh-select-check`, `makosh-select-error`,
`makosh-select--error`

**Детали реализации:**
- Индикатор выбранного элемента — иконка `tabler:check` (цвет `--hh-accent`).
- Иконка-шеврон (`tabler:chevron-down`) поворачивается на 180° при открытии
  (атрибут `data-state="open"`).
- Выбранный элемент получает цвет `--hh-accent` через `[data-state="checked"]`.
- Подсветка при навигации — `[data-highlighted]` с фоном `--hh-hover-bg`.
- Ширина контента ограничена `var(--reka-select-trigger-width)`.
- Ошибка отображается аналогично `Input`.

---

### Switch (`Switch.vue`)

Переключатель (toggle) на основе `reka-ui` (`SwitchRoot`, `SwitchThumb`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `modelValue` | `boolean?` | `false` |
| `disabled` | `boolean?` | `false` |
| `class` | `string?` | — |

**События:** `update:modelValue(value: boolean)`

**CSS-классы:** `makosh-switch`, `makosh-switch--disabled`, `makosh-switch-thumb`

**Состояния:**
- `[data-state="checked"]` — фон `--hh-accent`, thumb сдвигается на `translateX(1rem)`.
- `disabled` — opacity `0.5`, `cursor: not-allowed`.
- `focus-visible` — outline `2px solid var(--hh-focus-ring)`.

---

### Label (`Label.vue`)

Текстовая метка для элементов формы.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `htmlFor` | `string?` | — |
| `class` | `string?` | — |

**Слот:** default (содержимое метки).

**CSS-класс:** `makosh-label` (font-size `0.8125rem`, font-weight `500`, цвет `--hh-text-primary`).

---

### Separator (`Separator.vue`)

Визуальный разделитель на основе `reka-ui` (`Separator`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` |
| `decorative` | `boolean?` | `true` |
| `class` | `string?` | — |

**CSS-классы:** `makosh-separator`, `makosh-separator--horizontal` (height `1px`, width `100%`),
`makosh-separator--vertical` (width `1px`, height `100%`). Фон — `--hh-border`.

`decorative: true` скрывает разделитель от accessibility tree.

---

### DropdownMenuSeparator (`DropdownMenuSeparator.vue`)

Разделитель для выпадающих меню на основе `reka-ui` (`DropdownMenuSeparator`).

Не имеет пропсов. Фиксированный CSS-класс: `makosh-dropdown-separator`
(height `1px`, background `--hh-border`, margin `0.25rem 0.5rem`).

---

### ScrollArea (`ScrollArea.vue`)

Область с кастомной полосой прокрутки на основе `reka-ui`
(`ScrollAreaRoot`, `ScrollAreaViewport`, `ScrollAreaScrollbar`, `ScrollAreaThumb`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `class` | `string?` | — |
| `maxHeight` | `string?` | — |

> **Примечание:** пропс `maxHeight` объявлен, но не используется ни в шаблоне, ни в стилях.

**Слот:** default (содержимое viewport'а).

**CSS-классы:** `makosh-scroll-area`, `makosh-scroll-viewport`, `makosh-scrollbar`, `makosh-scroll-thumb`

Обе оси прокрутки (вертикальная и горизонтальная) отображаются всегда.
Полоса прокрутки делается видимой при hover (фон `--hh-hover-bg`).
Thumb использует `background: var(--hh-border)` и `border-radius: var(--hh-radius-pill)`.

---

### Surface (`Surface.vue`)

Контейнер-поверхность с тонировкой фона.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `as` | `string?` | `'section'` |
| `tone` | `'panel' \| 'raised' \| 'deep'` | `'panel'` |

**Слот:** default.

**CSS-классы:** `hh-surface`, `hh-surface--panel`, `hh-surface--raised`, `hh-surface--deep`

Стили для этих классов определены **не в самом компоненте** (scoped-стили отсутствуют),
а в глобальном файле `surfaces.css`. Каждый тон задаёт свой фон через
`rgba(..., var(--hh-panel-alpha))`, общую рамку `--hh-border-subtle`,
радиус `--hh-radius-md` и backdrop-filter `blur(var(--hh-panel-blur))`.

---

### Skeleton (`Skeleton.vue`)

Плейсхолдер-заглушка для состояния загрузки.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `class` | `string?` | — |
| `width` | `string?` | `'100%'` |
| `height` | `string?` | `'1rem'` |
| `rounded` | `boolean?` | `false` |

> **Примечание:** пропсы `width` и `height` объявлены, но не применены в шаблоне
> или стилях — компонент рендерит `<div>` только с CSS-классами.

**CSS-классы:** `makosh-skeleton`, `makosh-skeleton--rounded`

Анимация: `makosh-skeleton-pulse` — пульсация opacity между `0.4` и `0.8` с периодом `1.5s`.

---

### Icon (`Icon.vue`)

Обёртка над компонентом `Icon` из `@iconify/vue`.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `icon` | `string` | *обязательный* |
| `size` | `number \| string?` | `20` |
| `class` | `string?` | — |

Атрибут `aria-hidden="true"` установлен всегда — иконка скрыта от accessibility tree.

---

### Popover (`Popover.vue`)

Всплывающая панель на основе `reka-ui` (`PopoverRoot`, `PopoverTrigger`,
`PopoverPortal`, `PopoverContent`, `PopoverArrow`, `PopoverClose`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `open` | `boolean?` | — |
| `side` | `'top' \| 'bottom' \| 'left' \| 'right'` | `'bottom'` |
| `sideOffset` | `number?` | `4` |
| `align` | `'start' \| 'center' \| 'end'` | `'center'` |
| `class` | `string?` | — |

**События:** `update:open(value: boolean)`

**Слоты:**
- `trigger` — элемент-триггер (рендерится через `PopoverTrigger` с `as-child`).
- `default` — содержимое popover'а.

**CSS-классы:** `makosh-popover-content`, `makosh-popover-arrow`, `makosh-popover-close`, `makosh-popover-close-btn`

**Детали:**
- Кнопка закрытия с иконкой `tabler:x` (размер `0.875rem`) позиционирована абсолютно
  в правом верхнем углу (`top: 0.5rem; right: 0.5rem`).
- Анимация появления: `popover-in` — opacity и translateY за `150ms`.
- Минимальная ширина контента: `200px`.

---

### Sheet (`Sheet.vue`)

Боковая/верхняя/нижняя панель (slide-in) на основе диалоговых примитивов `reka-ui`
(`DialogRoot`, `DialogTrigger`, `DialogPortal`, `DialogOverlay`, `DialogContent`,
`DialogTitle`, `DialogDescription`, `DialogClose`).

> **Примечание:** reka-ui не предоставляет отдельного компонента Sheet;
> компонент построен на `Dialog*` примитивах с кастомной анимацией и позиционированием.

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `open` | `boolean?` | `false` |
| `title` | `string?` | — |
| `description` | `string?` | — |
| `side` | `'left' \| 'right' \| 'top' \| 'bottom'` | `'right'` |
| `class` | `string?` | — |
| `contentClass` | `string?` | — |

**События:** `update:open(value: boolean)`

**Слоты:**
- `trigger` — элемент-триггер.
- `header` — дополнительное содержимое заголовка (после title/description, если заданы).
- `default` — тело панели.
- `footer` — футер (отображается только если слот передан; имеет верхнюю границу).

**CSS-классы:** `makosh-sheet-overlay`, `makosh-sheet-content`, `makosh-sheet--left`,
`makosh-sheet--right`, `makosh-sheet--top`, `makosh-sheet--bottom`,
`makosh-sheet-header`, `makosh-sheet-body`, `makosh-sheet-footer`,
`makosh-sheet-title`, `makosh-sheet-description`, `makosh-sheet-close`,
`makosh-sheet-close-btn`

**Детали:**
- Ширина контента: `90vw`, максимум `400px`.
- Оверлей: фиксированный, `rgba(0, 0, 0, 0.6)`, анимация `sheet-overlay-in` за `200ms`.
- Анимации слайда: `sheet-slide-right`, `sheet-slide-left`, `sheet-slide-top`, `sheet-slide-bottom` за `250ms`.
- Кнопка закрытия с иконкой `tabler:x` (размер `1.125rem`).

---

### Tooltip (`Tooltip.vue`)

Всплывающая подсказка на основе `reka-ui` (`TooltipRoot`, `TooltipTrigger`,
`TooltipPortal`, `TooltipContent`, `TooltipArrow`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `content` | `string?` | — |
| `side` | `'top' \| 'bottom' \| 'left' \| 'right'` | `'top'` |
| `sideOffset` | `number?` | `4` |
| `delayDuration` | `number?` | `400` |
| `class` | `string?` | — |

**Слоты:**
- `trigger` — элемент-триггер.
- `default` — содержимое подсказки (если передан, заменяет пропс `content`).

**CSS-классы:** `makosh-tooltip-content`, `makosh-tooltip-arrow`

**Детали:**
- Задержка перед показом: `400ms` (настраивается через `delayDuration`).
- Анимация появления: `tooltip-in` — scale и opacity за `150ms`.
- Фон: `var(--hh-surface-deep, #041215)`, рамка `--hh-border`.
- Стрелка (`TooltipArrow`) заполняется цветом `--hh-surface-deep`.

---

### Toast (`Toast.vue`)

Система toast-уведомлений на основе `reka-ui` (`ToastProvider`, `ToastViewport`,
`ToastRoot`, `ToastTitle`, `ToastDescription`, `ToastClose`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `swipeDirection` | `'right' \| 'left' \| 'up' \| 'down'` | `'right'` |
| `duration` | `number?` | `4000` |
| `class` | `string?` | — |

**Интерфейс `ToastItem` (экспортируется):**

```ts
interface ToastItem {
  id: string
  title?: string
  description?: string
  variant?: 'default' | 'success' | 'warning' | 'error'
  duration?: number
}
```

**Внедряемый контекст (provide):**

Компонент предоставляет через `provide` объект с методами:
- `addToast(item: Omit<ToastItem, 'id'>): string` — добавляет toast, возвращает id.
- `removeToast(id: string): void` — удаляет toast по id.
- `success(title: string, description?: string): string`
- `warning(title: string, description?: string): string`
- `error(title: string, description?: string): string`

Ключ внедрения: `'makosh-toast-context'`.

**Слот:** default (основное содержимое страницы, в которое встраивается `ToastViewport`).

**CSS-классы:** `makosh-toast-viewport`, `makosh-toast-root`, `makosh-toast-inner`,
`makosh-toast-variant-icon`, `makosh-toast-content`, `makosh-toast-title`,
`makosh-toast-description`, `makosh-toast-close-btn`, `makosh-toast--success`,
`makosh-toast--warning`, `makosh-toast--error`

**Детали:**
- Viewport позиционирован фиксированно: `bottom: 1rem; right: 1rem`, max-width `360px`, z-index `200`.
- Иконки вариантов: `success` → `tabler:check-circle`, `warning` → `tabler:alert-triangle`,
  `error` → `tabler:alert-circle`. Вариант `default` — без иконки.
- Цвета границы вариантов используют `color-mix(in srgb, ...)` с 30% непрозрачности
  от соответствующего статусного цвета.
- Анимация появления: `toast-slide-in` (translateX справа, opacity); закрытия:
  `toast-slide-out`. Удаление происходит при `update:open === false`.
- Кнопка закрытия с иконкой `tabler:x`.

---

### Progress (`Progress.vue`)

Индикатор прогресса на основе `reka-ui` (`ProgressRoot`, `ProgressIndicator`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `modelValue` | `number?` | `0` |
| `max` | `number?` | `100` |
| `indeterminate` | `boolean?` | `false` |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` |
| `class` | `string?` | — |

**События:** `update:modelValue(value: number)`

**CSS-классы:** `makosh-progress-root`, `makosh-progress--sm`, `makosh-progress--md`,
`makosh-progress--lg`, `makosh-progress--indeterminate`, `makosh-progress-indicator`

**Детали:**
- Размеры: `sm` — height `0.25rem`, `md` — `0.5rem`, `lg` — `0.75rem`.
- В обычном режиме `watchEffect` вычисляет `translateX(-${100 - percentage}%)` для
  индикатора. Анимация перехода: `300ms cubic-bezier(0.16, 1, 0.3, 1)`.
- В режиме `indeterminate` ширина индикатора фиксирована (`40%`), применяется
  анимация `progress-indeterminate` (бесконечное движение `translateX` от `-100%`
  до `350%` за `1.5s`). Индикатор использует линейный градиент
  `transparent → var(--hh-accent) → transparent`.

---

### Tabs, TabTrigger, TabContent

Набор компонентов для вкладок на основе `reka-ui`.

#### Tabs (`Tabs.vue`)

Составной компонент-контейнер (`TabsRoot`, `TabsList`, `TabsTrigger`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `modelValue` | `string?` | — |
| `active` | `string?` | — |
| `defaultValue` | `string?` | — |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` |
| `class` | `string?` | — |
| `listClass` | `string?` | — |
| `contentClass` | `string?` | — |
| `tabs` | `Array<{ id: string; label: string }>?` | — |

> **Примечание:** `modelValue` и `active` мапятся в один `selectedValue` computed
> (`props.modelValue ?? props.active`). Двойной API может свидетельствовать о
> миграции.

**События:** `update:modelValue(value: string)`, `select(value: string)`

**Слоты:**
- `list` — кастомный список триггеров (если передан, заменяет автоматический рендер
  из пропса `tabs`).
- `default` — содержимое вкладок (обычно компоненты `TabContent`).

**CSS-классы:** `makosh-tabs`, `makosh-tabs-list`, `makosh-tabs-list--horizontal`,
`makosh-tabs-list--vertical`, `makosh-tabs-trigger`

> **Важно:** автоматически рендерящиеся триггеры (из пропса `tabs`) используют класс
> `makosh-tabs-trigger` (font-size `0.75rem`, padding `0.375rem 0.625rem`). Это
> *отличается* от класса `makosh-tab-trigger`, используемого в `TabTrigger.vue`.

#### TabTrigger (`TabTrigger.vue`)

Отдельный триггер вкладки (обёртка над `TabsTrigger` из `reka-ui`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `value` | `string` | *обязательный* |
| `disabled` | `boolean?` | `false` |
| `class` | `string?` | — |

**Слот:** default (содержимое триггера).

**CSS-класс:** `makosh-tab-trigger` (font-size `0.8125rem`, height `1.75rem`,
padding `0 0.75rem`, border-radius `calc(var(--hh-radius-sm) - 2px)`).

Состояние `[data-state="active"]` даёт цвет `--hh-accent`, фон `--hh-surface-panel` и
тень `0 1px 3px rgba(0, 0, 0, 0.2)`.

#### TabContent (`TabContent.vue`)

Содержимое вкладки (обёртка над `TabsContent` из `reka-ui`).

**Пропсы:**

| Проп | Тип | По умолчанию |
|---|---|---|
| `value` | `string` | *обязательный* |
| `class` | `string?` | — |
| `forceMount` | `boolean?` | `false` |

**Слот:** default.

**CSS-класс:** `makosh-tab-content`. При `focus-visible` — outline
`2px solid var(--hh-focus-ring)`.

---

## Shell-обёртки

### YandexTelemostSettingsPanelShell (`shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue`)

Прокси-компонент, импортирующий `YandexTelemostSettingsPanel` из
`../../integrations/yandexTelemost/components/YandexTelemostSettingsPanel.vue`.

**Пропсы:**

| Проп | Тип |
|---|---|
| `selectedAccount` | `ProviderAccount \| null?` |

Тип `ProviderAccount` импортирован из `./settingsBridge` (исходный код модуля
не предоставлен в данном контексте).

### ZoomSettingsPanelShell (`shared/zoom/ZoomSettingsPanelShell.vue`)

Прокси-компонент, импортирующий `ZoomSettingsPanel` из
`../../integrations/zoom/components/ZoomSettingsPanel.vue`.

**Пропсы:**

| Проп | Тип |
|---|---|
| `selectedAccount` | `ProviderAccount \| null?` |

**События:** `removed: []`

Тип `ProviderAccount` также импортирован из `./settingsBridge`.

---

## Дизайн-токены (`style.css`)

Глобальный файл стилей, определяющий:

- **Tailwind-директивы:** `@tailwind base`, `@tailwind components`, `@tailwind utilities`.
- **CSS-переменные Макошь** в `:root` с префиксом `--hh-*`.

### Основные категории токенов

| Категория | Примеры переменных |
|---|---|
| **Шрифт** | `--hh-font-sans` (Inter, SF Pro Display, системные) |
| **Размеры shell** | `--hh-shell-sidebar-width: 224px`, `--hh-shell-min-width: 800px`, `--hh-shell-min-height: 600px` |
| **Цвета (core)** | `--hh-color-bg: #02090b`, `--hh-color-accent: #2df0ce`, `--hh-color-danger: #ffabab`, `--hh-color-text-strong: #f2fffd`, `--hh-color-text-muted: #91a8a8` |
| **Цвета (семантические)** | `--hh-accent`, `--hh-accent-strong`, `--hh-accent-soft`, `--hh-accent-contrast` |
| **Границы и фон** | `--hh-border-subtle`, `--hh-border-accent`, `--hh-focus-ring`, `--hh-surface-panel`, `--hh-hover-bg`, `--hh-accent-tint` |
| **Радиусы** | `--hh-radius-xs: 4px`, `--hh-radius-sm: 6px`, `--hh-radius-md: 8px`, `--hh-radius-pill: 999px` |
| **Отступы** | `--hh-space-1: 4px … --hh-space-6: 24px`, `--hh-space-panel: 14px` |
| **Layout** | `--hh-layout-row: 37px`, `--hh-layout-gap: 10px` |
| **Размеры виджетов** | `--hh-widget-card: calc(var(--hh-widget-row) * 3)`, `--hh-widget-canvas: calc(var(--hh-widget-row) * 14)`, … |
| **Тени** | `--hh-shadow-sidebar`, `--hh-shadow-panel`, `--hh-shadow-modal` |
| **Прозрачность и блюр** | `--hh-panel-alpha: 0.7`, `--hh-panel-alpha-low: 0.56`, `--hh-panel-blur: 12px` |
| **Семантические алиасы** | `--hh-bg` → `--hh-color-bg`, `--hh-text-primary` → `--hh-color-text-strong`, `--hh-border` → `--hh-border-subtle` |
| **Compatibility aliases** | `--hh-bg-primary`, `--hh-bg-secondary`, `--hh-bg-hover`, `--color-surface`, `--text-primary`, `--bg-card`, `--accent`, `--success`, `--danger` и др. (для мигрированных экранов, использующих старые токены) |

Базовые стили `body` задают нулевой margin, шрифт `--hh-font-sans`, фон `--hh-color-bg`
и цвет текста `--hh-color-text`.

---

## Система тем (`theme-classes.css`)

Файл применяется к элементу `.viewport-guard` через theme store и определяет
классы для настройки внешнего вида shell.

### Акцентные цвета

Шесть цветовых схем, каждая переопределяет акцентные переменные:

| Класс | Цвет акцента |
|---|---|
| `.theme-accent-teal` | `#2df0ce` (по умолчанию) |
| `.theme-accent-cyan` | `#42ddff` |
| `.theme-accent-blue` | `#61a7ff` |
| `.theme-accent-violet` | `#b98cff` |
| `.theme-accent-amber` | `#f2b84b` |
| `.theme-accent-rose` | `#ff7aa8` |

Каждая схема задаёт значения для `--hh-color-accent`, `--hh-color-accent-strong`,
`--hh-color-accent-soft`, `--hh-color-accent-contrast`, `--hh-border-accent-soft`,
`--hh-border-accent`, `--hh-focus-ring`, `--hh-accent-tint`, `--hh-accent-control`.
Все схемы также устанавливают псевдонимы `--hh-accent`, `--hh-accent-strong`,
`--hh-accent-soft`, `--hh-accent-contrast`.

### Фоны shell

Классы вида `.shell-bg-*` задают `--hh-shell-bg-image` — URL изображения из
`/assets/shell-backgrounds/*.png`. Доступные фоны: `network-mesh`, `data-stream`,
`node-frame`, `eclipse-grid`, `dna-blueprint`, `forest-network`, `forest-stream`,
`knowledge-map`, `rune-gold`, `rune-teal`, а также `none`.

### Яркость фона

Классы `.shell-bg-brightness-{30..100}` задают `--hh-shell-bg-dim` от `0.74` до `0.18`.

### Прозрачность панелей

Классы `.panel-opacity-{40..100}` задают `--hh-panel-alpha` (от `0.4` до `1`),
`--hh-panel-alpha-low` (на `0.08`–`0.1` меньше) и `--hh-panel-opacity`.

### Блюр панелей

Классы `.panel-blur-{0,4,8,12,16,20,24}` задают `--hh-panel-blur` в пикселях.

### Плотность интерфейса

| Класс | `--hh-density-scale` | `--hh-space-panel` |
|---|---|---|
| `.spacing-density-compact` | `0.84` | `10px` |
| `.spacing-density-normal` | `1` | `14px` |
| `.spacing-density-comfortable` | `1.14` | `18px` |

### Фон рабочего стола

Класс `.desktop-shell` формирует многослойный фон:
```
linear-gradient(rgba(2, 9, 11, var(--hh-shell-bg-dim)), …),
var(--hh-shell-bg-image),
radial-gradient(circle at 72% 2%, rgba(23, 122, 121, 0.14), transparent 34%),
linear-gradient(180deg, rgba(7, 28, 32, 0.88), rgba(2, 9, 11, 0.98) 46%),
var(--hh-color-bg)
```

### Зарезервированные view-классы

Определены пустые селекторы `.view-home`, `.view-communications`, `.view-timeline`,
`.view-persons`, `.view-projects`, `.view-tasks`, `.view-calendar`, `.view-documents`,
`.view-notes`, `.view-knowledge`, `.view-review`, `.view-settings`, `.view-agents`,
`.view-organizations`, `.view-telegram`, `.view-whatsapp` — зарезервированы для
per-view CSS-переопределений.

---

## Стили поверхностей (`surfaces.css`)

Глобальный файл, определяющий классы для панелей, карточек, форм и других
поверхностей, используемых в shell и виджетах.

### Базовые поверхности

- `.hh-surface` — панель с рамкой `--hh-border-subtle`, радиусом `--hh-radius-md`,
  фоном `rgba(5, 22, 25, var(--hh-panel-alpha))`, backdrop-filter и тенью.
- `.hh-surface--raised` — фон `rgba(8, 29, 33, var(--hh-panel-alpha))`.
- `.hh-surface--deep` — фон `rgba(4, 18, 21, var(--hh-panel-alpha))`.

### Утилитарные классы

| Класс | Назначение |
|---|---|
| `.panel`, `.widget-frame`, `.info-card` | Общие панели с общим фоном, рамкой, блюром и тенью |
| `.view-header` | Заголовок страницы с flexbox, отступами и панельным фоном |
| `.view-title-with-icon` | Контейнер заголовка с иконкой |
| `.primary-button`, `.form-actions button` | Основная кнопка (`--hh-accent` фон, `--hh-accent-contrast` текст, hover-эффект `brightness(1.08)`, disabled — opacity `0.48`) |
| `.metric-grid` | Сетка метрик (`auto-fit`, minmax `150px`) |
| `.metric-card` | Карточка метрики |
| `.three-pane` | Трёхколоночный макет (колонки: `minmax(260px, 0.86fr) minmax(360px, 1.5fr) minmax(280px, 0.9fr)`) |
| `.conversation-list`, `.chat-pane`, `.stacked-rail`, `.info-card` | Компоненты коммуникаций |
| `.empty-panel` | Пустая панель-заглушка |
| `.round-icon`, `.hero-mark` | Круглая иконка (`38px`, акцентная рамка и фон) с вариантами `.green`, `.cyan`, `.blue`, `.ghost` |
| `.local-search` | Строка локального поиска |
| `.setup-form` | Форма настройки (Grid, gap `10px`), с вариантом `.compact-form` (две колонки) |
| `.form-actions` | Контейнер кнопок формы (flex, `justify-content: flex-end`) |
| `.health-row`, `.evidence-row`, `.detail-list li` | Строки статуса/данных |
| `.setup-state.success`, `.inline-error` | Статусные сообщения |
```

---

## Покрытие источников

| Файл | Факты, покрытые в предложенной странице |
|---|---|
| `DropdownMenuSeparator.vue` | Компонент-разделитель, зависимость `reka-ui`, CSS-класс `makosh-dropdown-separator`, фиксированные стили |
| `Icon.vue` | Обёртка `@iconify/vue`, пропсы `icon`, `size`, `class`, `aria-hidden="true"` |
| `Input.vue` | Пропсы (`modelValue`, `placeholder`, `disabled`, `readonly`, `type`, `error`, `class`), события (`update:modelValue`, `focus`, `blur`), CSS-классы и состояния стилей |
| `Label.vue` | Пропсы `htmlFor`, `class`, CSS-класс `makosh-label`, слот по умолчанию |
| `Popover.vue` | Зависимость `reka-ui` (6 примитивов), пропсы (`open`, `side`, `sideOffset`, `align`, `class`), событие `update:open`, слоты `trigger`/default, кнопка закрытия с `tabler:x`, анимация `popover-in` |
| `Progress.vue` | Зависимость `reka-ui` (`ProgressRoot`, `ProgressIndicator`), пропсы (`modelValue`, `max`, `indeterminate`, `size`, `class`), событие `update:modelValue`, `watchEffect` для transform, анимация `progress-indeterminate`, размеры `sm`/`md`/`lg` |
| `ScrollArea.vue` | Зависимость `reka-ui` (4 примитива), пропсы `class`, `maxHeight` (неиспользуемый), CSS-классы, вертикальный и горизонтальный скроллбары |
| `Select.vue` | Зависимость `reka-ui` (8 примитивов), пропсы, событие `update:modelValue`, `tabler:chevron-down`, `tabler:check`, классы и состояния |
| `Separator.vue` | Зависимость `reka-ui` (`Separator`), пропсы `orientation`, `decorative`, `class`, CSS-классы |
| `Sheet.vue` | Зависимость `reka-ui` (7 диалоговых примитивов), пропсы, событие `update:open`, слоты `trigger`/`header`/`default`/`footer`, анимации слайда, кнопка закрытия с `tabler:x` |
| `Skeleton.vue` | Пропсы (`class`, `width`, `height`, `rounded` — два последних не используются в шаблоне), анимация `makosh-skeleton-pulse` |
| `Surface.vue` | Пропсы `as`, `tone` (`'panel' \| 'raised' \| 'deep'`), отсутствие scoped-стилей, зависимость от глобального `surfaces.css` |
| `Switch.vue` | Зависимость `reka-ui` (`SwitchRoot`, `SwitchThumb`), пропсы, событие `update:modelValue`, состояния `checked`/`disabled`/`focus-visible` |
| `TabContent.vue` | Зависимость `reka-ui` (`TabsContent`), пропсы `value`, `class`, `forceMount`, CSS-класс `makosh-tab-content` |
| `TabTrigger.vue` | Зависимость `reka-ui` (`TabsTrigger`), пропсы, CSS-класс `makosh-tab-trigger` (отличается от `makosh-tabs-trigger` в `Tabs.vue`) |
| `Tabs.vue` | Зависимость `reka-ui` (`TabsRoot`, `TabsList`, `TabsTrigger`), пропсы (`modelValue`, `active`, `defaultValue`, `orientation`, `tabs` и др.), события (`update:modelValue`, `select`), слоты `list`/`default`, CSS-класс `makosh-tabs-trigger` |
| `Textarea.vue` | Пропсы, событие `update:modelValue`, CSS-классы, состояния стилей |
| `Toast.vue` | Зависимость `reka-ui` (6 примитивов), интерфейс `ToastItem`, provide-контекст, варианты `default`/`success`/`warning`/`error`, иконки вариантов, анимации появления/закрытия |
| `Tooltip.vue` | Зависимость `reka-ui` (5 примитивов), пропсы, слоты, задержка `400ms`, анимация `tooltip-in` |
| `YandexTelemostSettingsPanelShell.vue` | Прокси к `YandexTelemostSettingsPanel`, пропс `selectedAccount` типа `ProviderAccount` |
| `ZoomSettingsPanelShell.vue` | Прокси к `ZoomSettingsPanel`, пропс `selectedAccount`, событие `removed` |
| `style.css` | Tailwind-директивы, полный набор дизайн-токенов `--hh-*` (шрифт, цвета, радиусы, отступы, размеры shell, размеры виджетов, тени, прозрачность, семантические алиасы, compatibility aliases), базовые стили `body` |
| `surfaces.css` | Классы `.hh-surface`, `.hh-surface--*`, `.panel`, `.widget-frame`, `.view-header`, `.primary-button`, `.metric-grid`, `.three-pane`, `.setup-form` и др., их стили и вложенные селекторы |
| `theme-classes.css` | Классы акцентных тем (6 цветов), фоновые классы (11 вариантов), яркость фона (8 уровней), прозрачность панелей (7 уровней), блюр (7 уровней), плотность (3 уровня), класс `.desktop-shell`, зарезервированные view-классы |

## Исходные файлы

- [`frontend/src/shared/ui/DropdownMenuSeparator.vue`](../../../../frontend/src/shared/ui/DropdownMenuSeparator.vue)
- [`frontend/src/shared/ui/Icon.vue`](../../../../frontend/src/shared/ui/Icon.vue)
- [`frontend/src/shared/ui/Input.vue`](../../../../frontend/src/shared/ui/Input.vue)
- [`frontend/src/shared/ui/Label.vue`](../../../../frontend/src/shared/ui/Label.vue)
- [`frontend/src/shared/ui/Popover.vue`](../../../../frontend/src/shared/ui/Popover.vue)
- [`frontend/src/shared/ui/Progress.vue`](../../../../frontend/src/shared/ui/Progress.vue)
- [`frontend/src/shared/ui/ScrollArea.vue`](../../../../frontend/src/shared/ui/ScrollArea.vue)
- [`frontend/src/shared/ui/Select.vue`](../../../../frontend/src/shared/ui/Select.vue)
- [`frontend/src/shared/ui/Separator.vue`](../../../../frontend/src/shared/ui/Separator.vue)
- [`frontend/src/shared/ui/Sheet.vue`](../../../../frontend/src/shared/ui/Sheet.vue)
- [`frontend/src/shared/ui/Skeleton.vue`](../../../../frontend/src/shared/ui/Skeleton.vue)
- [`frontend/src/shared/ui/Surface.vue`](../../../../frontend/src/shared/ui/Surface.vue)
- [`frontend/src/shared/ui/Switch.vue`](../../../../frontend/src/shared/ui/Switch.vue)
- [`frontend/src/shared/ui/TabContent.vue`](../../../../frontend/src/shared/ui/TabContent.vue)
- [`frontend/src/shared/ui/TabTrigger.vue`](../../../../frontend/src/shared/ui/TabTrigger.vue)
- [`frontend/src/shared/ui/Tabs.vue`](../../../../frontend/src/shared/ui/Tabs.vue)
- [`frontend/src/shared/ui/Textarea.vue`](../../../../frontend/src/shared/ui/Textarea.vue)
- [`frontend/src/shared/ui/Toast.vue`](../../../../frontend/src/shared/ui/Toast.vue)
- [`frontend/src/shared/ui/Tooltip.vue`](../../../../frontend/src/shared/ui/Tooltip.vue)
- [`frontend/src/shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue`](../../../../frontend/src/shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue)
- [`frontend/src/shared/zoom/ZoomSettingsPanelShell.vue`](../../../../frontend/src/shared/zoom/ZoomSettingsPanelShell.vue)
- [`frontend/src/style.css`](../../../../frontend/src/style.css)
- [`frontend/src/styles/surfaces.css`](../../../../frontend/src/styles/surfaces.css)
- [`frontend/src/styles/theme-classes.css`](../../../../frontend/src/styles/theme-classes.css)

## Кандидаты на drift

1. **`Tabs.vue` vs `TabTrigger.vue` — разные CSS-классы для триггеров.**
   `Tabs.vue` рендерит встроенные триггеры с классом `makosh-tabs-trigger`
   (font-size `0.75rem`, padding `0.375rem 0.625rem`). Отдельный компонент
   `TabTrigger.vue` использует класс `makosh-tab-trigger` с отличающимися
   стилями (font-size `0.8125rem`, height `1.75rem`, padding `0 0.75rem`).
   Из контекста неясно, является ли это намеренным разделением
   (compound vs standalone) или расхождением, требующим унификации.

2. **`ScrollArea.vue` — неиспользуемый пропс `maxHeight`.**
   Пропс объявлен в `defineProps`, но не участвует ни в `computed`, ни в
   шаблоне, ни в стилях. Возможно, остался от предыдущей итерации или
   ожидает реализации.

3. **`Skeleton.vue` — неиспользуемые пропсы `width` и `height`.**
   Аналогично: объявлены, но не применены к элементу. Компонент всегда
   рендерится с `width: 100%` и `height: 1rem` из scoped-стилей.

4. **`Surface.vue` — отсутствие scoped-стилей.**
   Компонент полагается на глобальные классы из `surfaces.css`
   (`.hh-surface`, `.hh-surface--panel` и т. д.). Это создаёт неявную
   зависимость: удаление или переименование классов в `surfaces.css`
   сломает компонент без каких-либо ошибок на этапе сборки Vue SFC.

5. **`Tabs.vue` — двойной API `modelValue` / `active`.**
   Оба пропса мапятся в один `selectedValue` computed
   (`props.modelValue ?? props.active`). Двойной интерфейс может указывать
   на незавершённую миграцию с `active` на стандартный `modelValue`.
   Событие `select` дублирует `update:modelValue`.

6. **`Sheet.vue` — использование `Dialog*` примитивов под именем Sheet.**
   Компонент назван «Sheet», но построен исключительно на `DialogRoot`,
   `DialogContent` и т. д. из `reka-ui`. Сама библиотека не предоставляет
   Sheet-примитива, поэтому это архитектурное решение, однако имя компонента
   может вводить в заблуждение при поиске по кодовой базе.

7. **`Select.vue` — опциональный пропс `options` без значения по умолчанию.**
   В `withDefaults` для `options` не задан fallback. В шаблоне
   `v-for="opt in options"` — в Vue 3 это безопасно (рендерится ничего),
   но контракт неочевиден: если `options` не передан, список будет пустым,
   хотя ошибки не возникнет.
