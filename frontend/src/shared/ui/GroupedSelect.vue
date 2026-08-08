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

const classes = computed(() => ['makosh-grouped-select', props.class].filter(Boolean).join(' '))
const options = computed<SelectOption[]>(() => {
	return props.groups.flatMap((group) =>
		group.options.map((option) => ({
			...option,
			description: option.description ?? group.label
		}))
	)
})

function updateModelValue(value: string): void {
	emit('update:modelValue', value)
}

function emitSearch(query: string): void {
	emit('search', query)
}

function emitSelect(option: SelectOption): void {
	emit('select', option)
}

function emitClear(): void {
	emit('clear')
}
</script>

<template>
	<SearchableSelect
		:model-value="modelValue"
		:options="options"
		:placeholder="placeholder"
		:search-placeholder="searchPlaceholder"
		:aria-label="ariaLabel"
		:disabled="disabled"
		:empty-label="emptyLabel"
		:class="classes"
		@update:model-value="updateModelValue"
		@search="emitSearch"
		@select="emitSelect"
		@clear="emitClear"
	/>
</template>
