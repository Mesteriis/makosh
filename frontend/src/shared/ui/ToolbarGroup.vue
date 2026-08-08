<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	orientation?: 'horizontal' | 'vertical'
	label?: string
	class?: string
}>(), {
	as: 'div',
	orientation: 'horizontal'
})

const classes = computed(() => [
	'makosh-toolbar-group',
	`makosh-toolbar-group--${props.orientation}`,
	props.class
])
</script>

<template>
	<component
		:is="as"
		:class="classes"
		role="group"
		:aria-label="label"
	>
		<slot />
	</component>
</template>
