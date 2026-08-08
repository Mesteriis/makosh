<script setup lang="ts">
import { computed } from 'vue'

interface MultiSelectOption {
	value: string
	label: string
	disabled?: boolean
}

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string[]
	options?: MultiSelectOption[]
	label?: string
	ariaLabel?: string
	disabled?: boolean
	size?: number
	class?: string
}>(), {
	modelValue: () => [],
	options: () => [],
	label: '',
	disabled: false,
	size: 4
})

const emit = defineEmits<{
	'update:modelValue': [value: string[]]
}>()

const classes = computed(() => ['makosh-multi-select', props.class])
const selectedLabels = computed(() => props.options
	.filter((option) => props.modelValue.includes(option.value))
	.map((option) => option.label))

function handleChange(event: Event): void {
	const target = event.target as HTMLSelectElement
	emit('update:modelValue', Array.from(target.selectedOptions, (option) => option.value))
}
</script>

<template>
	<div :class="classes">
		<select
			class="makosh-native-control makosh-multi-select__control"
			:aria-label="ariaLabel ?? label"
			:disabled="disabled"
			:id="id"
			:size="size"
			multiple
			:value="modelValue"
			@change="handleChange"
		>
			<option
				v-for="option in options"
				:key="option.value"
				:disabled="option.disabled"
				:value="option.value"
			>
				{{ option.label }}
			</option>
		</select>
		<div v-if="selectedLabels.length" class="makosh-multi-select__chips" aria-live="polite">
			<span v-for="labelText in selectedLabels" :key="labelText" class="makosh-multi-select__chip">
				{{ labelText }}
			</span>
		</div>
	</div>
</template>
