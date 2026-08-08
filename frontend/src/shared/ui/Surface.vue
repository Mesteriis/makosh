<script setup lang="ts">
import { computed } from 'vue'

type SurfaceTone = 'default' | 'muted' | 'raised' | 'deep'
type SurfacePadding = 'none' | 'sm' | 'md' | 'lg'
type SurfaceRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: SurfaceTone
	padding?: SurfacePadding
	radius?: SurfaceRadius
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
	'makosh-surface',
	`makosh-surface--${props.tone}`,
	`makosh-surface--padding-${props.padding}`,
	`makosh-surface--radius-${props.radius}`,
	props.bordered && 'makosh-surface--bordered',
	props.clip && 'makosh-surface--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
