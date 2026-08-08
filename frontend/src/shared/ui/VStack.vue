<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutAlign, LayoutGap, LayoutJustify } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	gap?: LayoutGap
	align?: LayoutAlign
	justify?: LayoutJustify
	class?: string
}>(), {
	as: 'div',
	gap: 'md',
	align: 'stretch',
	justify: 'start'
})

const classes = computed(() => [
	'makosh-stack',
	'makosh-stack--column',
	`makosh-stack--gap-${props.gap}`,
	`makosh-stack--align-${props.align}`,
	`makosh-stack--justify-${props.justify}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
