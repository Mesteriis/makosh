<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: number | null
	placeholder?: string
	ariaLabel?: string
	min?: number
	max?: number
	step?: number
	disabled?: boolean
	readonly?: boolean
	error?: string
	class?: string
}>(), {
	modelValue: null,
	placeholder: '',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: number | null]
}>()

const classes = computed(() => ['makosh-native-control', { 'makosh-native-control--error': props.error }, props.class])
const value = computed(() => props.modelValue ?? '')

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value === '' ? null : Number(target.value))
}
</script>

<template>
	<div class="makosh-input-wrapper">
		<input
			:aria-label="ariaLabel"
			:class="classes"
			:disabled="disabled"
			:id="id"
			:max="max"
			:min="min"
			:placeholder="placeholder"
			:readonly="readonly"
			:step="step"
			type="number"
			:value="value"
			@input="handleInput"
		/>
		<span v-if="error" class="makosh-form-error">{{ error }}</span>
	</div>
</template>
