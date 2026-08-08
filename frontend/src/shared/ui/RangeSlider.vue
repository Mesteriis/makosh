<script setup lang="ts">
import { computed } from 'vue'

export interface RangeSliderValue {
	min: number
	max: number
}

const props = withDefaults(defineProps<{
	modelValue?: RangeSliderValue
	min?: number
	max?: number
	step?: number
	label?: string
	minLabel?: string
	maxLabel?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: () => ({ min: 20, max: 80 }),
	min: 0,
	max: 100,
	step: 1,
	minLabel: 'Minimum value',
	maxLabel: 'Maximum value',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: RangeSliderValue]
}>()

const classes = computed(() => ['makosh-slider makosh-range-slider', props.class])

function updateMin(event: Event): void {
	const target = event.target as HTMLInputElement
	const nextMin = Math.min(Number(target.value), props.modelValue.max)
	emit('update:modelValue', { min: nextMin, max: props.modelValue.max })
}

function updateMax(event: Event): void {
	const target = event.target as HTMLInputElement
	const nextMax = Math.max(Number(target.value), props.modelValue.min)
	emit('update:modelValue', { min: props.modelValue.min, max: nextMax })
}
</script>

<template>
	<div :class="classes">
		<div v-if="label" class="makosh-slider__header">
			<span>{{ label }}</span>
			<span class="makosh-slider__value">{{ modelValue.min }} - {{ modelValue.max }}</span>
		</div>
		<div class="makosh-range-slider__inputs">
			<input
				:aria-label="minLabel"
				class="makosh-slider__input"
				:disabled="disabled"
				:max="max"
				:min="min"
				:step="step"
				type="range"
				:value="modelValue.min"
				@input="updateMin"
			/>
			<input
				:aria-label="maxLabel"
				class="makosh-slider__input"
				:disabled="disabled"
				:max="max"
				:min="min"
				:step="step"
				type="range"
				:value="modelValue.max"
				@input="updateMax"
			/>
		</div>
	</div>
</template>
