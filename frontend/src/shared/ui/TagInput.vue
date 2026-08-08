<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	modelValue?: string[]
	suggestions?: string[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	removeLabel?: string
	class?: string
}>(), {
	modelValue: () => [],
	suggestions: () => [],
	placeholder: '',
	ariaLabel: 'Tag input',
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
	'makosh-tag-input',
	{ 'makosh-tag-input--disabled': props.disabled },
	props.class
])
const filteredSuggestions = computed(() => {
	const query = draft.value.trim().toLocaleLowerCase()
	return props.suggestions
		.filter((suggestion) => !props.modelValue.includes(suggestion))
		.filter((suggestion) => query === '' || suggestion.toLocaleLowerCase().includes(query))
		.slice(0, 6)
})

function normalizedDraft(): string {
	return draft.value.trim().replace(/\s+/g, ' ')
}

function addTag(tag = normalizedDraft()): void {
	if (!tag || props.disabled || props.modelValue.includes(tag)) {
		draft.value = ''
		return
	}
	emit('update:modelValue', [...props.modelValue, tag])
	emit('add', tag)
	draft.value = ''
}

function removeTag(tag: string): void {
	if (props.disabled) {
		return
	}
	emit('update:modelValue', props.modelValue.filter((value) => value !== tag))
	emit('remove', tag)
}

function handleKeydown(event: KeyboardEvent): void {
	if (event.key === 'Enter' || event.key === ',') {
		event.preventDefault()
		addTag()
	}
}
</script>

<template>
	<div :class="classes">
		<div class="makosh-tag-input__control">
			<span v-for="tag in modelValue" :key="tag" class="makosh-tag-input__tag">
				<span>{{ tag }}</span>
				<button
					class="makosh-tag-input__remove"
					type="button"
					:aria-label="`${removeLabel} ${tag}`"
					:disabled="disabled"
					@click="removeTag(tag)"
				>
					<Icon icon="tabler:x" size="0.875rem" aria-hidden="true" />
				</button>
			</span>
			<input
				v-model="draft"
				class="makosh-tag-input__field"
				:aria-label="ariaLabel"
				:disabled="disabled"
				:placeholder="placeholder"
				@blur="addTag()"
				@keydown="handleKeydown"
			/>
		</div>
		<div v-if="filteredSuggestions.length > 0" class="makosh-tag-input__suggestions" role="listbox" :aria-label="ariaLabel">
			<button
				v-for="suggestion in filteredSuggestions"
				:key="suggestion"
				class="makosh-tag-input__suggestion"
				type="button"
				role="option"
				@click="addTag(suggestion)"
			>
				{{ suggestion }}
			</button>
		</div>
	</div>
</template>
