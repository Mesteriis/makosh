<script setup lang="ts">
import { computed } from 'vue'

type WellTone = 'default' | 'muted' | 'inset'
type WellPadding = 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: WellTone
	padding?: WellPadding
	class?: string
}>(), {
	as: 'div',
	tone: 'default',
	padding: 'md'
})

const classes = computed(() => [
	'makosh-well',
	`makosh-well--${props.tone}`,
	`makosh-well--padding-${props.padding}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
