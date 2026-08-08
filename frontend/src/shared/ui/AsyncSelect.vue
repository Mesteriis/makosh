<script setup lang="ts">
import { computed } from 'vue'
import Button from './primitives/Button.vue'
import SearchableSelect from './SearchableSelect.vue'
import type { SelectOption } from './Selection.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: SelectOption[]
	placeholder?: string
	searchPlaceholder?: string
	ariaLabel?: string
	disabled?: boolean
	loading?: boolean
	loadingLabel?: string
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
	loadingLabel: 'Loading options',
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

const classes = computed(() => ['makosh-async-select', props.class])
const visibleLoadingLabel = computed(() => props.loadingLabel.trim())
const loadingAnnouncement = computed(() => visibleLoadingLabel.value || 'Loading options')
const errorMessage = computed(() => props.error.trim())
const hasError = computed(() => errorMessage.value.length > 0)
const isSelectDisabled = computed(() => props.disabled || props.loading || hasError.value)

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

function emitRetry(): void {
	emit('retry')
}
</script>

<template>
	<div :class="classes">
		<SearchableSelect
			:model-value="modelValue"
			:options="options"
			:placeholder="placeholder"
			:search-placeholder="searchPlaceholder"
			:aria-label="ariaLabel"
			:disabled="isSelectDisabled"
			:empty-label="emptyLabel"
			@update:model-value="updateModelValue"
			@search="emitSearch"
			@select="emitSelect"
			@clear="emitClear"
		/>
		<div
			v-if="loading"
			class="makosh-async-select__state"
			role="status"
			aria-live="polite"
			:aria-label="loadingAnnouncement"
		>
			<span class="makosh-async-select__loading-mark" aria-hidden="true">
				<span class="makosh-async-select__loading-dot" />
				<span class="makosh-async-select__loading-dot" />
				<span class="makosh-async-select__loading-dot" />
			</span>
			<span v-if="visibleLoadingLabel">{{ visibleLoadingLabel }}</span>
		</div>
		<div v-else-if="hasError" class="makosh-async-select__state makosh-async-select__state--error" role="alert">
			<span>{{ errorMessage }}</span>
			<Button variant="outline" size="sm" @click="emitRetry">{{ retryLabel }}</Button>
		</div>
	</div>
</template>
