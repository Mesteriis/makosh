<script setup lang="ts">
import { computed, nextTick } from 'vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	length?: number
	label?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: '',
	length: 6,
	label: 'One-time code',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	complete: [value: string]
}>()

const classes = computed(() => ['makosh-otp', props.class])
const positions = computed(() => Array.from({ length: props.length }, (_value, index) => index))

function inputId(index: number): string {
	return `${props.id ?? 'makosh-otp'}-${index}`
}

function digitAt(index: number): string {
	return props.modelValue[index] ?? ''
}

function cleanValue(value: string): string {
	return value.replace(/\s/g, '').slice(0, props.length)
}

function updateDigit(index: number, value: string): void {
	const nextDigits = props.modelValue.padEnd(props.length, ' ').slice(0, props.length).split('')
	nextDigits[index] = value.slice(-1)
	const nextValue = cleanValue(nextDigits.join(''))
	emit('update:modelValue', nextValue)
	if (nextValue.length === props.length) {
		emit('complete', nextValue)
	}
}

async function focusInput(index: number): Promise<void> {
	await nextTick()
	document.getElementById(inputId(index))?.focus()
}

function handleInput(index: number, event: Event): void {
	const target = event.target as HTMLInputElement
	const value = target.value
	if (value.length > 1) {
		handlePaste(index, value)
		return
	}
	updateDigit(index, value)
	if (value && index < props.length - 1) {
		void focusInput(index + 1)
	}
}

function handlePaste(index: number, value: string): void {
	const pasted = cleanValue(value)
	if (!pasted) {
		return
	}
	const current = props.modelValue.padEnd(props.length, ' ').slice(0, props.length).split('')
	for (let offset = 0; offset < pasted.length && index + offset < props.length; offset += 1) {
		current[index + offset] = pasted[offset]
	}
	const nextValue = cleanValue(current.join(''))
	emit('update:modelValue', nextValue)
	if (nextValue.length === props.length) {
		emit('complete', nextValue)
	}
	void focusInput(Math.min(index + pasted.length, props.length - 1))
}

function handlePasteEvent(index: number, event: ClipboardEvent): void {
	event.preventDefault()
	handlePaste(index, event.clipboardData?.getData('text') ?? '')
}

function handleKeydown(index: number, event: KeyboardEvent): void {
	if (event.key === 'Backspace' && !digitAt(index) && index > 0) {
		void focusInput(index - 1)
		return
	}
	if (event.key === 'ArrowLeft' && index > 0) {
		event.preventDefault()
		void focusInput(index - 1)
	}
	if (event.key === 'ArrowRight' && index < props.length - 1) {
		event.preventDefault()
		void focusInput(index + 1)
	}
}
</script>

<template>
	<div :class="classes" role="group" :aria-label="label">
		<input
			v-for="index in positions"
			:id="inputId(index)"
			:key="index"
			class="makosh-otp__cell"
			:aria-label="`${label} ${index + 1}`"
			:disabled="disabled"
			:inputmode="'numeric'"
			:maxlength="1"
			:readonly="readonly"
			:type="'text'"
			:value="digitAt(index)"
			@input="handleInput(index, $event)"
			@keydown="handleKeydown(index, $event)"
			@paste="handlePasteEvent(index, $event)"
		/>
	</div>
</template>
