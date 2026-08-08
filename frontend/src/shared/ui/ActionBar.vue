<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutJustify } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	position?: 'top' | 'bottom' | 'inline'
	justify?: LayoutJustify
	label?: string
	class?: string
}>(), {
	as: 'div',
	position: 'inline',
	justify: 'end'
})

const classes = computed(() => [
	'makosh-action-bar',
	`makosh-action-bar--${props.position}`,
	`makosh-action-bar--justify-${props.justify}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" role="group" :aria-label="label">
		<slot />
	</component>
</template>
