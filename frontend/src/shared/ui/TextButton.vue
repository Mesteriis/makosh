<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	size?: 'sm' | 'md' | 'lg'
	tone?: 'accent' | 'quiet' | 'danger'
	icon?: string
	disabled?: boolean
	type?: 'button' | 'submit' | 'reset'
	class?: string
}>(), {
	size: 'md',
	tone: 'accent',
	disabled: false,
	type: 'button'
})

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()

const classes = computed(() => [
	'makosh-text-button',
	`makosh-text-button--${props.size}`,
	`makosh-text-button--${props.tone}`,
	props.class
])

function handleClick(event: MouseEvent): void {
	if (!props.disabled) emit('click', event)
}
</script>

<template>
	<button :class="classes" :disabled="disabled" :type="type" @click="handleClick">
		<Icon v-if="icon" :icon="icon" size="1em" />
		<span><slot /></span>
	</button>
</template>
