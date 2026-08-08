<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
	level?: 1 | 2 | 3 | 4 | 5 | 6
	as?: string
	class?: string
}>(), {
	level: 2
})

const tagName = computed(() => props.as ?? `h${props.level}`)
const classes = computed(() => ['makosh-heading', `makosh-heading--${props.level}`, props.class])
</script>

<template>
	<component :is="tagName" :class="classes">
		<slot />
	</component>
</template>
