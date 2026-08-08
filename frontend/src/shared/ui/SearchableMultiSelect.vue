<script setup lang="ts">
import { computed, nextTick, ref, useId } from 'vue'
import Icon from './Icon.vue'
import type { SelectOption } from './Selection.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
	modelValue?: string[]
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	searchAriaLabel?: string
	listboxAriaLabel?: string
	actionsAriaLabel?: string
	removeLabel?: (option: SelectOption) => string
	selectedCountLabel?: (count: number) => string
	disabled?: boolean
	readonly?: boolean
	emptyLabel?: string
	selectAllLabel?: string
	clearAllLabel?: string
	class?: string
}>(), {
	modelValue: () => [],
	options: () => [],
	placeholder: 'Select options...',
	searchPlaceholder: 'Search...',
	removeLabel: (option: SelectOption) => `Remove ${option.label}`,
	selectedCountLabel: (count: number) => `${count} selected`,
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
const activeIndex = ref(0)
const rootRef = ref<HTMLElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)
const listboxRef = ref<HTMLElement | null>(null)
const componentId = `makosh-searchable-multi-select-${useId()}`
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeList, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, popoverRef.value]
})

const classes = computed(() => ['makosh-searchable-multi-select', props.class])
const listId = computed(() => `${componentId}-listbox`)
const selectedValues = computed(() => Array.from(new Set(props.modelValue)))
const selectedValueSet = computed(() => new Set(selectedValues.value))
const enabledOptions = computed(() => props.options.filter((option) => !option.disabled))
const selectedOptions = computed(() => props.options.filter((option) => selectedValueSet.value.has(option.value)))
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
const triggerLabel = computed(() => {
	if (selectedOptions.value.length === 0) {
		return props.placeholder
	}
	if (selectedOptions.value.length === 1) {
		return selectedOptions.value[0]?.label ?? props.placeholder
	}
	return props.selectedCountLabel(selectedOptions.value.length)
})
const accessibleLabel = computed(() => props.ariaLabel ?? props.placeholder)
const searchInputAriaLabel = computed(() => props.searchAriaLabel ?? props.searchPlaceholder)
const resolvedListboxAriaLabel = computed(() => props.listboxAriaLabel ?? accessibleLabel.value)
const resolvedActionsAriaLabel = computed(() => props.actionsAriaLabel ?? accessibleLabel.value)
const canMutate = computed(() => !props.disabled && !props.readonly)
const canSelectAll = computed(() => canMutate.value && enabledOptions.value.some((option) => !selectedValueSet.value.has(option.value)))
const canClearAll = computed(() => canMutate.value && selectedValues.value.length > 0)

function optionId(index: number): string {
	return `${listId.value}-option-${index}`
}

function setActiveIndexToSelected(): void {
	const selectedIndex = filteredOptions.value.findIndex((option) => selectedValueSet.value.has(option.value))
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
	if (!canMutate.value) {
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

function updateSelectedValues(values: string[]): void {
	emit('update:modelValue', Array.from(new Set(values)))
}

function toggleOption(option: SelectOption | undefined): void {
	if (!canMutate.value || !option || option.disabled) {
		return
	}
	const nextValues = selectedValueSet.value.has(option.value)
		? selectedValues.value.filter((value) => value !== option.value)
		: [...selectedValues.value, option.value]
	updateSelectedValues(nextValues)
	if (!selectedValueSet.value.has(option.value)) {
		emit('select', option)
	}
}

function removeChip(option: SelectOption): void {
	if (!canMutate.value) {
		return
	}
	updateSelectedValues(selectedValues.value.filter((value) => value !== option.value))
}

function selectAllEnabledOptions(): void {
	if (!canSelectAll.value) {
		return
	}
	updateSelectedValues([...selectedValues.value, ...enabledOptions.value.map((option) => option.value)])
}

function clearAllSelections(): void {
	if (!canClearAll.value) {
		return
	}
	updateSelectedValues([])
	emit('clear')
}

function isSearchInputTarget(target: EventTarget | null): boolean {
	return target instanceof HTMLInputElement && target.classList.contains('makosh-searchable-select__search')
}

function isTriggerTarget(target: EventTarget | null): boolean {
	return target instanceof HTMLButtonElement && target.classList.contains('makosh-searchable-select__trigger')
}

function handleKeydown(event: KeyboardEvent): void {
	const isSearchInput = isSearchInputTarget(event.target)
	const isTrigger = isTriggerTarget(event.target)
	if (event.key === 'Escape') {
		closeList()
		return
	}
	if (!isSearchInput && !isTrigger) {
		return
	}
	if (isSearchInput && ['Home', 'End'].includes(event.key)) {
		return
	}
	if (!isOpen.value && ['ArrowDown', 'Enter', ' '].includes(event.key)) {
		if (!isTrigger) {
			return
		}
		event.preventDefault()
		openList()
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
	if (event.key === 'Enter' || (event.key === ' ' && isTrigger)) {
		event.preventDefault()
		toggleOption(activeOption.value)
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
			class="makosh-searchable-select__trigger makosh-searchable-multi-select__trigger"
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
			<span class="makosh-searchable-select__value" :class="{ 'makosh-searchable-select__value--placeholder': selectedOptions.length === 0 }">
				{{ triggerLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" class="makosh-searchable-select__chevron" aria-hidden="true" />
		</button>
		<div v-if="selectedOptions.length" class="makosh-selection-chips" aria-live="polite">
			<span v-for="option in selectedOptions" :key="option.value" class="makosh-selection-chip">
				<span class="makosh-selection-chip__label">{{ option.label }}</span>
				<button
					class="makosh-selection-chip__remove"
					type="button"
					:aria-label="removeLabel(option)"
					:disabled="disabled || readonly"
					@click.stop="removeChip(option)"
				>
					<Icon icon="tabler:x" size="0.75rem" aria-hidden="true" />
				</button>
			</span>
		</div>
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
			<div class="makosh-selection-actions" role="group" :aria-label="resolvedActionsAriaLabel">
				<button
					class="makosh-selection-actions__button"
					type="button"
					:disabled="!canSelectAll"
					@click="selectAllEnabledOptions"
				>
					{{ selectAllLabel }}
				</button>
				<button
					class="makosh-selection-actions__button"
					type="button"
					:disabled="!canClearAll"
					@click="clearAllSelections"
				>
					{{ clearAllLabel }}
				</button>
			</div>
			<ul
				:id="listId"
				ref="listboxRef"
				class="makosh-selection-listbox"
				role="listbox"
				:aria-label="resolvedListboxAriaLabel"
				aria-multiselectable="true"
			>
				<li
					v-for="(option, index) in filteredOptions"
					:id="optionId(index)"
					:key="option.value"
					class="makosh-selection-option"
					:class="{ 'makosh-selection-option--active': index === activeIndex }"
					:aria-selected="selectedValueSet.has(option.value)"
					role="option"
					@mousedown.prevent
					@click="toggleOption(option)"
				>
					<Icon v-if="option.icon" :icon="option.icon" size="1rem" class="makosh-selection-option__icon" aria-hidden="true" />
					<span class="makosh-selection-option__body">
						<span class="makosh-selection-option__label">{{ option.label }}</span>
						<span v-if="option.description" class="makosh-selection-option__description">{{ option.description }}</span>
					</span>
					<Icon v-if="selectedValueSet.has(option.value)" icon="tabler:check" size="0.875rem" class="makosh-selection-option__check" aria-hidden="true" />
				</li>
				<li v-if="filteredOptions.length === 0" class="makosh-selection-empty" role="presentation">{{ emptyLabel }}</li>
			</ul>
		</div>
	</div>
</template>
