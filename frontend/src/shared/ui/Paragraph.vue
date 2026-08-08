<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	tone?: 'default' | 'muted' | 'soft'
	class?: string
}>(), {
	tone: 'default'
})

const classes = computed(() => ['makosh-paragraph', `makosh-paragraph--${props.tone}`, props.class])
</script>

<template>
	<p :class="classes">
		<slot />
	</p>
</template>
