# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `140-other-frontend-part-013`
- Group / Группа: `frontend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `frontend/src/shared/ui/DropdownMenuSeparator.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/DropdownMenuSeparator.vue`
- Size bytes / Размер в байтах: `298`
- Included characters / Включено символов: `298`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DropdownMenuSeparator } from 'reka-ui'
</script>

<template>
  <DropdownMenuSeparator class="makosh-dropdown-separator" />
</template>

<style scoped>
.makosh-dropdown-separator {
  height: 1px;
  background: var(--hh-border);
  margin: 0.25rem 0.5rem;
}
</style>
```

### `frontend/src/shared/ui/Icon.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Icon.vue`
- Size bytes / Размер в байтах: `413`
- Included characters / Включено символов: `413`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  icon: string
  size?: number | string
  class?: string
}>(), {
  size: 20
})

const iconClass = computed(() => props.class || '')
</script>

<template>
  <Icon
    :icon="icon"
    :width="size"
    :height="size"
    :class="iconClass"
    aria-hidden="true"
  />
</template>
```

### `frontend/src/shared/ui/Input.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Input.vue`
- Size bytes / Размер в байтах: `2307`
- Included characters / Включено символов: `2307`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  type?: string
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  readonly: false,
  type: 'text'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
}>()

const classes = computed(() => [
  'makosh-input',
  { 'makosh-input--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function handleFocus(event: FocusEvent): void {
  emit('focus', event)
}

function handleBlur(event: FocusEvent): void {
  emit('blur', event)
}
</script>

<template>
  <div class="makosh-input-wrapper">
    <input
      :class="classes"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :type="type"
      @input="handleInput"
      @focus="handleFocus"
      @blur="handleBlur"
    />
    <span v-if="error" class="makosh-input-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.makosh-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.makosh-input {
  width: 100%;
  height: 2.125rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  transition: all 150ms ease;
  outline: none;
  box-sizing: border-box;
}

.makosh-input::placeholder {
  color: var(--hh-text-muted);
}

.makosh-input:hover:not(:disabled):not(:read-only) {
  border-color: var(--hh-border-accent);
}

.makosh-input:focus {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.makosh-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.makosh-input--error {
  border-color: var(--hh-color-danger);
}

.makosh-input--error:focus {
  box-shadow: 0 0 0 1px var(--hh-color-danger);
}

.makosh-input-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
```

### `frontend/src/shared/ui/Label.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Label.vue`
- Size bytes / Размер в байтах: `447`
- Included characters / Включено символов: `447`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  htmlFor?: string
  class?: string
}>(), {})

const classes = computed(() => ['makosh-label', props.class])
</script>

<template>
  <label :class="classes" :for="htmlFor">
    <slot />
  </label>
</template>

<style scoped>
.makosh-label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  line-height: 1.2;
}
</style>
```

### `frontend/src/shared/ui/Popover.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Popover.vue`
- Size bytes / Размер в байтах: `2215`
- Included characters / Включено символов: `2215`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { PopoverRoot, PopoverTrigger, PopoverPortal, PopoverContent, PopoverArrow, PopoverClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  align?: 'start' | 'center' | 'end'
  class?: string
}>(), {
  side: 'bottom',
  sideOffset: 4,
  align: 'center'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => ['makosh-popover-content', props.class])
</script>

<template>
  <PopoverRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <PopoverTrigger as-child>
      <slot name="trigger" />
    </PopoverTrigger>
    <PopoverPortal>
      <PopoverContent :class="contentClasses" :side="side" :side-offset="sideOffset" :align="align">
        <PopoverArrow class="makosh-popover-arrow" />
        <slot />
        <PopoverClose class="makosh-popover-close" as-child>
          <button class="makosh-popover-close-btn">
            <Icon icon="tabler:x" size="0.875rem" />
          </button>
        </PopoverClose>
      </PopoverContent>
    </PopoverPortal>
  </PopoverRoot>
</template>

<style scoped>
.makosh-popover-content {
  min-width: 200px;
  padding: 1rem;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  animation: popover-in 150ms ease;
}

.makosh-popover-arrow {
  fill: var(--hh-surface-panel);
}

.makosh-popover-close {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
}

.makosh-popover-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border-radius: var(--hh-radius-xs);
  border: none;
  background: transparent;
  color: var(--hh-text-muted);
  cursor: pointer;
  transition: all 150ms ease;
}

.makosh-popover-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes popover-in {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
```

### `frontend/src/shared/ui/Progress.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Progress.vue`
- Size bytes / Размер в байтах: `2162`
- Included characters / Включено символов: `2162`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ProgressRoot, ProgressIndicator } from 'reka-ui'
import { computed, ref, watchEffect } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: number
  max?: number
  indeterminate?: boolean
  size?: 'sm' | 'md' | 'lg'
  class?: string
}>(), {
  modelValue: 0,
  max: 100,
  indeterminate: false,
  size: 'md'
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const percentage = computed(() => {
  if (props.max <= 0) return 0
  return Math.round((props.modelValue / props.max) * 100)
})

const rootClasses = computed(() => [
  'makosh-progress-root',
  `makosh-progress--${props.size}`,
  props.class,
  { 'makosh-progress--indeterminate': props.indeterminate }
])

const indicatorRef = ref<InstanceType<typeof ProgressIndicator> | null>(null)

watchEffect(() => {
  const element = indicatorRef.value?.$el as HTMLElement | undefined
  if (!element || props.indeterminate) return
  element.style.transform = `translateX(-${100 - percentage.value}%)`
})
</script>

<template>
  <ProgressRoot
    :model-value="modelValue"
    :max="max"
    :class="rootClasses"
    @update:model-value="(val: any) => emit('update:modelValue', Number(val))"
  >
    <ProgressIndicator ref="indicatorRef" class="makosh-progress-indicator" />
  </ProgressRoot>
</template>

<style scoped>
.makosh-progress-root {
  position: relative;
  overflow: hidden;
  background: var(--hh-hover-bg);
  border-radius: 9999px;
  width: 100%;
}

.makosh-progress--sm {
  height: 0.25rem;
}

.makosh-progress--md {
  height: 0.5rem;
}

.makosh-progress--lg {
  height: 0.75rem;
}

.makosh-progress-indicator {
  width: 100%;
  height: 100%;
  border-radius: inherit;
  background: var(--hh-accent);
  transition: transform 300ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-progress--indeterminate .makosh-progress-indicator {
  animation: progress-indeterminate 1.5s ease-in-out infinite;
  width: 40%;
  background: linear-gradient(90deg, transparent, var(--hh-accent), transparent);
}

@keyframes progress-indeterminate {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(350%);
  }
}
</style>
```

### `frontend/src/shared/ui/ScrollArea.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/ScrollArea.vue`
- Size bytes / Размер в байтах: `1724`
- Included characters / Включено символов: `1724`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ScrollAreaRoot, ScrollAreaViewport, ScrollAreaScrollbar, ScrollAreaThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  maxHeight?: string
}>(), {})

const classes = computed(() => ['makosh-scroll-area', props.class])
</script>

<template>
  <ScrollAreaRoot :class="classes">
    <ScrollAreaViewport class="makosh-scroll-viewport">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar class="makosh-scrollbar" orientation="vertical">
      <ScrollAreaThumb class="makosh-scroll-thumb" />
    </ScrollAreaScrollbar>
    <ScrollAreaScrollbar class="makosh-scrollbar" orientation="horizontal">
      <ScrollAreaThumb class="makosh-scroll-thumb" />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>

<style scoped>
.makosh-scroll-area {
  overflow: hidden;
  position: relative;
}

.makosh-scroll-viewport {
  width: 100%;
  height: 100%;
}

.makosh-scrollbar {
  display: flex;
  user-select: none;
  touch-action: none;
  transition: background 160ms ease;
  background: transparent;
}

.makosh-scrollbar[data-orientation="vertical"] {
  width: 0.5rem;
  padding: 0.125rem 0;
}

.makosh-scrollbar[data-orientation="horizontal"] {
  height: 0.5rem;
  padding: 0 0.125rem;
  flex-direction: column;
}

.makosh-scrollbar:hover {
  background: var(--hh-hover-bg);
}

.makosh-scroll-thumb {
  flex: 1;
  background: var(--hh-border);
  border-radius: var(--hh-radius-pill);
  position: relative;
}

.makosh-scroll-thumb::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  height: 100%;
  min-width: 2.5rem;
  min-height: 2.5rem;
}
</style>
```

### `frontend/src/shared/ui/Select.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Select.vue`
- Size bytes / Размер в байтах: `4059`
- Included characters / Включено символов: `4057`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { SelectRoot, SelectTrigger, SelectValue, SelectContent, SelectItem, SelectItemIndicator, SelectViewport, SelectPortal } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  error?: string
  class?: string
  options?: Array<{ value: string; label: string }>
}>(), {
  modelValue: '',
  placeholder: 'Select…',
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const triggerClasses = computed(() => [
  'makosh-select-trigger',
  { 'makosh-select--error': props.error },
  props.class
])
</script>

<template>
  <div class="makosh-select-wrapper">
    <SelectRoot
      :model-value="modelValue || undefined"
      :disabled="disabled"
      @update:model-value="(val) => emit('update:modelValue', val || '')"
    >
      <SelectTrigger :class="triggerClasses">
        <SelectValue :placeholder="placeholder" class="makosh-select-value" />
        <Icon icon="tabler:chevron-down" size="1rem" class="makosh-select-chevron" />
      </SelectTrigger>
      <SelectPortal>
        <SelectContent class="makosh-select-content" :side-offset="4">
          <SelectViewport class="makosh-select-viewport">
            <SelectItem
              v-for="opt in options"
              :key="opt.value"
              :value="opt.value"
              class="makosh-select-item"
            >
              <SelectItemIndicator>
                <Icon icon="tabler:check" size="0.875rem" class="makosh-select-check" />
              </SelectItemIndicator>
              <span>{{ opt.label }}</span>
            </SelectItem>
          </SelectViewport>
        </SelectContent>
      </SelectPortal>
    </SelectRoot>
    <span v-if="error" class="makosh-select-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.makosh-select-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.makosh-select-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  width: 100%;
  height: 2.125rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  cursor: pointer;
  transition: all 150ms ease;
  outline: none;
  text-align: left;
  box-sizing: border-box;
}

.makosh-select-trigger:hover {
  border-color: var(--hh-border-accent);
}

.makosh-select-trigger:focus-visible {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.makosh-select-trigger[data-placeholder] .makosh-select-value {
  color: var(--hh-text-muted);
}

.makosh-select--error {
  border-color: var(--hh-color-danger);
}

.makosh-select-chevron {
  color: var(--hh-text-muted);
  flex-shrink: 0;
  transition: transform 200ms ease;
}

.makosh-select-trigger[data-state="open"] .makosh-select-chevron {
  transform: rotate(180deg);
}

.makosh-select-content {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
  z-index: 100;
  min-width: var(--reka-select-trigger-width);
  overflow: hidden;
}

.makosh-select-viewport {
  padding: 0.25rem;
}

.makosh-select-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.8125rem;
  color: var(--hh-text-secondary);
  border-radius: var(--hh-radius-xs);
  cursor: pointer;
  outline: none;
  user-select: none;
  transition: background 100ms ease;
}

.makosh-select-item[data-highlighted] {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

.makosh-select-item[data-state="checked"] {
  color: var(--hh-accent);
}

.makosh-select-check {
  color: var(--hh-accent);
  flex-shrink: 0;
}

.makosh-select-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
```

### `frontend/src/shared/ui/Separator.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Separator.vue`
- Size bytes / Размер в байтах: `771`
- Included characters / Включено символов: `771`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { Separator as RekaSeparator } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  orientation?: 'horizontal' | 'vertical'
  decorative?: boolean
  class?: string
}>(), {
  orientation: 'horizontal',
  decorative: true
})

const classes = computed(() => [
  'makosh-separator',
  `makosh-separator--${props.orientation}`,
  props.class
])
</script>

<template>
  <RekaSeparator
    :class="classes"
    :orientation="orientation"
    :decorative="decorative"
  />
</template>

<style scoped>
.makosh-separator {
  flex-shrink: 0;
  background: var(--hh-border);
}

.makosh-separator--horizontal {
  height: 1px;
  width: 100%;
}

.makosh-separator--vertical {
  width: 1px;
  height: 100%;
}
</style>
```

### `frontend/src/shared/ui/Sheet.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Sheet.vue`
- Size bytes / Размер в байтах: `4383`
- Included characters / Включено символов: `4383`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { DialogRoot, DialogTrigger, DialogPortal, DialogOverlay, DialogContent, DialogTitle, DialogDescription, DialogClose } from 'reka-ui'
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  open?: boolean
  title?: string
  description?: string
  side?: 'left' | 'right' | 'top' | 'bottom'
  class?: string
  contentClass?: string
}>(), {
  open: false,
  side: 'right'
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const contentClasses = computed(() => [
  'makosh-sheet-content',
  `makosh-sheet--${props.side}`,
  props.contentClass
])
</script>

<template>
  <DialogRoot :open="open" @update:open="(val) => emit('update:open', val)">
    <DialogTrigger as-child>
      <slot name="trigger" />
    </DialogTrigger>
    <DialogPortal>
      <DialogOverlay class="makosh-sheet-overlay">
        <DialogContent :class="contentClasses">
          <div class="makosh-sheet-header">
            <DialogTitle v-if="title" class="makosh-sheet-title">{{ title }}</DialogTitle>
            <DialogDescription v-if="description" class="makosh-sheet-description">{{ description }}</DialogDescription>
            <slot name="header" />
          </div>
          <div class="makosh-sheet-body">
            <slot />
          </div>
          <div v-if="$slots.footer" class="makosh-sheet-footer">
            <slot name="footer" />
          </div>
          <DialogClose class="makosh-sheet-close" as-child>
            <button class="makosh-sheet-close-btn">
              <Icon icon="tabler:x" size="1.125rem" />
            </button>
          </DialogClose>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </DialogRoot>
</template>

<style scoped>
.makosh-sheet-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  z-index: 100;
  animation: sheet-overlay-in 200ms ease;
}

/* Side alignment */
.makosh-sheet--left {
  align-self: stretch;
  margin-right: auto;
  animation: sheet-slide-left 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-sheet--right {
  align-self: stretch;
  margin-left: auto;
  animation: sheet-slide-right 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-sheet--top {
  align-self: flex-start;
  width: 100%;
  animation: sheet-slide-top 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-sheet--bottom {
  align-self: flex-end;
  width: 100%;
  animation: sheet-slide-bottom 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-sheet-content {
  position: relative;
  display: flex;
  flex-direction: column;
  width: 90vw;
  max-width: 400px;
  max-height: 100vh;
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  box-shadow: var(--hh-shadow-modal);
  overflow-y: auto;
}

.makosh-sheet-header {
  padding: 1.5rem 1.5rem 0;
  flex-shrink: 0;
}

.makosh-sheet-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  margin-bottom: 0.25rem;
}

.makosh-sheet-description {
  font-size: 0.8125rem;
  color: var(--hh-text-muted);
  line-height: 1.4;
}

.makosh-sheet-body {
  padding: 1.25rem 1.5rem;
  flex: 1;
  overflow-y: auto;
}

.makosh-sheet-footer {
  padding: 1rem 1.5rem;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  border-top: 1px solid var(--hh-border);
  flex-shrink: 0;
}

.makosh-sheet-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
}

.makosh-sheet-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--hh-radius-xs);
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.makosh-sheet-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes sheet-overlay-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes sheet-slide-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

@keyframes sheet-slide-left {
  from { transform: translateX(-100%); }
  to { transform: translateX(0); }
}

@keyframes sheet-slide-top {
  from { transform: translateY(-100%); }
  to { transform: translateY(0); }
}

@keyframes sheet-slide-bottom {
  from { transform: translateY(100%); }
  to { transform: translateY(0); }
}
</style>
```

### `frontend/src/shared/ui/Skeleton.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Skeleton.vue`
- Size bytes / Размер в байтах: `800`
- Included characters / Включено символов: `800`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  class?: string
  width?: string
  height?: string
  rounded?: boolean
}>(), {
  width: '100%',
  height: '1rem',
  rounded: false
})

const classes = computed(() => [
  'makosh-skeleton',
  { 'makosh-skeleton--rounded': props.rounded },
  props.class
])
</script>

<template>
  <div :class="classes" />
</template>

<style scoped>
.makosh-skeleton {
  background: var(--hh-hover-bg);
  border-radius: var(--hh-radius-xs);
  width: 100%;
  height: 1rem;
  animation: makosh-skeleton-pulse 1.5s ease-in-out infinite;
}

.makosh-skeleton--rounded {
  border-radius: var(--hh-radius-pill);
}

@keyframes makosh-skeleton-pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.8;
  }
}
</style>
```

### `frontend/src/shared/ui/Surface.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Surface.vue`
- Size bytes / Размер в байтах: `316`
- Included characters / Включено символов: `316`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
type SurfaceTone = 'panel' | 'raised' | 'deep'

withDefaults(
	defineProps<{
		as?: string
		tone?: SurfaceTone
	}>(),
	{
		as: 'section',
		tone: 'panel'
	}
)
</script>

<template>
	<component :is="as" class="hh-surface" :class="`hh-surface--${tone}`">
		<slot />
	</component>
</template>
```

### `frontend/src/shared/ui/Switch.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Switch.vue`
- Size bytes / Размер в байтах: `1533`
- Included characters / Включено символов: `1533`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { SwitchRoot, SwitchThumb } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: boolean
  disabled?: boolean
  class?: string
}>(), {
  modelValue: false,
  disabled: false
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const rootClasses = computed(() => ['makosh-switch', { 'makosh-switch--disabled': props.disabled }, props.class])
</script>

<template>
  <SwitchRoot
    :class="rootClasses"
    :checked="modelValue"
    :disabled="disabled"
    @update:checked="(val: boolean) => emit('update:modelValue', val)"
  >
    <SwitchThumb class="makosh-switch-thumb" />
  </SwitchRoot>
</template>

<style scoped>
.makosh-switch {
  position: relative;
  width: 2rem;
  height: 1.125rem;
  border-radius: var(--hh-radius-pill);
  background: var(--hh-border);
  border: none;
  cursor: pointer;
  transition: background 200ms ease;
  flex-shrink: 0;
}

.makosh-switch[data-state="checked"] {
  background: var(--hh-accent);
}

.makosh-switch--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.makosh-switch:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.makosh-switch-thumb {
  display: block;
  width: 0.875rem;
  height: 0.875rem;
  border-radius: 50%;
  background: white;
  transition: transform 200ms ease;
  transform: translateX(0.125rem);
  will-change: transform;
}

.makosh-switch[data-state="checked"] .makosh-switch-thumb {
  transform: translateX(1rem);
}
</style>
```

### `frontend/src/shared/ui/TabContent.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/TabContent.vue`
- Size bytes / Размер в байтах: `654`
- Included characters / Включено символов: `654`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { TabsContent } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  class?: string
  forceMount?: boolean
}>(), {
  forceMount: false
})

const classes = computed(() => ['makosh-tab-content', props.class])
</script>

<template>
  <TabsContent
    :class="classes"
    :value="value"
    :force-mount="forceMount"
  >
    <slot />
  </TabsContent>
</template>

<style scoped>
.makosh-tab-content {
  outline: none;
}

.makosh-tab-content:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
  border-radius: var(--hh-radius-sm);
}
</style>
```

### `frontend/src/shared/ui/TabTrigger.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/TabTrigger.vue`
- Size bytes / Размер в байтах: `1363`
- Included characters / Включено символов: `1363`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  value: string
  disabled?: boolean
  class?: string
}>(), {
  disabled: false
})

const classes = computed(() => ['makosh-tab-trigger', { 'makosh-tab-trigger--disabled': props.disabled }, props.class])
</script>

<template>
  <TabsTrigger
    :class="classes"
    :value="value"
    :disabled="disabled"
  >
    <slot />
  </TabsTrigger>
</template>

<style scoped>
.makosh-tab-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.375rem;
  height: 1.75rem;
  padding: 0 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  border-radius: calc(var(--hh-radius-sm) - 2px);
  cursor: pointer;
  transition: all 150ms ease;
  white-space: nowrap;
  outline: none;
}

.makosh-tab-trigger:hover {
  color: var(--hh-text-secondary);
}

.makosh-tab-trigger[data-state="active"] {
  color: var(--hh-accent);
  background: var(--hh-surface-panel);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.makosh-tab-trigger:focus-visible {
  outline: 2px solid var(--hh-focus-ring);
  outline-offset: 2px;
}

.makosh-tab-trigger--disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
```

### `frontend/src/shared/ui/Tabs.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Tabs.vue`
- Size bytes / Размер в байтах: `2389`
- Included characters / Включено символов: `2389`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { TabsRoot, TabsList, TabsTrigger } from 'reka-ui'
import { computed } from 'vue'

type МакошьTab = {
  id: string
  label: string
}

// Re-export with Макошь styling
const props = withDefaults(defineProps<{
  modelValue?: string
  active?: string
  defaultValue?: string
  orientation?: 'horizontal' | 'vertical'
  class?: string
  listClass?: string
  contentClass?: string
  tabs?: МакошьTab[]
}>(), {
  orientation: 'horizontal'
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [value: string]
}>()

const tabs = computed(() => props.tabs ?? [])
const selectedValue = computed(() => props.modelValue ?? props.active)
const rootClasses = computed(() => ['makosh-tabs', props.class])
const listClasses = computed(() => ['makosh-tabs-list', `makosh-tabs-list--${props.orientation}`, props.listClass])

function handleUpdateModelValue(value: string | number) {
  const nextValue = String(value)
  emit('update:modelValue', nextValue)
  emit('select', nextValue)
}
</script>

<template>
  <TabsRoot
    :class="rootClasses"
    :model-value="selectedValue"
    :default-value="defaultValue"
    :orientation="orientation"
    @update:model-value="handleUpdateModelValue"
  >
    <TabsList :class="listClasses">
      <slot name="list">
        <TabsTrigger
          v-for="tab in tabs"
          :key="tab.id"
          :value="tab.id"
          class="makosh-tabs-trigger"
        >
          {{ tab.label }}
        </TabsTrigger>
      </slot>
    </TabsList>
    <slot />
  </TabsRoot>
</template>

<style scoped>
.makosh-tabs {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.makosh-tabs-list {
  display: flex;
  gap: 0.125rem;
  background: var(--hh-hover-bg);
  border-radius: var(--hh-radius-sm);
  padding: 0.1875rem;
}

.makosh-tabs-list--vertical {
  flex-direction: column;
}

.makosh-tabs-trigger {
  border: none;
  border-radius: var(--hh-radius-xs, 0.25rem);
  background: transparent;
  color: var(--hh-text-secondary, #6b7280);
  cursor: pointer;
  font: inherit;
  font-size: 0.75rem;
  padding: 0.375rem 0.625rem;
}

.makosh-tabs-trigger:hover {
  background: var(--hh-bg-hover, #f3f4f6);
  color: var(--hh-text-primary, #1f2937);
}

.makosh-tabs-trigger[data-state='active'] {
  background: var(--hh-bg-primary, #ffffff);
  color: var(--hh-accent, #3b82f6);
  font-weight: 600;
}
</style>
```

### `frontend/src/shared/ui/Textarea.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Textarea.vue`
- Size bytes / Размер в байтах: `1953`
- Included characters / Включено символов: `1953`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  placeholder?: string
  disabled?: boolean
  rows?: number
  error?: string
  class?: string
}>(), {
  modelValue: '',
  placeholder: '',
  disabled: false,
  rows: 3
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const classes = computed(() => [
  'makosh-textarea',
  { 'makosh-textarea--error': props.error },
  props.class
])

function handleInput(event: Event): void {
  const target = event.target as HTMLTextAreaElement
  emit('update:modelValue', target.value)
}
</script>

<template>
  <div class="makosh-textarea-wrapper">
    <textarea
      :class="classes"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :rows="rows"
      @input="handleInput"
    />
    <span v-if="error" class="makosh-textarea-error">{{ error }}</span>
  </div>
</template>

<style scoped>
.makosh-textarea-wrapper {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.makosh-textarea {
  width: 100%;
  padding: 0.625rem 0.75rem;
  font-family: var(--hh-font-sans);
  font-size: 0.8125rem;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, rgba(4, 18, 21, 0.8));
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-sm);
  transition: all 150ms ease;
  outline: none;
  resize: vertical;
  box-sizing: border-box;
  line-height: 1.5;
}

.makosh-textarea::placeholder {
  color: var(--hh-text-muted);
}

.makosh-textarea:hover:not(:disabled) {
  border-color: var(--hh-border-accent);
}

.makosh-textarea:focus {
  border-color: var(--hh-accent);
  box-shadow: 0 0 0 1px var(--hh-focus-ring);
}

.makosh-textarea:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.makosh-textarea--error {
  border-color: var(--hh-color-danger);
}

.makosh-textarea-error {
  font-size: 0.75rem;
  color: var(--hh-color-danger);
}
</style>
```

### `frontend/src/shared/ui/Toast.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Toast.vue`
- Size bytes / Размер в байтах: `5513`
- Included characters / Включено символов: `5513`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ToastProvider, ToastViewport, ToastRoot, ToastTitle, ToastDescription, ToastClose } from 'reka-ui'
import { ref, computed, provide, inject, type Ref } from 'vue'
import Icon from './Icon.vue'

export interface ToastItem {
  id: string
  title?: string
  description?: string
  variant?: 'default' | 'success' | 'warning' | 'error'
  duration?: number
}

const TOAST_INJECTION_KEY = 'makosh-toast-context'

const props = withDefaults(defineProps<{
  /** Swipe direction to dismiss */
  swipeDirection?: 'right' | 'left' | 'up' | 'down'
  /** Duration in ms before auto-dismiss */
  duration?: number
  class?: string
}>(), {
  swipeDirection: 'right',
  duration: 4000
})

const toasts = ref<ToastItem[]>([]) as Ref<ToastItem[]>

let toastCounter = 0

function addToast(item: Omit<ToastItem, 'id'>): string {
  const id = `toast-${++toastCounter}`
  toasts.value = [...toasts.value, { ...item, id }]
  return id
}

function removeToast(id: string): void {
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

function success(title: string, description?: string): string {
  return addToast({ title, description, variant: 'success', duration: props.duration })
}

function warning(title: string, description?: string): string {
  return addToast({ title, description, variant: 'warning', duration: props.duration })
}

function error(title: string, description?: string): string {
  return addToast({ title, description, variant: 'error', duration: props.duration })
}

provide(TOAST_INJECTION_KEY, { addToast, removeToast, success, warning, error })

const viewportClasses = computed(() => [
  'makosh-toast-viewport',
  props.class
])

const variantIcons: Record<string, string> = {
  success: 'tabler:check-circle',
  warning: 'tabler:alert-triangle',
  error: 'tabler:alert-circle'
}
</script>

<template>
  <ToastProvider :swipe-direction="swipeDirection" :duration="duration">
    <slot />

    <ToastViewport :class="viewportClasses">
      <ToastRoot
        v-for="toast in toasts"
        :key="toast.id"
        :class="['makosh-toast-root', `makosh-toast--${toast.variant || 'default'}`]"
        @update:open="(open: boolean) => { if (!open) removeToast(toast.id) }"
      >
        <div class="makosh-toast-inner">
          <Icon
            v-if="toast.variant && toast.variant !== 'default'"
            :icon="variantIcons[toast.variant]"
            size="1.125rem"
            class="makosh-toast-variant-icon"
          />
          <div class="makosh-toast-content">
            <ToastTitle v-if="toast.title" class="makosh-toast-title">
              {{ toast.title }}
            </ToastTitle>
            <ToastDescription v-if="toast.description" class="makosh-toast-description">
              {{ toast.description }}
            </ToastDescription>
          </div>
          <ToastClose class="makosh-toast-close-btn">
            <Icon icon="tabler:x" size="1rem" />
          </ToastClose>
        </div>
      </ToastRoot>
    </ToastViewport>
  </ToastProvider>
</template>

<style scoped>
.makosh-toast-viewport {
  position: fixed;
  bottom: 1rem;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0;
  max-width: 360px;
  width: 100%;
  z-index: 200;
  outline: none;
  list-style: none;
}

.makosh-toast-root {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  box-shadow: var(--hh-shadow-modal);
  padding: 0.875rem 1rem;
  animation: toast-slide-in 250ms cubic-bezier(0.16, 1, 0.3, 1);
}

.makosh-toast-root[data-state="closed"] {
  animation: toast-slide-out 200ms ease;
}

.makosh-toast-inner {
  display: flex;
  align-items: flex-start;
  gap: 0.625rem;
}

.makosh-toast-variant-icon {
  flex-shrink: 0;
  margin-top: 0.0625rem;
}

.makosh-toast--success .makosh-toast-variant-icon {
  color: var(--hh-status-success, #22c55e);
}

.makosh-toast--warning .makosh-toast-variant-icon {
  color: var(--hh-status-warning, #f59e0b);
}

.makosh-toast--error .makosh-toast-variant-icon {
  color: var(--hh-status-danger, #ef4444);
}

.makosh-toast--success {
  border-color: color-mix(in srgb, var(--hh-status-success, #22c55e) 30%, transparent);
}

.makosh-toast--warning {
  border-color: color-mix(in srgb, var(--hh-status-warning, #f59e0b) 30%, transparent);
}

.makosh-toast--error {
  border-color: color-mix(in srgb, var(--hh-status-danger, #ef4444) 30%, transparent);
}

.makosh-toast-content {
  flex: 1;
  min-width: 0;
}

.makosh-toast-title {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--hh-text-primary);
  line-height: 1.4;
}

.makosh-toast-description {
  font-size: 0.75rem;
  color: var(--hh-text-secondary);
  line-height: 1.4;
  margin-top: 0.125rem;
}

.makosh-toast-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 1.375rem;
  height: 1.375rem;
  border-radius: var(--hh-radius-xs);
  color: var(--hh-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background 150ms ease, color 150ms ease;
}

.makosh-toast-close-btn:hover {
  background: var(--hh-hover-bg);
  color: var(--hh-text-primary);
}

@keyframes toast-slide-in {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

@keyframes toast-slide-out {
  from {
    transform: translateX(0);
    opacity: 1;
  }
  to {
    transform: translateX(100%);
    opacity: 0;
  }
}
</style>
```

### `frontend/src/shared/ui/Tooltip.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/ui/Tooltip.vue`
- Size bytes / Размер в байтах: `1471`
- Included characters / Включено символов: `1471`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { TooltipRoot, TooltipTrigger, TooltipPortal, TooltipContent, TooltipArrow } from 'reka-ui'
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  content?: string
  side?: 'top' | 'bottom' | 'left' | 'right'
  sideOffset?: number
  delayDuration?: number
  class?: string
}>(), {
  side: 'top',
  sideOffset: 4,
  delayDuration: 400
})

const contentClasses = computed(() => ['makosh-tooltip-content', props.class])
</script>

<template>
  <TooltipRoot :delay-duration="delayDuration">
    <TooltipTrigger as-child>
      <slot name="trigger" />
    </TooltipTrigger>
    <TooltipPortal>
      <TooltipContent :class="contentClasses" :side="side" :side-offset="sideOffset">
        <slot>{{ content }}</slot>
        <TooltipArrow class="makosh-tooltip-arrow" />
      </TooltipContent>
    </TooltipPortal>
  </TooltipRoot>
</template>

<style scoped>
.makosh-tooltip-content {
  padding: 0.375rem 0.625rem;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--hh-text-primary);
  background: var(--hh-surface-deep, #041215);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-xs);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  line-height: 1.3;
  z-index: 150;
  animation: tooltip-in 150ms ease;
}

.makosh-tooltip-arrow {
  fill: var(--hh-surface-deep, #041215);
}

@keyframes tooltip-in {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
```

### `frontend/src/shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/yandexTelemost/YandexTelemostSettingsPanelShell.vue`
- Size bytes / Размер в байтах: `367`
- Included characters / Включено символов: `367`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import YandexTelemostSettingsPanel from '../../integrations/yandexTelemost/components/YandexTelemostSettingsPanel.vue'
import type { ProviderAccount } from './settingsBridge'

defineProps<{
  selectedAccount?: ProviderAccount | null
}>()
</script>

<template>
  <YandexTelemostSettingsPanel :selected-account="selectedAccount" />
</template>
```

### `frontend/src/shared/zoom/ZoomSettingsPanelShell.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/shared/zoom/ZoomSettingsPanelShell.vue`
- Size bytes / Размер в байтах: `399`
- Included characters / Включено символов: `399`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import ZoomSettingsPanel from '../../integrations/zoom/components/ZoomSettingsPanel.vue'
import type { ProviderAccount } from './settingsBridge'

defineProps<{
  selectedAccount?: ProviderAccount | null
}>()

defineEmits<{
  removed: []
}>()
</script>

<template>
  <ZoomSettingsPanel
    :selected-account="selectedAccount"
    @removed="$emit('removed')"
  />
</template>
```

### `frontend/src/style.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/style.css`
- Size bytes / Размер в байтах: `6791`
- Included characters / Включено символов: `6789`
- Truncated / Обрезано: `no`

```text
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Макошь Design Tokens — CSS custom properties for runtime access */
:root {
	/* Font family */
	--hh-font-sans: 'Inter', 'SF Pro Display', ui-sans-serif, system-ui, -apple-system,
		BlinkMacSystemFont, 'Segoe UI', sans-serif;

	/* Shell layout */
	--hh-supported-min-width: 800px;
	--hh-supported-min-height: 600px;
	--hh-shell-min-width: var(--hh-supported-min-width);
	--hh-shell-min-height: var(--hh-supported-min-height);
	--hh-shell-sidebar-width: 224px;
	--hh-shell-content-min-width: 0px;
	--hh-shell-sidebar-width-compact: 208px;
	--hh-shell-sidebar-width-rail: 64px;
	--hh-shell-content-min-width-compact: 0px;
	--hh-shell-right-inset: 14px;
	--hh-shell-bottom-inset: 0px;
	--hh-shell-topbar-offset: 10px;
	--hh-shell-workspace-gap: 10px;

	/* Core colors */
	--hh-color-bg: #02090b;
	--hh-color-bg-raised: #020d10;
	--hh-color-surface: #06181b;
	--hh-color-surface-deep: #041215;
	--hh-color-text: #eefefb;
	--hh-color-text-strong: #f2fffd;
	--hh-color-text-bright: #ffffff;
	--hh-color-text-soft: #dcefed;
	--hh-color-text-muted: #91a8a8;
	--hh-color-text-subtle: #8ea4a6;
	--hh-color-text-dim: #849ca0;
	--hh-color-accent: #2df0ce;
	--hh-color-accent-strong: #25d8bd;
	--hh-color-accent-soft: #9ee8df;
	--hh-color-accent-contrast: #032522;
	--hh-color-danger: #ffabab;
	--hh-color-danger-strong: #ef3140;
	--hh-accent: var(--hh-color-accent);
	--hh-accent-strong: var(--hh-color-accent-strong);
	--hh-accent-soft: var(--hh-color-accent-soft);
	--hh-accent-contrast: var(--hh-color-accent-contrast);

	/* Border & surface colors */
	--hh-border-accent-soft: rgba(45, 240, 206, 0.18);
	--hh-border-accent: rgba(45, 240, 206, 0.42);
	--hh-border-subtle: rgba(111, 205, 195, 0.14);
	--hh-border-muted: rgba(102, 189, 180, 0.1);
	--hh-focus-ring: rgba(45, 240, 206, 0.62);
	--hh-surface-tint: rgba(5, 22, 25, 0.78);
	--hh-surface-panel: rgba(8, 29, 33, 0.94);
	--hh-surface-deep: var(--hh-color-surface-deep);
	--hh-accent-tint: rgba(45, 240, 206, 0.08);
	--hh-accent-control: rgba(25, 154, 132, 0.2);
	--hh-danger-tint: rgba(128, 32, 40, 0.26);

	/* Semantic status colors */
	--hh-status-accent-surface: var(--hh-accent-tint);
	--hh-status-accent-text: var(--hh-color-accent);
	--hh-status-warning-surface: rgba(240, 170, 70, 0.16);
	--hh-status-warning-text: #f4c889;
	--hh-status-info-surface: rgba(120, 156, 240, 0.18);
	--hh-status-info-text: #aec6f7;
	--hh-status-success-surface: rgba(45, 214, 150, 0.16);
	--hh-status-success-text: #7fe6b4;
	--hh-status-danger-surface: var(--hh-danger-tint);
	--hh-status-danger-text: var(--hh-color-danger);
	--hh-status-archive-surface: rgba(176, 132, 240, 0.18);
	--hh-status-archive-text: #cdb2f2;
	--hh-status-neutral-surface: rgba(124, 156, 156, 0.12);
	--hh-status-neutral-text: var(--hh-color-text-muted);

	/* Border radius */
	--hh-radius-xs: 4px;
	--hh-radius-sm: 6px;
	--hh-radius-control: 7px;
	--hh-radius-md: 8px;
	--hh-radius-lg: 14px;
	--hh-radius-xl: 18px;
	--hh-radius-pill: 999px;
	--hh-radius-round: 50%;
	--hh-radius-sidebar: 0 18px 34px 0;

	/* Spacing */
	--hh-space-1: 4px;
	--hh-space-2: 8px;
	--hh-space-3: 12px;
	--hh-space-4: 16px;
	--hh-space-5: 20px;
	--hh-space-6: 24px;
	--hh-density-scale: 1;
	--hh-space-panel: 14px;
	--hh-space-section: 16px;
	--hh-space-control-x: 12px;

	/* Layout */
	--hh-layout-row: 37px;
	--hh-layout-gap: 10px;
	--hh-layout-columns: 12;
	--hh-layout-topbar-height: var(--hh-layout-row);
	--hh-layout-summary-height: calc(var(--hh-layout-row) * 3);
	--hh-layout-status-height: var(--hh-layout-row);

	/* Widget dimensions */
	--hh-widget-unit: var(--hh-layout-row);
	--hh-widget-row: var(--hh-layout-row);
	--hh-widget-card-compact: calc(var(--hh-widget-row) * 3);
	--hh-widget-card: calc(var(--hh-widget-row) * 3);
	--hh-widget-card-large: calc(var(--hh-widget-row) * 6);
	--hh-widget-panel: calc(var(--hh-widget-row) * 6);
	--hh-widget-panel-large: calc(var(--hh-widget-row) * 8);
	--hh-widget-canvas: calc(var(--hh-widget-row) * 14);
	--hh-widget-canvas-large: calc(var(--hh-widget-row) * 23);
	--hh-widget-workbench: calc(var(--hh-widget-row) * 18);
	--hh-widget-workbench-large: calc(var(--hh-widget-row) * 26);
	--hh-widget-workbench-tall: calc(var(--hh-widget-row) * 28);

	/* Shadows */
	--hh-shadow-sidebar: inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28);
	--hh-shadow-panel: inset 0 1px 0 rgba(255, 255, 255, 0.035);
	--hh-shadow-modal: 0 24px 80px rgba(0, 0, 0, 0.55);
	--hh-shell-bg-image: none;
	--hh-shell-bg-dim: 0.42;
	--hh-panel-alpha: 0.7;
	--hh-panel-alpha-low: 0.56;
	--hh-panel-opacity: var(--hh-panel-alpha);
	--hh-panel-blur: 12px;

	/* Semantic aliases used by shell components */
	--hh-bg: var(--hh-color-bg);
	--hh-panel-bg: var(--hh-surface-panel);
	--hh-border: var(--hh-border-subtle);
	--hh-text-primary: var(--hh-color-text-strong);
	--hh-text-secondary: var(--hh-color-text);
	--hh-text-muted: var(--hh-color-text-muted);
	--hh-hover-bg: var(--hh-accent-tint);
	--hh-active-bg: rgba(45, 240, 206, 0.1);

	/* Compatibility aliases for migrated Vue screens that still use legacy tokens. */
	--hh-bg-primary: rgba(5, 22, 25, var(--hh-panel-alpha));
	--hh-bg-secondary: rgba(8, 29, 33, var(--hh-panel-alpha-low));
	--hh-bg-hover: var(--hh-accent-tint);
	--hh-bg-active: rgba(45, 240, 206, 0.16);
	--hh-bg-input: rgba(2, 12, 16, 0.54);
	--hh-text-tertiary: var(--hh-color-text-muted);
	--hh-text-error: var(--hh-color-danger);
	--hh-bg-info-light: var(--hh-status-info-surface);
	--hh-status-success: var(--hh-status-success-text);
	--hh-status-danger: var(--hh-status-danger-text);
	--color-surface: rgba(5, 22, 25, var(--hh-panel-alpha));
	--color-bg: rgba(4, 18, 21, var(--hh-panel-alpha-low));
	--color-border: var(--hh-border-subtle);
	--color-text: var(--hh-color-text-strong);
	--color-text-secondary: var(--hh-color-text-muted);
	--color-primary: var(--hh-accent);
	--color-primary-subtle: var(--hh-accent-tint);
	--color-avatar-bg: rgba(45, 240, 206, 0.14);
	--color-success-bg: var(--hh-status-success-surface);
	--color-error-bg: var(--hh-status-danger-surface);
	--text-primary: var(--hh-color-text-strong);
	--text-secondary: var(--hh-color-text-muted);
	--bg-card: rgba(5, 22, 25, var(--hh-panel-alpha));
	--bg-hover: var(--hh-accent-tint);
	--bg-active: rgba(45, 240, 206, 0.16);
	--bg-input: rgba(2, 12, 16, 0.54);
	--border: var(--hh-border-subtle);
	--accent: var(--hh-accent);
	--accent-bg: var(--hh-accent-tint);
	--accent-fg: var(--hh-accent-contrast);
	--success: var(--hh-status-success-text);
	--warning: var(--hh-status-warning-text);
	--danger: var(--hh-status-danger-text);
	--radius-sm: var(--hh-radius-sm);
}

body {
	margin: 0;
	font-family: var(--hh-font-sans);
	background-color: var(--hh-color-bg);
	color: var(--hh-color-text);
}
```

### `frontend/src/styles/surfaces.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/styles/surfaces.css`
- Size bytes / Размер в байтах: `7160`
- Included characters / Включено символов: `7160`
- Truncated / Обрезано: `no`

```text
.hh-surface {
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, var(--hh-panel-alpha));
	backdrop-filter: blur(var(--hh-panel-blur));
	box-shadow: var(--hh-shadow-panel);
}

.hh-surface--raised {
	background: rgba(8, 29, 33, var(--hh-panel-alpha));
}

.hh-surface--deep {
	background: rgba(4, 18, 21, var(--hh-panel-alpha));
}

.panel,
.widget-frame,
.info-card {
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, var(--hh-panel-alpha));
	backdrop-filter: blur(var(--hh-panel-blur));
	box-shadow: var(--hh-shadow-panel);
	color: var(--hh-text-primary);
}

.panel input,
.panel textarea,
.widget-frame input,
.widget-frame textarea,
.settings-tree input,
.settings-tree textarea {
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-control);
	background: rgba(2, 12, 16, var(--hh-panel-alpha));
	color: var(--hh-text-primary);
}

.panel input::placeholder,
.panel textarea::placeholder,
.widget-frame input::placeholder,
.widget-frame textarea::placeholder,
.settings-tree input::placeholder,
.settings-tree textarea::placeholder {
	color: color-mix(in srgb, var(--hh-text-muted) 72%, transparent);
}

.view-header,
.communications-actionbar,
.telegram-command-header {
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, var(--hh-panel-alpha));
	backdrop-filter: blur(var(--hh-panel-blur));
	box-shadow: var(--hh-shadow-panel);
}

.view-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--hh-space-section);
	padding: var(--hh-space-panel);
}

.view-title-with-icon {
	display: flex;
	align-items: center;
	gap: 12px;
	min-width: 0;
}

.view-header h1,
.view-title-with-icon h1 {
	margin: 0;
	color: var(--hh-text-primary);
	font-size: 24px;
	font-weight: 760;
	line-height: 1.15;
}

.view-header p,
.view-title-with-icon p {
	margin: 4px 0 0;
	color: var(--hh-text-muted);
	font-size: 13px;
}

.primary-button,
.form-actions button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 6px;
	min-height: 34px;
	border: 1px solid var(--hh-border-accent);
	border-radius: var(--hh-radius-control);
	background: var(--hh-accent);
	color: var(--hh-accent-contrast);
	font-family: var(--hh-font-sans);
	font-size: 13px;
	font-weight: 720;
	padding: 0 12px;
	cursor: pointer;
	transition: filter 140ms ease, transform 140ms ease, opacity 140ms ease;
}

.primary-button:hover:not(:disabled),
.form-actions button:hover:not(:disabled) {
	filter: brightness(1.08);
}

.primary-button:disabled,
.form-actions button:disabled {
	cursor: not-allowed;
	opacity: 0.48;
}

.metric-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
	gap: var(--hh-layout-gap);
}

.metric-card {
	display: grid;
	gap: 8px;
	min-height: 92px;
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background: rgba(5, 22, 25, var(--hh-panel-alpha));
	backdrop-filter: blur(var(--hh-panel-blur));
	box-shadow: var(--hh-shadow-panel);
	padding: var(--hh-space-panel);
}

.metric-card span {
	color: var(--hh-text-muted);
	font-size: 11px;
	font-weight: 760;
	text-transform: uppercase;
}

.metric-card strong {
	color: var(--hh-text-primary);
	font-size: 22px;
	font-weight: 780;
	line-height: 1;
}

.metric-card small {
	color: var(--hh-accent-soft);
	font-size: 12px;
}

.three-pane,
.communications-grid {
	display: grid;
	gap: var(--hh-layout-gap);
	min-height: 0;
}

.three-pane {
	grid-template-columns: minmax(260px, 0.86fr) minmax(360px, 1.5fr) minmax(280px, 0.9fr);
}

.conversation-list,
.chat-pane,
.stacked-rail,
.info-card {
	min-width: 0;
	min-height: 0;
}

.stacked-rail {
	display: flex;
	flex-direction: column;
	gap: var(--hh-layout-gap);
	overflow-y: auto;
}

.info-card {
	display: grid;
	gap: 12px;
	padding: var(--hh-space-panel);
}

.info-card h2 {
	margin: 0;
	color: var(--hh-text-primary);
	font-size: 14px;
	font-weight: 760;
}

.empty-panel {
	display: flex;
	align-items: center;
	justify-content: center;
	min-height: 160px;
	padding: var(--hh-space-panel);
	color: var(--hh-text-muted);
	text-align: center;
}

.empty-panel.fill {
	flex: 1;
	min-height: 0;
}

.round-icon,
.hero-mark {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 38px;
	height: 38px;
	flex: 0 0 auto;
	border: 1px solid var(--hh-border-accent-soft);
	border-radius: var(--hh-radius-round);
	background: var(--hh-accent-tint);
	color: var(--hh-accent);
}

.hero-mark.small {
	width: 42px;
	height: 42px;
}

.round-icon.green {
	color: var(--hh-status-success-text);
	background: var(--hh-status-success-surface);
}

.round-icon.cyan {
	color: var(--hh-accent);
	background: var(--hh-accent-tint);
}

.round-icon.blue {
	color: var(--hh-status-info-text);
	background: var(--hh-status-info-surface);
}

.round-icon.ghost {
	color: var(--hh-text-muted);
	background: var(--hh-status-neutral-surface);
}

.local-search {
	display: flex;
	align-items: center;
	gap: 8px;
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-control);
	background: rgba(2, 12, 16, var(--hh-panel-alpha));
	padding: 8px 10px;
	color: var(--hh-text-muted);
}

.local-search input {
	width: 100%;
	min-width: 0;
	border: 0;
	background: transparent;
	color: var(--hh-text-primary);
	font: inherit;
	outline: 0;
}

.setup-form {
	display: grid;
	gap: 10px;
}

.setup-form.compact-form {
	grid-template-columns: repeat(2, minmax(0, 1fr));
}

.setup-form label {
	display: grid;
	gap: 5px;
	color: var(--hh-text-muted);
	font-size: 11px;
	font-weight: 700;
}

.setup-form label.wide,
.form-actions.wide {
	grid-column: 1 / -1;
}

.setup-form input,
.setup-form textarea {
	width: 100%;
	min-width: 0;
	padding: 8px 10px;
	font: inherit;
	resize: vertical;
}

.form-actions {
	display: flex;
	justify-content: flex-end;
	gap: 8px;
}

.health-row,
.evidence-row,
.detail-list li {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 12px;
	border: 1px solid var(--hh-border-muted);
	border-radius: var(--hh-radius-sm);
	background: rgba(2, 12, 16, var(--hh-panel-alpha-low));
	padding: 8px 10px;
}

.health-row span,
.detail-list,
.evidence-row p {
	color: var(--hh-text-muted);
}

.health-row strong,
.detail-list em,
.evidence-row strong {
	color: var(--hh-text-primary);
	font-style: normal;
}

.detail-list {
	display: grid;
	gap: 8px;
	margin: 0;
	padding: 0;
	list-style: none;
}

.evidence-row {
	display: grid;
	align-items: start;
	justify-content: stretch;
}

.evidence-row p {
	margin: 0;
}

.setup-state,
.inline-error {
	margin: 0;
	border-radius: var(--hh-radius-control);
	padding: 10px 12px;
	font-size: 13px;
}

.setup-state.success {
	border: 1px solid color-mix(in srgb, var(--hh-status-success-text) 30%, transparent);
	background: var(--hh-status-success-surface);
	color: var(--hh-status-success-text);
}

.inline-error {
	border: 1px solid color-mix(in srgb, var(--hh-status-danger-text) 30%, transparent);
	background: var(--hh-status-danger-surface);
	color: var(--hh-status-danger-text);
}
```

### `frontend/src/styles/theme-classes.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/styles/theme-classes.css`
- Size bytes / Размер в байтах: `6380`
- Included characters / Включено символов: `6380`
- Truncated / Обрезано: `no`

```text
/* Shell theme classes are applied to .viewport-guard by theme store. */

.theme-accent-teal {
	--hh-color-accent: #2df0ce;
	--hh-color-accent-strong: #25d8bd;
	--hh-color-accent-soft: #9ee8df;
	--hh-color-accent-contrast: #032522;
	--hh-border-accent-soft: rgba(45, 240, 206, 0.18);
	--hh-border-accent: rgba(45, 240, 206, 0.42);
	--hh-focus-ring: rgba(45, 240, 206, 0.62);
	--hh-accent-tint: rgba(45, 240, 206, 0.08);
	--hh-accent-control: rgba(25, 154, 132, 0.2);
}

.theme-accent-cyan {
	--hh-color-accent: #42ddff;
	--hh-color-accent-strong: #25bfe5;
	--hh-color-accent-soft: #a9efff;
	--hh-color-accent-contrast: #031f2a;
	--hh-border-accent-soft: rgba(66, 221, 255, 0.18);
	--hh-border-accent: rgba(66, 221, 255, 0.42);
	--hh-focus-ring: rgba(66, 221, 255, 0.62);
	--hh-accent-tint: rgba(66, 221, 255, 0.08);
	--hh-accent-control: rgba(35, 151, 180, 0.2);
}

.theme-accent-blue {
	--hh-color-accent: #61a7ff;
	--hh-color-accent-strong: #4b8ee5;
	--hh-color-accent-soft: #bed9ff;
	--hh-color-accent-contrast: #061a35;
	--hh-border-accent-soft: rgba(97, 167, 255, 0.18);
	--hh-border-accent: rgba(97, 167, 255, 0.42);
	--hh-focus-ring: rgba(97, 167, 255, 0.62);
	--hh-accent-tint: rgba(97, 167, 255, 0.08);
	--hh-accent-control: rgba(48, 104, 176, 0.2);
}

.theme-accent-violet {
	--hh-color-accent: #b98cff;
	--hh-color-accent-strong: #9b6ff0;
	--hh-color-accent-soft: #ddcaff;
	--hh-color-accent-contrast: #26143c;
	--hh-border-accent-soft: rgba(185, 140, 255, 0.18);
	--hh-border-accent: rgba(185, 140, 255, 0.42);
	--hh-focus-ring: rgba(185, 140, 255, 0.62);
	--hh-accent-tint: rgba(185, 140, 255, 0.08);
	--hh-accent-control: rgba(111, 70, 171, 0.2);
}

.theme-accent-amber {
	--hh-color-accent: #f2b84b;
	--hh-color-accent-strong: #d99c2e;
	--hh-color-accent-soft: #f8d99c;
	--hh-color-accent-contrast: #241806;
	--hh-border-accent-soft: rgba(242, 184, 75, 0.18);
	--hh-border-accent: rgba(242, 184, 75, 0.42);
	--hh-focus-ring: rgba(242, 184, 75, 0.62);
	--hh-accent-tint: rgba(242, 184, 75, 0.08);
	--hh-accent-control: rgba(154, 103, 30, 0.2);
}

.theme-accent-rose {
	--hh-color-accent: #ff7aa8;
	--hh-color-accent-strong: #e85e8e;
	--hh-color-accent-soft: #ffc5da;
	--hh-color-accent-contrast: #35101e;
	--hh-border-accent-soft: rgba(255, 122, 168, 0.18);
	--hh-border-accent: rgba(255, 122, 168, 0.42);
	--hh-focus-ring: rgba(255, 122, 168, 0.62);
	--hh-accent-tint: rgba(255, 122, 168, 0.08);
	--hh-accent-control: rgba(171, 64, 101, 0.2);
}

.theme-accent-teal,
.theme-accent-cyan,
.theme-accent-blue,
.theme-accent-violet,
.theme-accent-amber,
.theme-accent-rose {
	--hh-accent: var(--hh-color-accent);
	--hh-accent-strong: var(--hh-color-accent-strong);
	--hh-accent-soft: var(--hh-color-accent-soft);
	--hh-accent-contrast: var(--hh-color-accent-contrast);
}

.shell-bg-none { --hh-shell-bg-image: none; }
.shell-bg-network-mesh { --hh-shell-bg-image: url('/assets/shell-backgrounds/network-mesh.png'); }
.shell-bg-data-stream { --hh-shell-bg-image: url('/assets/shell-backgrounds/data-stream.png'); }
.shell-bg-node-frame { --hh-shell-bg-image: url('/assets/shell-backgrounds/node-frame.png'); }
.shell-bg-eclipse-grid { --hh-shell-bg-image: url('/assets/shell-backgrounds/eclipse-grid.png'); }
.shell-bg-dna-blueprint { --hh-shell-bg-image: url('/assets/shell-backgrounds/dna-blueprint.png'); }
.shell-bg-forest-network { --hh-shell-bg-image: url('/assets/shell-backgrounds/forest-network.png'); }
.shell-bg-forest-stream { --hh-shell-bg-image: url('/assets/shell-backgrounds/forest-stream.png'); }
.shell-bg-knowledge-map { --hh-shell-bg-image: url('/assets/shell-backgrounds/knowledge-map.png'); }
.shell-bg-rune-gold { --hh-shell-bg-image: url('/assets/shell-backgrounds/rune-gold.png'); }
.shell-bg-rune-teal { --hh-shell-bg-image: url('/assets/shell-backgrounds/rune-teal.png'); }

.shell-bg-brightness-30 { --hh-shell-bg-dim: 0.74; }
.shell-bg-brightness-40 { --hh-shell-bg-dim: 0.66; }
.shell-bg-brightness-50 { --hh-shell-bg-dim: 0.58; }
.shell-bg-brightness-60 { --hh-shell-bg-dim: 0.5; }
.shell-bg-brightness-70 { --hh-shell-bg-dim: 0.42; }
.shell-bg-brightness-80 { --hh-shell-bg-dim: 0.34; }
.shell-bg-brightness-90 { --hh-shell-bg-dim: 0.26; }
.shell-bg-brightness-100 { --hh-shell-bg-dim: 0.18; }

.panel-opacity-40 { --hh-panel-alpha: 0.4; --hh-panel-alpha-low: 0.32; --hh-panel-opacity: 0.4; }
.panel-opacity-50 { --hh-panel-alpha: 0.5; --hh-panel-alpha-low: 0.4; --hh-panel-opacity: 0.5; }
.panel-opacity-60 { --hh-panel-alpha: 0.6; --hh-panel-alpha-low: 0.48; --hh-panel-opacity: 0.6; }
.panel-opacity-70 { --hh-panel-alpha: 0.7; --hh-panel-alpha-low: 0.56; --hh-panel-opacity: 0.7; }
.panel-opacity-80 { --hh-panel-alpha: 0.8; --hh-panel-alpha-low: 0.64; --hh-panel-opacity: 0.8; }
.panel-opacity-90 { --hh-panel-alpha: 0.9; --hh-panel-alpha-low: 0.72; --hh-panel-opacity: 0.9; }
.panel-opacity-100 { --hh-panel-alpha: 1; --hh-panel-alpha-low: 0.9; --hh-panel-opacity: 1; }

.panel-blur-0 { --hh-panel-blur: 0px; }
.panel-blur-4 { --hh-panel-blur: 4px; }
.panel-blur-8 { --hh-panel-blur: 8px; }
.panel-blur-12 { --hh-panel-blur: 12px; }
.panel-blur-16 { --hh-panel-blur: 16px; }
.panel-blur-20 { --hh-panel-blur: 20px; }
.panel-blur-24 { --hh-panel-blur: 24px; }

.spacing-density-compact {
	--hh-density-scale: 0.84;
	--hh-space-panel: 10px;
	--hh-space-section: 12px;
	--hh-space-control-x: 9px;
}

.spacing-density-normal {
	--hh-density-scale: 1;
	--hh-space-panel: 14px;
	--hh-space-section: 16px;
	--hh-space-control-x: 12px;
}

.spacing-density-comfortable {
	--hh-density-scale: 1.14;
	--hh-space-panel: 18px;
	--hh-space-section: 20px;
	--hh-space-control-x: 14px;
}

.desktop-shell {
	background:
		linear-gradient(
			rgba(2, 9, 11, var(--hh-shell-bg-dim)),
			rgba(2, 9, 11, var(--hh-shell-bg-dim))
		),
		var(--hh-shell-bg-image),
		radial-gradient(circle at 72% 2%, rgba(23, 122, 121, 0.14), transparent 34%),
		linear-gradient(180deg, rgba(7, 28, 32, 0.88), rgba(2, 9, 11, 0.98) 46%),
		var(--hh-color-bg);
	background-position: center;
	background-repeat: no-repeat;
	background-size: cover, cover, auto, auto, auto;
}

.view-home,
.view-communications,
.view-timeline,
.view-persons,
.view-projects,
.view-tasks,
.view-calendar,
.view-documents,
.view-notes,
.view-knowledge,
.view-review,
.view-settings,
.view-agents,
.view-organizations,
.view-telegram,
.view-whatsapp {
	/* Reserved for per-view CSS overrides. */
}
```
