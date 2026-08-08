<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	ariaLabel?: string
	orientation?: 'horizontal' | 'vertical'
	class?: string
}>(), {
	ariaLabel: 'Button group',
	orientation: 'horizontal'
})

const classes = computed(() => [
	'makosh-button-group',
	`makosh-button-group--${props.orientation}`,
	props.class
])
</script>

<template>
	<div :class="classes" role="group" :aria-label="ariaLabel">
		<slot />
	</div>
</template>
