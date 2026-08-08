<script setup lang="ts">
import { computed } from 'vue'

type DateRangeValue = {
	start?: string
	end?: string
}

const props = withDefaults(defineProps<{
	modelValue?: DateRangeValue
	startLabel?: string
	endLabel?: string
	ariaLabel?: string
	min?: string
	max?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: () => ({ start: '', end: '' }),
	startLabel: 'Start',
	endLabel: 'End',
	ariaLabel: 'Date range',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: DateRangeValue]
}>()

const classes = computed(() => ['makosh-date-range-picker', props.class])

function updateValue(key: keyof DateRangeValue, event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', { ...props.modelValue, [key]: target.value })
}
</script>

<template>
	<div :class="classes" role="group" :aria-label="ariaLabel">
		<label class="makosh-date-range-picker__field">
			<span class="makosh-date-range-picker__label">{{ startLabel }}</span>
			<input
				class="makosh-native-control"
				:disabled="disabled"
				:max="max"
				:min="min"
				:readonly="readonly"
				type="date"
				:value="modelValue.start"
				@input="updateValue('start', $event)"
			/>
		</label>
		<label class="makosh-date-range-picker__field">
			<span class="makosh-date-range-picker__label">{{ endLabel }}</span>
			<input
				class="makosh-native-control"
				:disabled="disabled"
				:max="max"
				:min="min"
				:readonly="readonly"
				type="date"
				:value="modelValue.end"
				@input="updateValue('end', $event)"
			/>
		</label>
	</div>
</template>
