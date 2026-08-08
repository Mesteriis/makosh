<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	modelValue?: string[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	maxTokens?: number
	removeLabel?: string
	class?: string
}>(), {
	modelValue: () => [],
	placeholder: '',
	ariaLabel: 'Token input',
	disabled: false,
	removeLabel: 'Remove'
})

const emit = defineEmits<{
	'update:modelValue': [value: string[]]
	add: [value: string]
	remove: [value: string]
}>()

const draft = ref('')
const classes = computed(() => [
	'makosh-token-input',
	{ 'makosh-token-input--disabled': props.disabled },
	props.class
])
const canAddMore = computed(() => props.maxTokens === undefined || props.modelValue.length < props.maxTokens)

function normalizedDraft(): string {
	return draft.value.trim().replace(/\s+/g, ' ')
}

function addToken(): void {
	const nextToken = normalizedDraft()
	if (!nextToken || props.disabled || !canAddMore.value || props.modelValue.includes(nextToken)) {
		draft.value = ''
		return
	}
	emit('update:modelValue', [...props.modelValue, nextToken])
	emit('add', nextToken)
	draft.value = ''
}

function handleKeydown(event: KeyboardEvent): void {
	if (event.key === 'Enter' || event.key === ',') {
		event.preventDefault()
		addToken()
	}
	if (event.key === 'Backspace' && draft.value === '' && props.modelValue.length > 0) {
		removeToken(props.modelValue[props.modelValue.length - 1])
	}
}

function removeToken(token: string): void {
	if (props.disabled) {
		return
	}
	emit('update:modelValue', props.modelValue.filter((value) => value !== token))
	emit('remove', token)
}
</script>

<template>
	<div :class="classes">
		<div class="makosh-token-input__control">
			<span v-for="token in modelValue" :key="token" class="makosh-selection-chip">
				<span class="makosh-selection-chip__label">{{ token }}</span>
				<button
					class="makosh-selection-chip__remove"
					type="button"
					:aria-label="`${removeLabel} ${token}`"
					:disabled="disabled"
					@click="removeToken(token)"
				>
					<Icon icon="tabler:x" size="0.875rem" aria-hidden="true" />
				</button>
			</span>
			<input
				v-model="draft"
				class="makosh-token-input__field"
				:aria-label="ariaLabel"
				:disabled="disabled || !canAddMore"
				:placeholder="placeholder"
				@blur="addToken"
				@keydown="handleKeydown"
			/>
		</div>
	</div>
</template>
