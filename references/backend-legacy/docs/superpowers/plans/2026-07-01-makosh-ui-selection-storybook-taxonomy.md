# Макошь UI Selection Storybook Taxonomy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first `Макошь UI / General` Storybook slice for standard selection controls and add richer shared UI selection components.

**Architecture:** Keep all selection controls under `frontend/src/shared/ui` with no domain imports, stores, API calls or provider vocabulary. Add small shared selection types, focused Vue components, tokenized CSS in `controls.css`, localized Storybook stories under component-specific `Макошь UI / General` paths, and boundary tests that enforce export/story coverage.

**Tech Stack:** Vue 3, TypeScript, Vite, Vitest, Storybook 10, Reka UI where it already fits, Макошь UI CSS tokens.

**Repository rule:** Do not create git commits unless the user explicitly asks. Use checkpoint staging only when requested during execution.

---

## File Structure

Create:

- `frontend/src/shared/ui/Selection.types.ts` - shared option, group and tree option types for selection controls.
- `frontend/src/shared/ui/SearchableSelect.vue` - single select with local search and clear affordance.
- `frontend/src/shared/ui/SearchableSelect.README.md` - component contract and boundary notes.
- `frontend/src/shared/ui/SearchableMultiSelect.vue` - multi-select with search, chips, select all and clear all.
- `frontend/src/shared/ui/SearchableMultiSelect.README.md` - component contract and boundary notes.
- `frontend/src/shared/ui/GroupedSelect.vue` - sectioned single-select for grouped flat options.
- `frontend/src/shared/ui/GroupedSelect.README.md` - component contract and boundary notes.
- `frontend/src/shared/ui/TreeSelect.vue` - hierarchical single-select popover over local tree data.
- `frontend/src/shared/ui/TreeSelect.README.md` - component contract and boundary notes.
- `frontend/src/shared/ui/Cascader.vue` - column-based hierarchical selection for deep trees.
- `frontend/src/shared/ui/Cascader.README.md` - component contract and boundary notes.
- `frontend/src/shared/ui/AsyncSelect.vue` - UI-only async-state wrapper around externally supplied options.
- `frontend/src/shared/ui/AsyncSelect.README.md` - component contract and boundary notes.
- `frontend/stories/ui/GeneralSelect.stories.ts` - `Макошь UI / General / Select` baseline story.
- `frontend/stories/ui/GeneralSearchableSelect.stories.ts` - `Макошь UI / General / Searchable Select` story.
- `frontend/stories/ui/GeneralMultiSelect.stories.ts` - `Макошь UI / General / Multi Select` baseline story.
- `frontend/stories/ui/GeneralSearchableMultiSelect.stories.ts` - `Макошь UI / General / Searchable Multi Select` story.
- `frontend/stories/ui/GeneralGroupedSelect.stories.ts` - `Макошь UI / General / Grouped Select` story.
- `frontend/stories/ui/GeneralTreeSelect.stories.ts` - `Макошь UI / General / Tree Select` story.
- `frontend/stories/ui/GeneralCascader.stories.ts` - `Макошь UI / General / Cascader` story.
- `frontend/stories/ui/GeneralAsyncSelect.stories.ts` - `Макошь UI / General / Async Select` state story.

Modify:

- `frontend/src/shared/ui/index.ts` - export new components and selection types.
- `frontend/src/shared/ui/inputs.boundary.test.ts` - enforce files, README docs, exports, UI-only source and Storybook coverage.
- `frontend/src/shared/ui/styles/controls.css` - add shared listbox, chips, grouped, tree-select, cascader and async-select styles.
- `frontend/stories/ui/storybook-i18n.ts` - add localized `selection` copy for `en`, `ru`, `es`.
- `docs/architecture/frontend-ui-components.md` - add the new selection controls and new Storybook hierarchy note.

Do not modify:

- domain files under `frontend/src/domains/*`;
- provider/integration files;
- backend code;
- generated files.

---

## Task 1: Add Shared Selection Types And Boundary Expectations

**Files:**
- Create: `frontend/src/shared/ui/Selection.types.ts`
- Modify: `frontend/src/shared/ui/index.ts`
- Modify: `frontend/src/shared/ui/inputs.boundary.test.ts`

- [ ] **Step 1: Add failing boundary expectations**

Modify `frontend/src/shared/ui/inputs.boundary.test.ts` and extend `inputComponents` with the new controls:

```ts
const inputComponents = [
	'SearchInput',
	'PasswordInput',
	'EmailInput',
	'NumberInput',
	'OTPInput',
	'Checkbox',
	'Radio',
	'RadioGroup',
	'Slider',
	'RangeSlider',
	'Form',
	'FormField',
	'FormLabel',
	'FormHint',
	'FormError',
	'CharacterCounter',
	'MultiSelect',
	'Combobox',
	'Autocomplete',
	'SearchableSelect',
	'SearchableMultiSelect',
	'GroupedSelect',
	'TreeSelect',
	'Cascader',
	'AsyncSelect',
	'ColorPicker',
	'DatePicker',
	'TimePicker',
	'DateTimePicker',
	'FilePicker',
	'FileDropZone'
]
```

Add a second story coverage list in the same file:

```ts
const selectionStoryFiles = [
	'GeneralSelect.stories.ts',
	'GeneralSearchableSelect.stories.ts',
	'GeneralMultiSelect.stories.ts',
	'GeneralSearchableMultiSelect.stories.ts',
	'GeneralGroupedSelect.stories.ts',
	'GeneralTreeSelect.stories.ts',
	'GeneralCascader.stories.ts',
	'GeneralAsyncSelect.stories.ts'
]
```

Replace the Storybook assertion with:

```ts
it('keeps selection components represented in General Storybook entries', () => {
	const storiesRoot = fileURLToPath(new URL('../../../stories/ui/', import.meta.url))

	for (const storyFile of selectionStoryFiles) {
		expect(existsSync(join(storiesRoot, storyFile)), storyFile).toBe(true)
		const storySource = readFileSync(join(storiesRoot, storyFile), 'utf8')
		expect(storySource).toContain('Макошь UI/General/')
	}
})
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cd frontend && pnpm test:unit -- src/shared/ui/inputs.boundary.test.ts
```

Expected: FAIL because the new component files and story files do not exist.

- [ ] **Step 3: Add shared type file**

Create `frontend/src/shared/ui/Selection.types.ts`:

```ts
export type SelectionTone = 'default' | 'muted' | 'warning' | 'danger' | 'success'

export interface SelectOption {
	value: string
	label: string
	description?: string
	disabled?: boolean
	icon?: string
	tone?: SelectionTone
}

export interface SelectGroup {
	id: string
	label: string
	options: SelectOption[]
}

export interface TreeSelectOption extends SelectOption {
	children?: TreeSelectOption[]
}

export interface SelectionSearchState {
	query: string
}
```

- [ ] **Step 4: Export shared types and planned components**

Modify `frontend/src/shared/ui/index.ts` near existing form exports:

```ts
export { default as SearchableSelect } from './SearchableSelect.vue'
export { default as SearchableMultiSelect } from './SearchableMultiSelect.vue'
export { default as GroupedSelect } from './GroupedSelect.vue'
export { default as TreeSelect } from './TreeSelect.vue'
export { default as Cascader } from './Cascader.vue'
export { default as AsyncSelect } from './AsyncSelect.vue'
```

Modify type exports near the other `export type` statements:

```ts
export type { SelectGroup, SelectOption, SelectionSearchState, SelectionTone, TreeSelectOption } from './Selection.types'
```

- [ ] **Step 5: Run test to verify the next failures are component/story files**

Run:

```bash
cd frontend && pnpm test:unit -- src/shared/ui/inputs.boundary.test.ts
```

Expected: FAIL with missing `.vue`, `.README.md` and story file assertions for the new controls.

---

## Task 2: Implement SearchableSelect

**Files:**
- Create: `frontend/src/shared/ui/SearchableSelect.vue`
- Create: `frontend/src/shared/ui/SearchableSelect.README.md`
- Modify: `frontend/src/shared/ui/styles/controls.css`

- [ ] **Step 1: Add component**

Create `frontend/src/shared/ui/SearchableSelect.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { SelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	clearable?: boolean
	emptyLabel?: string
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	searchPlaceholder: 'Search...',
	disabled: false,
	readonly: false,
	clearable: true,
	emptyLabel: 'No options'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	search: [query: string]
	select: [option: SelectOption]
	clear: []
	open: []
	close: []
}>()

const isOpen = ref(false)
const query = ref('')
const activeIndex = ref(0)

const classes = computed(() => ['makosh-searchable-select', props.class])
const selectedOption = computed(() => props.options.find((option) => option.value === props.modelValue))
const enabledOptions = computed(() => props.options.filter((option) => !option.disabled))
const filteredOptions = computed(() => {
	const normalizedQuery = query.value.trim().toLocaleLowerCase()
	const source = normalizedQuery
		? enabledOptions.value.filter((option) =>
			option.label.toLocaleLowerCase().includes(normalizedQuery) ||
			option.value.toLocaleLowerCase().includes(normalizedQuery) ||
			option.description?.toLocaleLowerCase().includes(normalizedQuery)
		)
		: enabledOptions.value
	return source
})
const activeOption = computed(() => filteredOptions.value[activeIndex.value])
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)
const canClear = computed(() => props.clearable && Boolean(props.modelValue) && !props.disabled && !props.readonly)

function openList(): void {
	if (props.disabled || props.readonly) {
		return
	}
	isOpen.value = true
	activeIndex.value = 0
	emit('open')
}

function closeList(): void {
	isOpen.value = false
	emit('close')
}

function setQuery(value: string): void {
	query.value = value
	activeIndex.value = 0
	emit('search', value)
}

function selectOption(option: SelectOption | undefined): void {
	if (!option || option.disabled) {
		return
	}
	emit('update:modelValue', option.value)
	emit('select', option)
	closeList()
}

function clearSelection(): void {
	emit('update:modelValue', '')
	emit('clear')
}

function handleKeydown(event: KeyboardEvent): void {
	if (!isOpen.value && ['ArrowDown', 'Enter', ' '].includes(event.key)) {
		event.preventDefault()
		openList()
		return
	}
	if (event.key === 'Escape') {
		closeList()
		return
	}
	if (!isOpen.value || filteredOptions.value.length === 0) {
		return
	}
	if (event.key === 'ArrowDown') {
		event.preventDefault()
		activeIndex.value = Math.min(activeIndex.value + 1, filteredOptions.value.length - 1)
	}
	if (event.key === 'ArrowUp') {
		event.preventDefault()
		activeIndex.value = Math.max(activeIndex.value - 1, 0)
	}
	if (event.key === 'Home') {
		event.preventDefault()
		activeIndex.value = 0
	}
	if (event.key === 'End') {
		event.preventDefault()
		activeIndex.value = filteredOptions.value.length - 1
	}
	if (event.key === 'Enter') {
		event.preventDefault()
		selectOption(activeOption.value)
	}
}
</script>

<template>
	<div :class="classes" @keydown="handleKeydown">
		<button
			class="makosh-searchable-select__trigger"
			type="button"
			:aria-expanded="isOpen"
			:aria-label="ariaLabel ?? placeholder"
			:disabled="disabled"
			@click="isOpen ? closeList() : openList()"
		>
			<span class="makosh-searchable-select__value" :class="{ 'makosh-searchable-select__value--placeholder': !selectedOption }">
				{{ displayLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" class="makosh-searchable-select__chevron" aria-hidden="true" />
		</button>
		<button
			v-if="canClear"
			class="makosh-searchable-select__clear"
			type="button"
			:aria-label="'Clear selection'"
			@click.stop="clearSelection"
		>
			<Icon icon="tabler:x" size="0.875rem" aria-hidden="true" />
		</button>
		<div v-if="isOpen" class="makosh-searchable-select__popover">
			<input
				class="makosh-native-control makosh-searchable-select__search"
				:placeholder="searchPlaceholder"
				:value="query"
				@input="setQuery(($event.target as HTMLInputElement).value)"
			/>
			<ul class="makosh-selection-listbox" role="listbox">
				<li
					v-for="(option, index) in filteredOptions"
					:key="option.value"
					class="makosh-selection-option"
					:aria-selected="option.value === modelValue"
					role="option"
					@mousedown.prevent="selectOption(option)"
				>
					<Icon v-if="option.icon" :icon="option.icon" size="1rem" class="makosh-selection-option__icon" aria-hidden="true" />
					<span class="makosh-selection-option__body">
						<span class="makosh-selection-option__label">{{ option.label }}</span>
						<span v-if="option.description" class="makosh-selection-option__description">{{ option.description }}</span>
					</span>
					<Icon v-if="index === activeIndex || option.value === modelValue" icon="tabler:check" size="0.875rem" aria-hidden="true" />
				</li>
				<li v-if="filteredOptions.length === 0" class="makosh-selection-empty">{{ emptyLabel }}</li>
			</ul>
		</div>
	</div>
</template>
```

- [ ] **Step 2: Add README**

Create `frontend/src/shared/ui/SearchableSelect.README.md`:

```md
# SearchableSelect

Single-value select with local search, clear action and keyboard navigation.

Boundary rules:

- accepts local `SelectOption[]`;
- emits search and selection intent;
- does not fetch data;
- does not import domains, integrations, stores or query clients.
```

- [ ] **Step 3: Add shared CSS for SearchableSelect**

Append to `frontend/src/shared/ui/styles/controls.css`:

```css
.makosh-searchable-select {
	position: relative;
	display: grid;
	gap: var(--h-spacing-2);
}

.makosh-searchable-select__trigger {
	display: inline-flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--h-spacing-2);
	width: 100%;
	height: var(--h-control-height-md);
	border: 1px solid var(--h-color-border);
	border-radius: var(--h-radius-md);
	background: var(--h-color-surface);
	color: var(--h-color-text-strong);
	font-family: var(--h-font-sans);
	font-size: 13px;
	padding: 0 var(--h-control-padding-x);
	cursor: pointer;
}

.makosh-searchable-select__trigger:focus-visible {
	box-shadow: var(--h-focus-ring);
	outline: 0;
}

.makosh-searchable-select__value {
	min-width: 0;
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.makosh-searchable-select__value--placeholder,
.makosh-searchable-select__chevron {
	color: var(--h-color-text-muted);
}

.makosh-searchable-select__clear {
	position: absolute;
	top: 5px;
	right: 32px;
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 24px;
	height: 24px;
	border: 0;
	border-radius: var(--h-radius-sm);
	background: transparent;
	color: var(--h-color-text-muted);
	cursor: pointer;
}

.makosh-searchable-select__clear:hover {
	background: var(--h-color-surface-hover);
	color: var(--h-color-text-strong);
}

.makosh-searchable-select__popover {
	position: absolute;
	z-index: var(--h-z-popover);
	top: calc(100% + var(--h-spacing-1));
	right: 0;
	left: 0;
	display: grid;
	gap: var(--h-spacing-1);
	border: 1px solid var(--h-color-border);
	border-radius: var(--h-radius-md);
	background: var(--h-color-surface-raised);
	box-shadow: var(--h-shadow-lg);
	padding: var(--h-spacing-2);
}

.makosh-searchable-select__search {
	height: var(--h-control-height-sm);
}

.makosh-selection-listbox {
	display: grid;
	max-height: calc(var(--h-control-height-md) * 6);
	overflow: auto;
	list-style: none;
	margin: 0;
	padding: 0;
}

.makosh-selection-option {
	display: flex;
	align-items: center;
	gap: var(--h-spacing-2);
	border-radius: var(--h-radius-sm);
	color: var(--h-color-text-soft);
	font-size: 13px;
	line-height: 1.35;
	padding: var(--h-spacing-2);
	cursor: pointer;
}

.makosh-selection-option:hover,
.makosh-selection-option[aria-selected='true'] {
	background: var(--h-color-surface-hover);
	color: var(--h-color-text-strong);
}

.makosh-selection-option__body {
	display: grid;
	gap: 2px;
	min-width: 0;
}

.makosh-selection-option__label,
.makosh-selection-option__description {
	overflow: hidden;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.makosh-selection-option__description,
.makosh-selection-empty {
	color: var(--h-color-text-muted);
	font-size: 12px;
}

.makosh-selection-empty {
	padding: var(--h-spacing-2);
}
```

- [ ] **Step 4: Run targeted typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS for `SearchableSelect.vue`.

---

## Task 3: Implement SearchableMultiSelect

**Files:**
- Create: `frontend/src/shared/ui/SearchableMultiSelect.vue`
- Create: `frontend/src/shared/ui/SearchableMultiSelect.README.md`
- Modify: `frontend/src/shared/ui/styles/controls.css`

- [ ] **Step 1: Add component**

Create `frontend/src/shared/ui/SearchableMultiSelect.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { SelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string[]
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	emptyLabel?: string
	selectAllLabel?: string
	clearAllLabel?: string
	class?: string
}>(), {
	modelValue: () => [],
	options: () => [],
	placeholder: 'Select options',
	searchPlaceholder: 'Search...',
	disabled: false,
	readonly: false,
	emptyLabel: 'No options',
	selectAllLabel: 'Select all',
	clearAllLabel: 'Clear all'
})

const emit = defineEmits<{
	'update:modelValue': [value: string[]]
	search: [query: string]
	select: [option: SelectOption]
	clear: []
	open: []
	close: []
}>()

const isOpen = ref(false)
const query = ref('')

const selectedValues = computed(() => new Set(props.modelValue))
const enabledOptions = computed(() => props.options.filter((option) => !option.disabled))
const selectedOptions = computed(() => props.options.filter((option) => selectedValues.value.has(option.value)))
const filteredOptions = computed(() => {
	const normalizedQuery = query.value.trim().toLocaleLowerCase()
	return normalizedQuery
		? enabledOptions.value.filter((option) =>
			option.label.toLocaleLowerCase().includes(normalizedQuery) ||
			option.value.toLocaleLowerCase().includes(normalizedQuery) ||
			option.description?.toLocaleLowerCase().includes(normalizedQuery)
		)
		: enabledOptions.value
})
const triggerLabel = computed(() => selectedOptions.value.length ? `${selectedOptions.value.length} selected` : props.placeholder)

function openList(): void {
	if (props.disabled || props.readonly) {
		return
	}
	isOpen.value = true
	emit('open')
}

function closeList(): void {
	isOpen.value = false
	emit('close')
}

function setQuery(value: string): void {
	query.value = value
	emit('search', value)
}

function toggleOption(option: SelectOption): void {
	const next = new Set(props.modelValue)
	if (next.has(option.value)) {
		next.delete(option.value)
	} else {
		next.add(option.value)
		emit('select', option)
	}
	emit('update:modelValue', Array.from(next))
}

function removeOption(value: string): void {
	emit('update:modelValue', props.modelValue.filter((selectedValue) => selectedValue !== value))
}

function selectAll(): void {
	emit('update:modelValue', enabledOptions.value.map((option) => option.value))
}

function clearAll(): void {
	emit('update:modelValue', [])
	emit('clear')
}
</script>

<template>
	<div class="makosh-searchable-multi-select" :class="class">
		<button
			class="makosh-searchable-select__trigger"
			type="button"
			:aria-expanded="isOpen"
			:aria-label="ariaLabel ?? placeholder"
			:disabled="disabled"
			@click="isOpen ? closeList() : openList()"
		>
			<span class="makosh-searchable-select__value" :class="{ 'makosh-searchable-select__value--placeholder': selectedOptions.length === 0 }">
				{{ triggerLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" class="makosh-searchable-select__chevron" aria-hidden="true" />
		</button>
		<div v-if="selectedOptions.length" class="makosh-selection-chips" aria-live="polite">
			<span v-for="option in selectedOptions" :key="option.value" class="makosh-selection-chip">
				<span>{{ option.label }}</span>
				<button type="button" :aria-label="`Remove ${option.label}`" @click="removeOption(option.value)">
					<Icon icon="tabler:x" size="0.75rem" aria-hidden="true" />
				</button>
			</span>
		</div>
		<div v-if="isOpen" class="makosh-searchable-select__popover">
			<input
				class="makosh-native-control makosh-searchable-select__search"
				:placeholder="searchPlaceholder"
				:value="query"
				@input="setQuery(($event.target as HTMLInputElement).value)"
			/>
			<div class="makosh-selection-actions">
				<button type="button" @click="selectAll">{{ selectAllLabel }}</button>
				<button type="button" @click="clearAll">{{ clearAllLabel }}</button>
			</div>
			<ul class="makosh-selection-listbox" role="listbox" aria-multiselectable="true">
				<li
					v-for="option in filteredOptions"
					:key="option.value"
					class="makosh-selection-option"
					:aria-selected="selectedValues.has(option.value)"
					role="option"
					@mousedown.prevent="toggleOption(option)"
				>
					<Icon :icon="selectedValues.has(option.value) ? 'tabler:square-check-filled' : 'tabler:square'" size="1rem" aria-hidden="true" />
					<span class="makosh-selection-option__body">
						<span class="makosh-selection-option__label">{{ option.label }}</span>
						<span v-if="option.description" class="makosh-selection-option__description">{{ option.description }}</span>
					</span>
				</li>
				<li v-if="filteredOptions.length === 0" class="makosh-selection-empty">{{ emptyLabel }}</li>
			</ul>
		</div>
	</div>
</template>
```

- [ ] **Step 2: Add README**

Create `frontend/src/shared/ui/SearchableMultiSelect.README.md`:

```md
# SearchableMultiSelect

Multi-value select with local search, removable chips, select-all and clear-all
actions.

Boundary rules:

- accepts local `SelectOption[]`;
- emits selected values;
- does not fetch data;
- does not own domain validation.
```

- [ ] **Step 3: Add chip/action CSS**

Append to `frontend/src/shared/ui/styles/controls.css`:

```css
.makosh-searchable-multi-select {
	position: relative;
	display: grid;
	gap: var(--h-spacing-2);
}

.makosh-selection-chips {
	display: flex;
	flex-wrap: wrap;
	gap: var(--h-spacing-1);
}

.makosh-selection-chip {
	display: inline-flex;
	align-items: center;
	gap: var(--h-spacing-1);
	min-height: var(--h-control-height-sm);
	border: 1px solid var(--h-color-border);
	border-radius: var(--h-radius-pill);
	background: var(--h-color-surface-raised);
	color: var(--h-color-text-soft);
	font-size: 12px;
	font-weight: 650;
	padding: 0 var(--h-spacing-1) 0 var(--h-spacing-2);
}

.makosh-selection-chip button,
.makosh-selection-actions button {
	border: 0;
	border-radius: var(--h-radius-sm);
	background: transparent;
	color: var(--h-color-text-muted);
	cursor: pointer;
}

.makosh-selection-chip button:hover,
.makosh-selection-actions button:hover {
	background: var(--h-color-surface-hover);
	color: var(--h-color-text-strong);
}

.makosh-selection-actions {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--h-spacing-2);
	font-size: 12px;
}
```

- [ ] **Step 4: Run typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS for `SearchableMultiSelect.vue`.

---

## Task 4: Implement GroupedSelect And AsyncSelect

**Files:**
- Create: `frontend/src/shared/ui/GroupedSelect.vue`
- Create: `frontend/src/shared/ui/GroupedSelect.README.md`
- Create: `frontend/src/shared/ui/AsyncSelect.vue`
- Create: `frontend/src/shared/ui/AsyncSelect.README.md`
- Modify: `frontend/src/shared/ui/styles/controls.css`

- [ ] **Step 1: Add GroupedSelect**

Create `frontend/src/shared/ui/GroupedSelect.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue'
import SearchableSelect from './SearchableSelect.vue'
import type { SelectGroup, SelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	groups?: SelectGroup[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	disabled?: boolean
	emptyLabel?: string
	class?: string
}>(), {
	modelValue: '',
	groups: () => [],
	placeholder: 'Select...',
	searchPlaceholder: 'Search...',
	disabled: false,
	emptyLabel: 'No options'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	search: [query: string]
	select: [option: SelectOption]
	clear: []
}>()

const options = computed(() => props.groups.flatMap((group) =>
	group.options.map((option) => ({
		...option,
		description: option.description ?? group.label
	}))
))
</script>

<template>
	<SearchableSelect
		:aria-label="ariaLabel"
		:class="['makosh-grouped-select', class]"
		:disabled="disabled"
		:empty-label="emptyLabel"
		:model-value="modelValue"
		:options="options"
		:placeholder="placeholder"
		:search-placeholder="searchPlaceholder"
		@clear="emit('clear')"
		@search="emit('search', $event)"
		@select="emit('select', $event)"
		@update:model-value="emit('update:modelValue', $event)"
	/>
</template>
```

- [ ] **Step 2: Add GroupedSelect README**

Create `frontend/src/shared/ui/GroupedSelect.README.md`:

```md
# GroupedSelect

Flat grouped select for sectioned option sets.

Use `GroupedSelect` when groups are labels, not navigable hierarchy. Use
`TreeSelect` or `Cascader` when parent-child semantics matter.
```

- [ ] **Step 3: Add AsyncSelect**

Create `frontend/src/shared/ui/AsyncSelect.vue`:

```vue
<script setup lang="ts">
import SearchableSelect from './SearchableSelect.vue'
import Button from './Button.vue'
import Spinner from './Spinner.vue'
import type { SelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	disabled?: boolean
	loading?: boolean
	error?: string
	emptyLabel?: string
	retryLabel?: string
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	searchPlaceholder: 'Search...',
	disabled: false,
	loading: false,
	error: '',
	emptyLabel: 'No options',
	retryLabel: 'Retry'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	search: [query: string]
	select: [option: SelectOption]
	clear: []
	retry: []
}>()
</script>

<template>
	<div class="makosh-async-select" :class="class">
		<SearchableSelect
			:aria-label="ariaLabel"
			:disabled="disabled || loading || Boolean(error)"
			:empty-label="emptyLabel"
			:model-value="modelValue"
			:options="options"
			:placeholder="placeholder"
			:search-placeholder="searchPlaceholder"
			@clear="emit('clear')"
			@search="emit('search', $event)"
			@select="emit('select', $event)"
			@update:model-value="emit('update:modelValue', $event)"
		/>
		<div v-if="loading" class="makosh-async-select__state" role="status">
			<Spinner size="sm" />
			<span>Loading options</span>
		</div>
		<div v-if="error" class="makosh-async-select__state makosh-async-select__state--error" role="alert">
			<span>{{ error }}</span>
			<Button size="sm" variant="outline" @click="emit('retry')">{{ retryLabel }}</Button>
		</div>
	</div>
</template>
```

- [ ] **Step 4: Add AsyncSelect README**

Create `frontend/src/shared/ui/AsyncSelect.README.md`:

```md
# AsyncSelect

UI-only async state wrapper for externally loaded select options.

It receives `loading`, `error` and `options` from a parent. It never performs a
network request and emits `search` / `retry` intent only.
```

- [ ] **Step 5: Add async state CSS**

Append to `frontend/src/shared/ui/styles/controls.css`:

```css
.makosh-async-select {
	display: grid;
	gap: var(--h-spacing-2);
}

.makosh-async-select__state {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--h-spacing-2);
	border: 1px solid var(--h-color-border);
	border-radius: var(--h-radius-md);
	background: var(--h-color-surface-raised);
	color: var(--h-color-text-muted);
	font-size: 12px;
	padding: var(--h-spacing-2);
}

.makosh-async-select__state--error {
	border-color: color-mix(in srgb, var(--h-color-danger) 50%, var(--h-color-border));
	color: var(--h-color-danger);
}
```

- [ ] **Step 6: Run typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS for `GroupedSelect.vue` and `AsyncSelect.vue`.

---

## Task 5: Implement TreeSelect

**Files:**
- Create: `frontend/src/shared/ui/TreeSelect.vue`
- Create: `frontend/src/shared/ui/TreeSelect.README.md`
- Modify: `frontend/src/shared/ui/styles/controls.css`

- [ ] **Step 1: Add TreeSelect**

Create `frontend/src/shared/ui/TreeSelect.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { TreeSelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: TreeSelectOption[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	emptyLabel?: string
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	disabled: false,
	emptyLabel: 'No options'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: TreeSelectOption]
	open: []
	close: []
}>()

const isOpen = ref(false)
const expanded = ref<string[]>([])

const selectedOption = computed(() => findOption(props.options, props.modelValue))
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)

function findOption(options: TreeSelectOption[], value: string | undefined): TreeSelectOption | undefined {
	for (const option of options) {
		if (option.value === value) {
			return option
		}
		const child = option.children ? findOption(option.children, value) : undefined
		if (child) {
			return child
		}
	}
	return undefined
}

function toggleOpen(): void {
	if (props.disabled) {
		return
	}
	isOpen.value = !isOpen.value
	emit(isOpen.value ? 'open' : 'close')
}

function toggleExpanded(option: TreeSelectOption): void {
	const next = new Set(expanded.value)
	if (next.has(option.value)) {
		next.delete(option.value)
	} else {
		next.add(option.value)
	}
	expanded.value = Array.from(next)
}

function selectOption(option: TreeSelectOption): void {
	if (option.disabled) {
		return
	}
	emit('update:modelValue', option.value)
	emit('select', option)
	isOpen.value = false
}
</script>

<template>
	<div class="makosh-tree-select" :class="class">
		<button
			class="makosh-searchable-select__trigger"
			type="button"
			:aria-expanded="isOpen"
			:aria-label="ariaLabel ?? placeholder"
			:disabled="disabled"
			@click="toggleOpen"
		>
			<span class="makosh-searchable-select__value" :class="{ 'makosh-searchable-select__value--placeholder': !selectedOption }">
				{{ displayLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" aria-hidden="true" />
		</button>
		<div v-if="isOpen" class="makosh-searchable-select__popover">
			<ul v-if="options.length" class="makosh-tree-select__tree" role="tree">
				<li v-for="option in options" :key="option.value" class="makosh-tree-select__item" role="treeitem">
					<button class="makosh-tree-select__row" type="button" @click="option.children?.length ? toggleExpanded(option) : selectOption(option)">
						<Icon v-if="option.children?.length" :icon="expanded.includes(option.value) ? 'tabler:chevron-down' : 'tabler:chevron-right'" size="0.875rem" aria-hidden="true" />
						<span v-else class="makosh-tree-select__spacer"></span>
						<span>{{ option.label }}</span>
					</button>
					<ul v-if="option.children?.length && expanded.includes(option.value)" class="makosh-tree-select__children" role="group">
						<li v-for="child in option.children" :key="child.value">
							<button class="makosh-tree-select__row" type="button" @click="selectOption(child)">
								<span class="makosh-tree-select__spacer"></span>
								<span>{{ child.label }}</span>
							</button>
						</li>
					</ul>
				</li>
			</ul>
			<div v-else class="makosh-selection-empty">{{ emptyLabel }}</div>
		</div>
	</div>
</template>
```

- [ ] **Step 2: Add README**

Create `frontend/src/shared/ui/TreeSelect.README.md`:

```md
# TreeSelect

Single-value select for local hierarchical option trees.

Use it when parent-child structure matters and a compact dropdown tree is enough.
Use `Cascader` when a deep hierarchy is easier to scan by columns.
```

- [ ] **Step 3: Add TreeSelect CSS**

Append to `frontend/src/shared/ui/styles/controls.css`:

```css
.makosh-tree-select {
	position: relative;
}

.makosh-tree-select__tree,
.makosh-tree-select__children {
	display: grid;
	gap: 2px;
	list-style: none;
	margin: 0;
	padding: 0;
}

.makosh-tree-select__children {
	padding-left: var(--h-spacing-4);
}

.makosh-tree-select__row {
	display: flex;
	align-items: center;
	gap: var(--h-spacing-2);
	width: 100%;
	border: 0;
	border-radius: var(--h-radius-sm);
	background: transparent;
	color: var(--h-color-text-soft);
	font-family: var(--h-font-sans);
	font-size: 13px;
	padding: var(--h-spacing-2);
	text-align: left;
	cursor: pointer;
}

.makosh-tree-select__row:hover {
	background: var(--h-color-surface-hover);
	color: var(--h-color-text-strong);
}

.makosh-tree-select__spacer {
	width: 14px;
}
```

- [ ] **Step 4: Run typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS for `TreeSelect.vue`.

---

## Task 6: Implement Cascader

**Files:**
- Create: `frontend/src/shared/ui/Cascader.vue`
- Create: `frontend/src/shared/ui/Cascader.README.md`
- Modify: `frontend/src/shared/ui/styles/controls.css`

- [ ] **Step 1: Add Cascader**

Create `frontend/src/shared/ui/Cascader.vue`:

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'
import type { TreeSelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: TreeSelectOption[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: TreeSelectOption]
}>()

const isOpen = ref(false)
const path = ref<TreeSelectOption[]>([])

const selectedOption = computed(() => path.value.at(-1))
const columns = computed(() => {
	const result: TreeSelectOption[][] = [props.options]
	for (const option of path.value) {
		if (option.children?.length) {
			result.push(option.children)
		}
	}
	return result
})
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)

function choose(option: TreeSelectOption, columnIndex: number): void {
	path.value = [...path.value.slice(0, columnIndex), option]
	if (!option.children?.length) {
		emit('update:modelValue', option.value)
		emit('select', option)
		isOpen.value = false
	}
}
</script>

<template>
	<div class="makosh-cascader" :class="class">
		<button
			class="makosh-searchable-select__trigger"
			type="button"
			:aria-expanded="isOpen"
			:aria-label="ariaLabel ?? placeholder"
			:disabled="disabled"
			@click="isOpen = !isOpen"
		>
			<span class="makosh-searchable-select__value" :class="{ 'makosh-searchable-select__value--placeholder': !selectedOption }">
				{{ displayLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" aria-hidden="true" />
		</button>
		<div v-if="isOpen" class="makosh-cascader__popover">
			<ul v-for="(column, columnIndex) in columns" :key="columnIndex" class="makosh-cascader__column">
				<li v-for="option in column" :key="option.value">
					<button class="makosh-cascader__option" type="button" @click="choose(option, columnIndex)">
						<span>{{ option.label }}</span>
						<Icon v-if="option.children?.length" icon="tabler:chevron-right" size="0.875rem" aria-hidden="true" />
					</button>
				</li>
			</ul>
		</div>
	</div>
</template>
```

- [ ] **Step 2: Add README**

Create `frontend/src/shared/ui/Cascader.README.md`:

```md
# Cascader

Column-based hierarchical selector for local tree data.

Use it for deep structured choices where a single expanded tree becomes harder
to scan than progressive columns.
```

- [ ] **Step 3: Add Cascader CSS**

Append to `frontend/src/shared/ui/styles/controls.css`:

```css
.makosh-cascader {
	position: relative;
}

.makosh-cascader__popover {
	position: absolute;
	z-index: var(--h-z-popover);
	top: calc(100% + var(--h-spacing-1));
	left: 0;
	display: flex;
	max-width: min(720px, 90vw);
	border: 1px solid var(--h-color-border);
	border-radius: var(--h-radius-md);
	background: var(--h-color-surface-raised);
	box-shadow: var(--h-shadow-lg);
	overflow: auto;
}

.makosh-cascader__column {
	display: grid;
	align-content: start;
	min-width: 180px;
	max-height: 280px;
	overflow: auto;
	list-style: none;
	margin: 0;
	padding: var(--h-spacing-1);
	border-right: 1px solid var(--h-color-border);
}

.makosh-cascader__column:last-child {
	border-right: 0;
}

.makosh-cascader__option {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: var(--h-spacing-2);
	width: 100%;
	border: 0;
	border-radius: var(--h-radius-sm);
	background: transparent;
	color: var(--h-color-text-soft);
	font-family: var(--h-font-sans);
	font-size: 13px;
	padding: var(--h-spacing-2);
	text-align: left;
	cursor: pointer;
}

.makosh-cascader__option:hover {
	background: var(--h-color-surface-hover);
	color: var(--h-color-text-strong);
}
```

- [ ] **Step 4: Run typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS for `Cascader.vue`.

---

## Task 7: Add General Selection Storybook Entries And i18n

**Files:**
- Create: `frontend/stories/ui/GeneralSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralSearchableSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralMultiSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralSearchableMultiSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralGroupedSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralTreeSelect.stories.ts`
- Create: `frontend/stories/ui/GeneralCascader.stories.ts`
- Create: `frontend/stories/ui/GeneralAsyncSelect.stories.ts`
- Modify: `frontend/stories/ui/storybook-i18n.ts`

- [ ] **Step 1: Add `selection` copy to English locale**

In `frontend/stories/ui/storybook-i18n.ts`, add this sibling next to `form` in the English object:

```ts
selection: {
	title: 'Selection controls',
	select: 'Select',
	searchableSelect: 'Searchable select',
	multiSelect: 'Multi select',
	searchableMultiSelect: 'Searchable multi select',
	groupedSelect: 'Grouped select',
	treeSelect: 'Tree select',
	cascader: 'Cascader',
	asyncSelect: 'Async select',
	placeholder: 'Choose a context',
	searchPlaceholder: 'Search local options',
	empty: 'No matching options',
	selectAll: 'Select all',
	clearAll: 'Clear all',
	retry: 'Retry',
	loading: 'Loading options',
	error: 'Could not load options',
	options: [
		{ value: 'communications', label: 'Communications', description: 'Canonical messages and source evidence', icon: 'tabler:messages' },
		{ value: 'knowledge', label: 'Knowledge', description: 'Reviewed facts and observations', icon: 'tabler:bulb' },
		{ value: 'projects', label: 'Projects', description: 'Bounded work context', icon: 'tabler:briefcase' },
		{ value: 'documents', label: 'Documents', description: 'Versioned evidence artifacts', icon: 'tabler:file-text' }
	],
	groups: [
		{ id: 'memory', label: 'Memory', options: [
			{ value: 'communications', label: 'Communications' },
			{ value: 'knowledge', label: 'Knowledge' }
		] },
		{ id: 'work', label: 'Work', options: [
			{ value: 'projects', label: 'Projects' },
			{ value: 'documents', label: 'Documents' }
		] }
	],
	tree: [
		{ value: 'memory', label: 'Memory', children: [
			{ value: 'communications', label: 'Communications' },
			{ value: 'knowledge', label: 'Knowledge' }
		] },
		{ value: 'work', label: 'Work', children: [
			{ value: 'projects', label: 'Projects' },
			{ value: 'documents', label: 'Documents' }
		] }
	]
}
```

- [ ] **Step 2: Add matching Russian and Spanish `selection` copy**

Add Russian and Spanish blocks with matching keys. Keep option `value` fields
unchanged:

```ts
selection: {
	title: 'Контролы выбора',
	select: 'Выбор',
	searchableSelect: 'Выбор с поиском',
	multiSelect: 'Множественный выбор',
	searchableMultiSelect: 'Множественный выбор с поиском',
	groupedSelect: 'Группированный выбор',
	treeSelect: 'Иерархический выбор',
	cascader: 'Каскадный выбор',
	asyncSelect: 'Асинхронный выбор',
	placeholder: 'Выберите контекст',
	searchPlaceholder: 'Искать локальные варианты',
	empty: 'Нет подходящих вариантов',
	selectAll: 'Выбрать все',
	clearAll: 'Очистить все',
	retry: 'Повторить',
	loading: 'Загрузка вариантов',
	error: 'Не удалось загрузить варианты',
	options: [
		{ value: 'communications', label: 'Коммуникации', description: 'Канонические сообщения и исходные evidence', icon: 'tabler:messages' },
		{ value: 'knowledge', label: 'Знания', description: 'Проверенные факты и наблюдения', icon: 'tabler:bulb' },
		{ value: 'projects', label: 'Проекты', description: 'Ограниченный рабочий контекст', icon: 'tabler:briefcase' },
		{ value: 'documents', label: 'Документы', description: 'Версионированные evidence-артефакты', icon: 'tabler:file-text' }
	],
	groups: [
		{ id: 'memory', label: 'Память', options: [
			{ value: 'communications', label: 'Коммуникации' },
			{ value: 'knowledge', label: 'Знания' }
		] },
		{ id: 'work', label: 'Работа', options: [
			{ value: 'projects', label: 'Проекты' },
			{ value: 'documents', label: 'Документы' }
		] }
	],
	tree: [
		{ value: 'memory', label: 'Память', children: [
			{ value: 'communications', label: 'Коммуникации' },
			{ value: 'knowledge', label: 'Знания' }
		] },
		{ value: 'work', label: 'Работа', children: [
			{ value: 'projects', label: 'Проекты' },
			{ value: 'documents', label: 'Документы' }
		] }
	]
}
```

```ts
selection: {
	title: 'Controles de selección',
	select: 'Selección',
	searchableSelect: 'Selección con búsqueda',
	multiSelect: 'Selección múltiple',
	searchableMultiSelect: 'Selección múltiple con búsqueda',
	groupedSelect: 'Selección agrupada',
	treeSelect: 'Selección jerárquica',
	cascader: 'Selección en cascada',
	asyncSelect: 'Selección asíncrona',
	placeholder: 'Elige un contexto',
	searchPlaceholder: 'Buscar opciones locales',
	empty: 'No hay opciones coincidentes',
	selectAll: 'Seleccionar todo',
	clearAll: 'Limpiar todo',
	retry: 'Reintentar',
	loading: 'Cargando opciones',
	error: 'No se pudieron cargar las opciones',
	options: [
		{ value: 'communications', label: 'Comunicaciones', description: 'Mensajes canónicos y evidencia fuente', icon: 'tabler:messages' },
		{ value: 'knowledge', label: 'Conocimiento', description: 'Hechos y observaciones revisados', icon: 'tabler:bulb' },
		{ value: 'projects', label: 'Proyectos', description: 'Contexto de trabajo acotado', icon: 'tabler:briefcase' },
		{ value: 'documents', label: 'Documentos', description: 'Artefactos de evidencia versionados', icon: 'tabler:file-text' }
	],
	groups: [
		{ id: 'memory', label: 'Memoria', options: [
			{ value: 'communications', label: 'Comunicaciones' },
			{ value: 'knowledge', label: 'Conocimiento' }
		] },
		{ id: 'work', label: 'Trabajo', options: [
			{ value: 'projects', label: 'Proyectos' },
			{ value: 'documents', label: 'Documentos' }
		] }
	],
	tree: [
		{ value: 'memory', label: 'Memoria', children: [
			{ value: 'communications', label: 'Comunicaciones' },
			{ value: 'knowledge', label: 'Conocimiento' }
		] },
		{ value: 'work', label: 'Trabajo', children: [
			{ value: 'projects', label: 'Proyectos' },
			{ value: 'documents', label: 'Documentos' }
		] }
	]
}
```

- [ ] **Step 3: Create the baseline Select story**

Create `frontend/stories/ui/GeneralSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, Select } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, Select },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'communications' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.select }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.select }}</FormLabel>
						<Select v-model="value" :options="text.selection.options" :aria-label="text.selection.select" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 4: Create `GeneralSearchableSelect.stories.ts`**

Create `frontend/stories/ui/GeneralSearchableSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, SearchableSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Searchable Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, SearchableSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'knowledge' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.searchableSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.searchableSelect }}</FormLabel>
						<SearchableSelect
							v-model="value"
							:empty-label="text.selection.empty"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:search-placeholder="text.selection.searchPlaceholder"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 5: Create `GeneralMultiSelect.stories.ts`**

Create `frontend/stories/ui/GeneralMultiSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, MultiSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Multi Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, MultiSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: ['communications', 'knowledge'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.multiSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.multiSelect }}</FormLabel>
						<MultiSelect v-model="value" :aria-label="text.selection.multiSelect" :options="text.selection.options" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 6: Create `GeneralSearchableMultiSelect.stories.ts`**

Create `frontend/stories/ui/GeneralSearchableMultiSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, SearchableMultiSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Searchable Multi Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, SearchableMultiSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: ['communications', 'projects'] }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.searchableMultiSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.searchableMultiSelect }}</FormLabel>
						<SearchableMultiSelect
							v-model="value"
							:clear-all-label="text.selection.clearAll"
							:empty-label="text.selection.empty"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:search-placeholder="text.selection.searchPlaceholder"
							:select-all-label="text.selection.selectAll"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 7: Create `GeneralGroupedSelect.stories.ts`**

Create `frontend/stories/ui/GeneralGroupedSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, GroupedSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Grouped Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, GroupedSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'documents' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.groupedSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.groupedSelect }}</FormLabel>
						<GroupedSelect
							v-model="value"
							:groups="text.selection.groups"
							:placeholder="text.selection.placeholder"
							:search-placeholder="text.selection.searchPlaceholder"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 8: Create `GeneralTreeSelect.stories.ts`**

Create `frontend/stories/ui/GeneralTreeSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, TreeSelect } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Tree Select',
	render: (_args, context) => ({
		components: { FormField, FormLabel, TreeSelect },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: 'knowledge' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.treeSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.treeSelect }}</FormLabel>
						<TreeSelect v-model="value" :options="text.selection.tree" :placeholder="text.selection.placeholder" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 9: Create `GeneralCascader.stories.ts`**

Create `frontend/stories/ui/GeneralCascader.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Cascader, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Cascader',
	render: (_args, context) => ({
		components: { Cascader, FormField, FormLabel },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return { text, value: '' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.cascader }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.cascader }}</FormLabel>
						<Cascader v-model="value" :options="text.selection.tree" :placeholder="text.selection.placeholder" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 10: Create `GeneralAsyncSelect.stories.ts`**

Create `frontend/stories/ui/GeneralAsyncSelect.stories.ts`:

```ts
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { AsyncSelect, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Async Select',
	render: (_args, context) => ({
		components: { AsyncSelect, FormField, FormLabel },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				loadedValue: 'communications',
				loadingValue: '',
				errorValue: ''
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.asyncSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.asyncSelect }}</FormLabel>
						<AsyncSelect
							v-model="loadedValue"
							:empty-label="text.selection.empty"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
							:search-placeholder="text.selection.searchPlaceholder"
						/>
					</FormField>
					<FormField>
						<FormLabel>{{ text.selection.loading }}</FormLabel>
						<AsyncSelect
							v-model="loadingValue"
							loading
							:options="[]"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
						/>
					</FormField>
					<FormField>
						<FormLabel>{{ text.selection.error }}</FormLabel>
						<AsyncSelect
							v-model="errorValue"
							:error="text.selection.error"
							:options="[]"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
```

- [ ] **Step 11: Run Storybook index check**

Run:

```bash
cd frontend && pnpm storybook:build
```

Expected: PASS and generated index includes `makosh-ui-general-select`, `makosh-ui-general-searchable-select`, `makosh-ui-general-searchable-multi-select`, `makosh-ui-general-tree-select`, `makosh-ui-general-cascader`, and `makosh-ui-general-async-select`.

---

## Task 8: Update Docs Inventory And Run Validation

**Files:**
- Modify: `docs/architecture/frontend-ui-components.md`

- [ ] **Step 1: Update component inventory**

Add the following names to the component inventory in `docs/architecture/frontend-ui-components.md`:

```text
AsyncSelect
Cascader
GroupedSelect
SearchableMultiSelect
SearchableSelect
TreeSelect
```

Update the Storybook coverage section to include:

```text
GeneralSelect.stories.ts
GeneralSearchableSelect.stories.ts
GeneralMultiSelect.stories.ts
GeneralSearchableMultiSelect.stories.ts
GeneralGroupedSelect.stories.ts
GeneralTreeSelect.stories.ts
GeneralCascader.stories.ts
GeneralAsyncSelect.stories.ts
```

Add this sentence:

```md
General control stories are organized by component name under `Макошь UI / General`
so standard controls can be inspected directly before domain-specific surfaces.
```

- [ ] **Step 2: Run targeted unit tests**

Run:

```bash
cd frontend && pnpm test:unit -- src/shared/ui/inputs.boundary.test.ts
```

Expected: PASS.

- [ ] **Step 3: Run typecheck**

Run:

```bash
cd frontend && pnpm typecheck
```

Expected: PASS.

- [ ] **Step 4: Run lint for touched files**

Run:

```bash
cd frontend && pnpm lint:ox src/shared/ui/Selection.types.ts src/shared/ui/SearchableSelect.vue src/shared/ui/SearchableMultiSelect.vue src/shared/ui/GroupedSelect.vue src/shared/ui/TreeSelect.vue src/shared/ui/Cascader.vue src/shared/ui/AsyncSelect.vue src/shared/ui/inputs.boundary.test.ts src/shared/ui/styles/controls.css stories/ui/GeneralSelect.stories.ts stories/ui/GeneralSearchableSelect.stories.ts stories/ui/GeneralMultiSelect.stories.ts stories/ui/GeneralSearchableMultiSelect.stories.ts stories/ui/GeneralGroupedSelect.stories.ts stories/ui/GeneralTreeSelect.stories.ts stories/ui/GeneralCascader.stories.ts stories/ui/GeneralAsyncSelect.stories.ts stories/ui/storybook-i18n.ts
```

Expected: PASS.

- [ ] **Step 5: Run Storybook build**

Run:

```bash
cd frontend && pnpm storybook:build
```

Expected: PASS. Existing third-party Vite/Rolldown warnings may appear, but the command must exit with status 0.

- [ ] **Step 6: Run Storybook interaction tests**

Run:

```bash
cd frontend && pnpm storybook:test
```

Expected: PASS.

- [ ] **Step 7: Update visual snapshots**

Run:

```bash
cd frontend && pnpm test:visual:update
```

Expected: PASS and new screenshots generated for the eight `Макошь UI / General` selection stories across configured themes, locales and widths.

- [ ] **Step 8: Re-run visual comparison**

Run:

```bash
cd frontend && pnpm test:visual
```

Expected: PASS using the updated snapshots.

- [ ] **Step 9: Restart Storybook static server**

Run:

```bash
cd frontend && MAKOSH_STORYBOOK_HOST=0.0.0.0 pnpm storybook:serve
```

Expected: Storybook serves at `http://127.0.0.1:6006`.

- [ ] **Step 10: Confirm Storybook index entries**

Run:

```bash
curl -fsS http://127.0.0.1:6006/index.json | node -e "let s='';process.stdin.on('data',d=>s+=d);process.stdin.on('end',()=>{const data=JSON.parse(s); const ids=Object.keys(data.entries || {}); console.log(ids.filter((id)=>id.startsWith('makosh-ui-general-')).sort().join('\n'))})"
```

Expected output contains:

```text
makosh-ui-general-async-select--default
makosh-ui-general-cascader--default
makosh-ui-general-grouped-select--default
makosh-ui-general-multi-select--default
makosh-ui-general-searchable-multi-select--default
makosh-ui-general-searchable-select--default
makosh-ui-general-select--default
makosh-ui-general-tree-select--default
```

- [ ] **Step 11: Check git state for this slice**

Run:

```bash
git status --short -- frontend/src/shared/ui frontend/stories/ui docs/architecture/frontend-ui-components.md frontend/tests/visual
```

Expected: only files from this selection slice plus visual snapshots are changed in this scope.

Do not commit unless the user explicitly requests it.

---

## Self-Review Checklist

- Spec coverage: The plan implements target `Макошь UI / General` selection stories, six new components, baseline direct stories for existing `Select` and `MultiSelect`, shared types, docs, tests and visual validation.
- Placeholder scan: No placeholder markers, incomplete file paths or undefined component names are intentionally left in this plan.
- Type consistency: The plan uses `SelectOption`, `SelectGroup`, `TreeSelectOption`, `modelValue`, `options`, `groups`, `search`, `select`, `clear`, `retry`, `open` and `close` consistently across tasks.
- Scope check: Domain-specific entity pickers, backend search and provider-specific selection are excluded from this slice.
