<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

type StatCardTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger'

const props = withDefaults(defineProps<{
	label: string
	value: string | number
	description?: string
	trend?: string
	tone?: StatCardTone
	icon?: string
	class?: string
}>(), {
	tone: 'neutral'
})

const classes = computed(() => [
	'makosh-stat-card',
	`makosh-stat-card--${props.tone}`,
	props.class
])
</script>

<template>
	<article :class="classes">
		<Icon
			v-if="icon"
			:icon="icon"
			size="1.25rem"
			class="makosh-stat-card-icon"
		/>
		<div class="makosh-stat-card-body">
			<span class="makosh-stat-card-label">{{ label }}</span>
			<strong class="makosh-stat-card-value">{{ value }}</strong>
			<span v-if="trend" class="makosh-stat-card-trend">{{ trend }}</span>
			<p v-if="description" class="makosh-stat-card-description">{{ description }}</p>
		</div>
	</article>
</template>
