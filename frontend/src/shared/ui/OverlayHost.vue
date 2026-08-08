<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	label?: string
	layer?: 'popover' | 'dialog' | 'toast'
	passive?: boolean
	class?: string
}>(), {
	as: 'div',
	layer: 'dialog',
	passive: false
})

const classes = computed(() => [
	'makosh-overlay-host',
	`makosh-overlay-host--${props.layer}`,
	{ 'makosh-overlay-host--passive': props.passive },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<slot />
	</component>
</template>
