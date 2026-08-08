<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { MessageDeliveryState } from './Communication.types'

const props = withDefaults(defineProps<{
	status?: MessageDeliveryState
	label?: string
	showLabel?: boolean
	class?: string
}>(), {
	status: 'sent',
	showLabel: true
})

const statusMeta: Record<MessageDeliveryState, { icon: string; label: string }> = {
	queued: { icon: 'tabler:clock', label: 'Queued' },
	sent: { icon: 'tabler:check', label: 'Sent' },
	delivered: { icon: 'tabler:checks', label: 'Delivered' },
	read: { icon: 'tabler:eye-check', label: 'Read' },
	failed: { icon: 'tabler:alert-triangle', label: 'Failed' }
}

const current = computed(() => statusMeta[props.status])
const resolvedLabel = computed(() => props.label ?? current.value.label)
const classes = computed(() => ['makosh-message-status', `makosh-message-status--${props.status}`, props.class])
</script>

<template>
	<span :class="classes" :aria-label="resolvedLabel">
		<Icon :icon="current.icon" size="1rem" />
		<span v-if="showLabel">{{ resolvedLabel }}</span>
	</span>
</template>
