<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	position?: 'left' | 'right' | 'top' | 'bottom'
	label?: string
	compact?: boolean
	class?: string
}>(), {
	as: 'nav',
	position: 'left',
	compact: false
})

const classes = computed(() => [
	'makosh-dock',
	`makosh-dock--${props.position}`,
	{ 'makosh-dock--compact': props.compact },
	props.class
])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<slot />
	</component>
</template>
