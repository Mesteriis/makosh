<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutAlign, LayoutDirection, LayoutGap, LayoutJustify } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	direction?: LayoutDirection
	gap?: LayoutGap
	align?: LayoutAlign
	justify?: LayoutJustify
	wrap?: boolean
	inline?: boolean
	class?: string
}>(), {
	as: 'div',
	direction: 'row',
	gap: 'md',
	align: 'stretch',
	justify: 'start',
	wrap: false,
	inline: false
})

const classes = computed(() => [
	'makosh-flex',
	`makosh-flex--${props.direction}`,
	`makosh-flex--gap-${props.gap}`,
	`makosh-flex--align-${props.align}`,
	`makosh-flex--justify-${props.justify}`,
	{
		'makosh-flex--wrap': props.wrap,
		'makosh-flex--inline': props.inline
	},
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
