<script setup lang="ts">
import { computed } from 'vue'
import type { LayoutTone } from './Layout.types'

const props = withDefaults(defineProps<{
	as?: string
	axis?: 'horizontal' | 'vertical' | 'both'
	tone?: LayoutTone
	class?: string
}>(), {
	as: 'section',
	axis: 'horizontal',
	tone: 'default'
})

const classes = computed(() => [
	'makosh-resizable',
	`makosh-resizable--${props.axis}`,
	`makosh-resizable--${props.tone}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
