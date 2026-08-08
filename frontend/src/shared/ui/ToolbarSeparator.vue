<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	orientation?: 'horizontal' | 'vertical'
	label?: string
	decorative?: boolean
	class?: string
}>(), {
	orientation: 'vertical',
	decorative: true
})

const classes = computed(() => [
	'makosh-toolbar-separator',
	`makosh-toolbar-separator--${props.orientation}`,
	props.class
])
</script>

<template>
	<span
		v-if="decorative"
		:class="classes"
		aria-hidden="true"
	/>
	<span
		v-else
		:class="classes"
		role="separator"
		:aria-label="label"
		:aria-orientation="orientation"
	/>
</template>
