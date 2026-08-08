<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	title?: string
	description?: string
	density?: 'compact' | 'default'
	class?: string
}>(), {
	as: 'header',
	density: 'default'
})

const classes = computed(() => [
	'makosh-top-bar',
	`makosh-top-bar--${props.density}`,
	props.class
])
</script>

<template>
	<component :is="as" :class="classes">
		<div v-if="$slots.start" class="makosh-top-bar__slot makosh-top-bar__slot--start">
			<slot name="start" />
		</div>
		<div class="makosh-top-bar__main">
			<strong v-if="title" class="makosh-top-bar__title">{{ title }}</strong>
			<span v-if="description" class="makosh-top-bar__description">{{ description }}</span>
			<slot />
		</div>
		<div v-if="$slots.end" class="makosh-top-bar__slot makosh-top-bar__slot--end">
			<slot name="end" />
		</div>
	</component>
</template>
