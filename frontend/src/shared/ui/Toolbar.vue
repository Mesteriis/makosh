<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	orientation?: 'horizontal' | 'vertical'
	density?: 'compact' | 'default' | 'comfortable'
	label?: string
	class?: string
}>(), {
	as: 'div',
	orientation: 'horizontal',
	density: 'default'
})

const classes = computed(() => [
	'makosh-toolbar',
	`makosh-toolbar--${props.orientation}`,
	`makosh-toolbar--${props.density}`,
	props.class
])
</script>

<template>
	<component
		:is="as"
		:class="classes"
		role="toolbar"
		:aria-label="label"
		:aria-orientation="orientation"
	>
		<slot />
	</component>
</template>
