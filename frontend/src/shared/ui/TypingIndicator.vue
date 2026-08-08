<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	label?: string
	active?: boolean
	class?: string
}>(), {
	label: 'Typing',
	active: true
})

const classes = computed(() => [
	'makosh-typing-indicator',
	{ 'makosh-typing-indicator--active': props.active },
	props.class
])
</script>

<template>
	<span :class="classes" role="status" :aria-label="label">
		<span class="makosh-typing-indicator__dot" aria-hidden="true" />
		<span class="makosh-typing-indicator__dot" aria-hidden="true" />
		<span class="makosh-typing-indicator__dot" aria-hidden="true" />
		<span class="makosh-sr-only">{{ label }}</span>
	</span>
</template>
