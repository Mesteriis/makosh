<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	id?: string
	modelValue?: string
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	showLabel?: string
	hideLabel?: string
	class?: string
}>(), {
	modelValue: '',
	placeholder: '',
	disabled: false,
	readonly: false,
	showLabel: 'Show password',
	hideLabel: 'Hide password'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
}>()

const visible = ref(false)
const classes = computed(() => ['makosh-native-control', props.class])
const inputType = computed(() => visible.value ? 'text' : 'password')
const toggleLabel = computed(() => visible.value ? props.hideLabel : props.showLabel)
const toggleIcon = computed(() => visible.value ? 'tabler:eye-off' : 'tabler:eye')

function handleInput(event: Event): void {
	const target = event.target as HTMLInputElement
	emit('update:modelValue', target.value)
}
</script>

<template>
	<div class="makosh-affix-control makosh-affix-control--trailing">
		<input
			:aria-label="ariaLabel"
			:class="classes"
			:disabled="disabled"
			:id="id"
			:placeholder="placeholder"
			:readonly="readonly"
			:type="inputType"
			:value="modelValue"
			@input="handleInput"
		/>
		<span class="makosh-affix-control__trailing">
			<button
				class="makosh-password-toggle"
				type="button"
				:aria-label="toggleLabel"
				:disabled="disabled || readonly"
				@click="visible = !visible"
			>
				<Icon :icon="toggleIcon" size="1rem" />
			</button>
		</span>
	</div>
</template>
