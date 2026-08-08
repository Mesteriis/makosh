<script setup lang="ts">
import { computed } from 'vue'
import Button from './primitives/Button.vue'

const props = withDefaults(defineProps<{
	icon: string
	label: string
	variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive'
	size?: 'sm' | 'md' | 'lg'
	disabled?: boolean
	loading?: boolean
	class?: string
	type?: 'button' | 'submit' | 'reset'
}>(), {
	variant: 'ghost',
	size: 'md',
	disabled: false,
	loading: false,
	type: 'button'
})

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()

const classes = computed(() => ['makosh-icon-button', props.class].filter(Boolean).join(' '))
</script>

<template>
	<Button
		:aria-label="label"
		:class="classes"
		:disabled="disabled"
		:icon="icon"
		:loading="loading"
		:size="size"
		:title="label"
		:type="type"
		:variant="variant"
		@click="emit('click', $event)"
	/>
</template>
