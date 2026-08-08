<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	modelValue?: number
	min?: number
	max?: number
	step?: number
	label?: string
	disabled?: boolean
	showValue?: boolean
	class?: string
}>(), {
	modelValue: 0,
	min: 0,
	max: 100,
	step: 1,
	disabled: false,
	showValue: true
})

const emit = defineEmits<{
	'update:modelValue': [value: number]
}>()

const classes = computed(() => ['makosh-slider', props.class])

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', Number(target.value))
}
</script>

<template>
	<label :class="classes">
		<span v-if="label || showValue" class="makosh-slider__header">
			<span>{{ label }}</span>
			<span v-if="showValue" class="makosh-slider__value">{{ modelValue }}</span>
		</span>
		<input
			class="makosh-slider__input"
			:disabled="disabled"
			:max="max"
			:min="min"
			:step="step"
			type="range"
			:value="modelValue"
			@input="handleInput"
		/>
	</label>
</template>
