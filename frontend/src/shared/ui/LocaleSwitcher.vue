<script setup lang="ts">
import { computed } from 'vue'
import type { LocaleSwitcherOption } from './Utility.types'

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: LocaleSwitcherOption[]
	label?: string
	disabled?: boolean
	class?: string
}>(), {
	modelValue: 'en',
	options: () => [
		{ value: 'ru', label: 'RU' },
		{ value: 'en', label: 'EN' },
		{ value: 'es', label: 'ES' }
	],
	label: 'Locale',
	disabled: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
}>()

const classes = computed(() => ['makosh-locale-switcher', props.class])
</script>

<template>
	<div :class="classes" role="radiogroup" :aria-label="label">
		<button
			v-for="option in options"
			:key="option.value"
			class="makosh-locale-switcher__option"
			type="button"
			role="radio"
			:aria-checked="modelValue === option.value"
			:disabled="disabled"
			:title="option.description"
			@click="emit('update:modelValue', option.value)"
		>
			{{ option.label }}
		</button>
	</div>
</template>
