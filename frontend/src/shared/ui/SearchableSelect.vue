<script setup lang="ts">
import { computed, nextTick, ref, useId } from 'vue'
import Icon from './Icon.vue'
import type { SelectOption } from './Selection.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	clearLabel?: string
	searchAriaLabel?: string
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
	clearLabel: 'Clear selection',
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
const rootRef = ref<HTMLElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)
const listboxRef = ref<HTMLElement | null>(null)
const componentId = `makosh-searchable-select-${useId()}`
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeList, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, popoverRef.value]
})

const classes = computed(() => ['makosh-searchable-select', props.class])
const listId = computed(() => `${componentId}-listbox`)
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
const activeOptionId = computed(() => {
	if (!isOpen.value || !activeOption.value) {
		return undefined
	}
	return optionId(activeIndex.value)
})
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)
const canClear = computed(() => props.clearable && Boolean(props.modelValue) && !props.disabled && !props.readonly)
const accessibleLabel = computed(() => props.ariaLabel ?? props.placeholder)
const searchInputAriaLabel = computed(() => props.searchAriaLabel ?? props.searchPlaceholder)

function optionId(index: number): string {
	return `${listId.value}-option-${index}`
}

function setActiveIndexToSelected(): void {
	const selectedIndex = filteredOptions.value.findIndex((option) => option.value === props.modelValue)
	activeIndex.value = selectedIndex >= 0 ? selectedIndex : 0
}

function scrollActiveOptionIntoView(): void {
	void nextTick(() => {
		const activeElement = listboxRef.value?.children.item(activeIndex.value)
		if (activeElement instanceof HTMLElement) {
			activeElement.scrollIntoView({ block: 'nearest' })
		}
	})
}

function openList(): void {
	if (props.disabled || props.readonly) {
		return
	}
	cancelMouseLeaveDismiss()
	setActiveIndexToSelected()
	if (isOpen.value) {
		scrollActiveOptionIntoView()
		return
	}
	isOpen.value = true
	emit('open')
	scrollActiveOptionIntoView()
}

function closeList(): void {
	cancelMouseLeaveDismiss()
	if (!isOpen.value) {
		return
	}
	isOpen.value = false
	resetQuery()
	activeIndex.value = 0
	emit('close')
}

function resetQuery(): void {
	if (query.value === '') {
		return
	}
	query.value = ''
	emit('search', '')
}

function setQuery(value: string): void {
	query.value = value
	activeIndex.value = 0
	emit('search', value)
	scrollActiveOptionIntoView()
}

function handleSearchInput(event: Event): void {
	const target = event.target as HTMLInputElement
	setQuery(target.value)
}

function selectOption(option: SelectOption | undefined): void {
	if (props.disabled || props.readonly || !option || option.disabled) {
		return
	}
	emit('update:modelValue', option.value)
	emit('select', option)
	closeList()
}

function clearSelection(): void {
	if (!canClear.value) {
		return
	}
	emit('update:modelValue', '')
	emit('clear')
}

function isSearchInputTarget(target: EventTarget | null): boolean {
	return target instanceof HTMLInputElement && target.classList.contains('makosh-searchable-select__search')
}

function handleKeydown(event: KeyboardEvent): void {
	const isSearchInput = isSearchInputTarget(event.target)
	if (isSearchInput && ['Home', 'End'].includes(event.key)) {
		return
	}
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
		scrollActiveOptionIntoView()
	}
	if (event.key === 'ArrowUp') {
		event.preventDefault()
		activeIndex.value = Math.max(activeIndex.value - 1, 0)
		scrollActiveOptionIntoView()
	}
	if (event.key === 'Home') {
		event.preventDefault()
		activeIndex.value = 0
		scrollActiveOptionIntoView()
	}
	if (event.key === 'End') {
		event.preventDefault()
		activeIndex.value = filteredOptions.value.length - 1
		scrollActiveOptionIntoView()
	}
	if (event.key === 'Enter') {
		event.preventDefault()
		selectOption(activeOption.value)
	}
}

function handleFocusout(event: FocusEvent): void {
	const currentTarget = event.currentTarget as HTMLElement
	const nextTarget = event.relatedTarget
	if (!(nextTarget instanceof Node) || !currentTarget.contains(nextTarget)) {
		closeList()
	}
}
</script>

<template>
	<div
		ref="rootRef"
		:class="classes"
		@focusout="handleFocusout"
		@keydown="handleKeydown"
		@mouseenter="cancelMouseLeaveDismiss"
		@mouseleave="scheduleMouseLeaveDismiss"
	>
		<button
			class="makosh-searchable-select__trigger"
			:class="{ 'makosh-searchable-select__trigger--readonly': readonly }"
			type="button"
			:aria-activedescendant="activeOptionId"
			:aria-controls="listId"
			:aria-expanded="isOpen"
			:aria-haspopup="'listbox'"
			:aria-label="accessibleLabel"
			:aria-readonly="readonly"
			:disabled="disabled"
			role="combobox"
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
			:aria-label="clearLabel"
			@click.stop="clearSelection"
		>
			<Icon icon="tabler:x" size="0.875rem" aria-hidden="true" />
		</button>
		<div v-if="isOpen" ref="popoverRef" class="makosh-searchable-select__popover">
			<input
				class="makosh-native-control makosh-searchable-select__search"
				:aria-activedescendant="activeOptionId"
				:aria-controls="listId"
				:aria-label="searchInputAriaLabel"
				:disabled="disabled"
				:placeholder="searchPlaceholder"
				:readonly="readonly"
				:type="'search'"
				:value="query"
				@input="handleSearchInput"
			/>
			<ul :id="listId" ref="listboxRef" class="makosh-selection-listbox" role="listbox">
				<li
					v-for="(option, index) in filteredOptions"
					:id="optionId(index)"
					:key="option.value"
					class="makosh-selection-option"
					:class="{ 'makosh-selection-option--active': index === activeIndex }"
					:aria-selected="option.value === modelValue"
					role="option"
					@mousedown.prevent="selectOption(option)"
				>
					<Icon v-if="option.icon" :icon="option.icon" size="1rem" class="makosh-selection-option__icon" aria-hidden="true" />
					<span class="makosh-selection-option__body">
						<span class="makosh-selection-option__label">{{ option.label }}</span>
						<span v-if="option.description" class="makosh-selection-option__description">{{ option.description }}</span>
					</span>
					<Icon v-if="option.value === modelValue" icon="tabler:check" size="0.875rem" class="makosh-selection-option__check" aria-hidden="true" />
				</li>
				<li v-if="filteredOptions.length === 0" class="makosh-selection-empty" role="presentation">{{ emptyLabel }}</li>
			</ul>
		</div>
	</div>
</template>
