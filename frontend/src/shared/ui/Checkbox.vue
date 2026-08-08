<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	modelValue?: boolean
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: false,
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: boolean]
}>()

const classes = computed(() => ['makosh-choice', { 'makosh-choice--disabled': props.disabled }, props.class])

function handleChange(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.checked)
}
</script>

<template>
	<label :class="classes">
		<input
			class="makosh-checkbox-input"
			:checked="modelValue"
			:disabled="disabled"
			type="checkbox"
			@change="handleChange"
		/>
		<span><slot>{{ label }}</slot></span>
	</label>
</template>
