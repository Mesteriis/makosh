<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutAlign, LayoutGap, LayoutJustify } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	gap?: LayoutGap
	align?: LayoutAlign
	justify?: LayoutJustify
	wrap?: boolean
	class?: string
}>(), {
	as: 'div',
	gap: 'md',
	align: 'center',
	justify: 'start',
	wrap: false
})

const classes = computed(() => [
	'makosh-stack',
	'makosh-stack--row',
	`makosh-stack--gap-${props.gap}`,
	`makosh-stack--align-${props.align}`,
	`makosh-stack--justify-${props.justify}`,
	{ 'makosh-stack--wrap': props.wrap },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
