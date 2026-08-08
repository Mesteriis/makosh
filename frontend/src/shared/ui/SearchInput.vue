<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	clearLabel?: string
	class?: string
}>(), {
	modelValue: '',
	placeholder: '',
	disabled: false,
	readonly: false,
	clearLabel: 'Clear search'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	search: [value: string]
	clear: []
}>()

const classes = computed(() => ['makosh-native-control', props.class])
const canClear = computed(() => Boolean(props.modelValue) && !props.disabled && !props.readonly)

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
	emit('search', target.value)
}

function clearSearch(): void {
	emit('update:modelValue', '')
	emit('search', '')
	emit('clear')
}
</script>

<template>
	<div class="makosh-affix-control makosh-affix-control--leading makosh-affix-control--trailing">
		<span class="makosh-affix-control__leading" aria-hidden="true">
			<Icon icon="tabler:search" size="1rem" />
		</span>
		<input
			:aria-label="ariaLabel"
			:class="classes"
			:disabled="disabled"
			:id="id"
			:placeholder="placeholder"
			:readonly="readonly"
			:type="'search'"
			:value="modelValue"
			@input="handleInput"
		/>
		<span v-if="canClear" class="makosh-affix-control__trailing">
			<button class="makosh-input-clear" type="button" :aria-label="clearLabel" @click="clearSearch">
				<Icon icon="tabler:x" size="1rem" />
			</button>
		</span>
	</div>
</template>
