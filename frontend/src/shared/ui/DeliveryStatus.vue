<script setup lang="ts">
import { computed } from 'vue'
import MessageStatus from './MessageStatus.vue'
import type { MessageDeliveryState } from './Communication.types'

const props = withDefaults(defineProps<{
	status?: MessageDeliveryState
	label?: string
	description?: string
	timestamp?: string
	class?: string
}>(), {
	status: 'sent'
})

const classes = computed(() => ['makosh-delivery-status', `makosh-delivery-status--${props.status}`, props.class])
</script>

<template>
	<div :class="classes">
		<MessageStatus :status="status" :label="label" />
		<span v-if="description" class="makosh-delivery-status__description">{{ description }}</span>
		<time v-if="timestamp" class="makosh-delivery-status__time">{{ timestamp }}</time>
	</div>
</template>
