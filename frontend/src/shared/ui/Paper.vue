<script setup lang="ts">
import { computed } from 'vue'

type PaperTone = 'default' | 'muted' | 'raised' | 'deep'
type PaperPadding = 'none' | 'sm' | 'md' | 'lg'
type PaperRadius = 'none' | 'sm' | 'md' | 'lg'

const props = withDefaults(defineProps<{
	as?: string
	tone?: PaperTone
	padding?: PaperPadding
	radius?: PaperRadius
	bordered?: boolean
	clip?: boolean
	class?: string
}>(), {
	as: 'article',
	tone: 'default',
	padding: 'md',
	radius: 'md',
	bordered: true,
	clip: false
})

const classes = computed(() => [
	'makosh-paper',
	`makosh-paper--${props.tone}`,
	`makosh-paper--padding-${props.padding}`,
	`makosh-paper--radius-${props.radius}`,
	props.bordered && 'makosh-paper--bordered',
	props.clip && 'makosh-paper--clip',
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
