<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import Button from './primitives/Button.vue'
import IconButton from './IconButton.vue'

const props = withDefaults(defineProps<{
	modelValue?: string
	id?: string
	label?: string
	placeholder?: string
	helper?: string
	sendLabel?: string
	attachLabel?: string
	disabled?: boolean
	loading?: boolean
	rows?: number
	autoGrow?: boolean
	maxRows?: number
	maxLength?: number
	showAttach?: boolean
	class?: string
}>(), {
	modelValue: '',
	placeholder: 'Write a message',
	sendLabel: 'Send',
	attachLabel: 'Attach',
	disabled: false,
	loading: false,
	rows: 3,
	autoGrow: false,
	maxRows: 5,
	showAttach: true
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	submit: [value: string]
	attach: []
}>()

const classes = computed(() => [
	'makosh-chat-input',
	{
		'makosh-chat-input--disabled': props.disabled,
		'makosh-chat-input--auto-grow': props.autoGrow,
		'makosh-chat-input--loading': props.loading
	},
	props.class
])
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const canSubmit = computed(() => props.modelValue.trim().length > 0 && !props.disabled && !props.loading)
const remaining = computed(() => {
	if (props.maxLength == null) {
		return undefined
	}
	return props.maxLength - props.modelValue.length
})

function resizeTextarea(): void {
	if (!props.autoGrow || !textareaRef.value) {
		return
	}

	const textarea = textareaRef.value
	textarea.style.height = 'auto'

	const styles = window.getComputedStyle(textarea)
	const lineHeight = Number.parseFloat(styles.lineHeight)
	const paddingTop = Number.parseFloat(styles.paddingTop)
	const paddingBottom = Number.parseFloat(styles.paddingBottom)
	const rowHeight = Number.isFinite(lineHeight) ? lineHeight : 20
	const verticalPadding = (Number.isFinite(paddingTop) ? paddingTop : 0) + (Number.isFinite(paddingBottom) ? paddingBottom : 0)
	const maxHeight = rowHeight * props.maxRows + verticalPadding

	textarea.style.height = `${Math.min(textarea.scrollHeight, maxHeight)}px`
	textarea.style.overflowY = textarea.scrollHeight > maxHeight ? 'auto' : 'hidden'
}

function handleInput(event: Event): void {
	const target = event.target as HTMLTextAreaElement
	emit('update:modelValue', target.value)
	void nextTick(resizeTextarea)
}

function submit(): void {
	if (canSubmit.value) {
		emit('submit', props.modelValue)
	}
}

watch(
	() => [props.modelValue, props.rows, props.maxRows, props.autoGrow] as const,
	() => {
		void nextTick(resizeTextarea)
	},
	{ immediate: true }
)
</script>

<template>
	<form :class="classes" @submit.prevent="submit">
		<label v-if="label" class="makosh-chat-input__label" :for="id">{{ label }}</label>
		<div class="makosh-chat-input__surface">
			<textarea
				:id="id"
				ref="textareaRef"
				class="makosh-chat-input__textarea"
				:disabled="disabled || loading"
				:maxlength="maxLength"
				:placeholder="placeholder"
				:rows="rows"
				:value="modelValue"
				@input="handleInput"
				@keydown.ctrl.enter.prevent="submit"
				@keydown.meta.enter.prevent="submit"
			/>
			<div class="makosh-chat-input__actions">
				<IconButton
					v-if="showAttach"
					icon="tabler:paperclip"
					:label="attachLabel"
					size="sm"
					variant="ghost"
					:disabled="disabled || loading"
					@click="emit('attach')"
				/>
				<slot name="toolbar" />
				<Button type="submit" size="sm" :loading="loading" :disabled="!canSubmit">
					{{ sendLabel }}
				</Button>
			</div>
		</div>
		<div v-if="helper || remaining !== undefined" class="makosh-chat-input__meta">
			<span v-if="helper">{{ helper }}</span>
			<span v-if="remaining !== undefined">{{ remaining }}</span>
		</div>
	</form>
</template>
