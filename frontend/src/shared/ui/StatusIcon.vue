<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { StatusIconKind } from './Utility.types'

const props = withDefaults(defineProps<{
	status?: StatusIconKind
	label?: string
	size?: number | string
	class?: string
}>(), {
	status: 'idle',
	size: '1.25rem'
})

const statusIcons: Record<StatusIconKind, string> = {
	idle: 'tabler:circle',
	active: 'tabler:activity',
	success: 'tabler:circle-check',
	warning: 'tabler:alert-triangle',
	danger: 'tabler:circle-x',
	offline: 'tabler:wifi-off',
	syncing: 'tabler:refresh'
}

const classes = computed(() => ['makosh-status-icon', `makosh-status-icon--${props.status}`, props.class])
const accessibleLabel = computed(() => props.label ?? props.status)
</script>

<template>
	<span :class="classes" role="img" :aria-label="accessibleLabel">
		<Icon :icon="statusIcons[status]" :size="size" />
	</span>
</template>
