<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	orientation?: 'horizontal' | 'vertical'
	class?: string
}>(), {
	as: 'span',
	orientation: 'horizontal'
})

const classes = computed(() => [
	'makosh-spacer',
	`makosh-spacer--${props.orientation}`,
	props.class
])
</script>

<template>
	<component
		:is="as"
		:class="classes"
		aria-hidden="true"
	/>
</template>
