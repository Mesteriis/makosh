<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	as?: string
	size?: 'narrow' | 'default' | 'wide' | 'full'
	class?: string
}>(), {
	as: 'div',
	size: 'default'
})

const classes = computed(() => ['makosh-container', `makosh-container--${props.size}`, props.class])
</script>

<template>
	<component :is="as" :class="classes">
		<slot />
	</component>
</template>
