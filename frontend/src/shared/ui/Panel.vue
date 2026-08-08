<script setup lang="ts">
import { computed } from 'vue'

type PanelTone = 'default' | 'muted' | 'raised' | 'deep'
type PanelPadding = 'none' | 'sm' | 'md' | 'lg'
type PanelRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: PanelTone
	padding?: PanelPadding
	radius?: PanelRadius
	bordered?: boolean
	clip?: boolean
	class?: string
}>(), {
	as: 'section',
	tone: 'default',
	padding: 'md',
	radius: 'md',
	bordered: true,
	clip: false
})

const classes = computed(() => [
	'makosh-panel',
	`makosh-panel--${props.tone}`,
	`makosh-panel--padding-${props.padding}`,
	`makosh-panel--radius-${props.radius}`,
	props.bordered && 'makosh-panel--bordered',
	props.clip && 'makosh-panel--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
