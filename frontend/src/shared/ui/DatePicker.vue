<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	ariaLabel?: string
	min?: string
	max?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: '',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
}>()

const classes = computed(() => ['makosh-native-control', props.class])

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
}
</script>

<template>
	<input
		:aria-label="ariaLabel"
		:class="classes"
		:disabled="disabled"
		:id="id"
		:max="max"
		:min="min"
		:readonly="readonly"
		type="date"
		:value="modelValue"
		@input="handleInput"
	/>
</template>
