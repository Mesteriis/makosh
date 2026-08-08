<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { CommunicationTone } from './Communication.types'

const props = withDefaults(defineProps<{
	label: string
	icon?: string
	tone?: CommunicationTone
	class?: string
}>(), {
	tone: 'accent'
})

const classes = computed(() => ['makosh-mention', `makosh-mention--${props.tone}`, props.class])
</script>

<template>
	<span :class="classes">
		<Icon v-if="icon" :icon="icon" size="1em" />
		<span>{{ label }}</span>
	</span>
</template>
