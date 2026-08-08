<script setup lang="ts">
import { computed, ref } from 'vue'
import Button from './primitives/Button.vue'

type CopyState = 'idle' | 'copied' | 'error'

const props = withDefaults(defineProps<{
	value?: string
	label?: string
	copiedLabel?: string
	errorLabel?: string
	disabled?: boolean
	class?: string
}>(), {
	value: '',
	label: 'Copy',
	copiedLabel: 'Copied',
	errorLabel: 'Copy failed',
	disabled: false
})

const emit = defineEmits<{
	copied: [value: string]
	error: [error: unknown]
}>()

const state = ref<CopyState>('idle')
const buttonLabel = computed(() => {
	if (state.value === 'copied') {
		return props.copiedLabel
	}
	if (state.value === 'error') {
		return props.errorLabel
	}
	return props.label
})
const icon = computed(() => {
	if (state.value === 'copied') {
		return 'tabler:check'
	}
	if (state.value === 'error') {
		return 'tabler:alert-triangle'
	}
	return 'tabler:copy'
})
const classes = computed(() => ['makosh-copy-button', `makosh-copy-button--${state.value}`, props.class].filter(Boolean).join(' '))

async function copy(): Promise<void> {
	if (props.disabled || !props.value) {
		return
	}

	try {
		if (typeof navigator === 'undefined' || !navigator.clipboard?.writeText) {
			throw new Error('Clipboard API unavailable')
		}
		await navigator.clipboard.writeText(props.value)
		state.value = 'copied'
		emit('copied', props.value)
	} catch (error) {
		state.value = 'error'
		emit('error', error)
	}
}
</script>

<template>
	<Button
		:class="classes"
		:disabled="disabled || !value"
		:icon="icon"
		size="sm"
		variant="outline"
		@click="copy"
	>
		{{ buttonLabel }}
	</Button>
</template>
