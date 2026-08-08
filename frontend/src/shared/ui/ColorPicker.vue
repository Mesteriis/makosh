<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: '#178f6e',
	label: 'Color',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
}>()

const classes = computed(() => ['makosh-color-picker', props.class])

function updateValue(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
}
</script>

<template>
	<div :class="classes">
		<input
			class="makosh-color-picker__input"
			:aria-label="label"
			:disabled="disabled"
			:id="id"
			type="color"
			:value="modelValue"
			@input="updateValue"
		/>
		<input
			class="makosh-native-control makosh-color-picker__value"
			:aria-label="`${label} value`"
			:disabled="disabled"
			:value="modelValue"
			@input="updateValue"
		/>
	</div>
</template>
