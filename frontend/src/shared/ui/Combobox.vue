<script setup lang="ts">
import { computed } from 'vue'

interface ComboboxOption {
	value: string
	label: string
	disabled?: boolean
}

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	options?: readonly ComboboxOption[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: '',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: ComboboxOption]
}>()

const classes = computed(() => ['makosh-combobox', props.class])
const listId = computed(() => `${props.id ?? 'makosh-combobox'}-options`)

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
	const selectedOption = props.options.find((option) => option.value === target.value || option.label === target.value)
	if (selectedOption) {
		emit('select', selectedOption)
	}
}
</script>

<template>
	<div :class="classes">
		<input
			class="makosh-native-control"
			:aria-label="ariaLabel"
			:disabled="disabled"
			:id="id"
			:list="listId"
			:placeholder="placeholder"
			:readonly="readonly"
			:type="'text'"
			:value="modelValue"
			@input="handleInput"
		/>
		<datalist :id="listId">
			<option
				v-for="option in options"
				:key="option.value"
				:disabled="option.disabled"
				:value="option.value"
			>
				{{ option.label }}
			</option>
		</datalist>
	</div>
</template>
