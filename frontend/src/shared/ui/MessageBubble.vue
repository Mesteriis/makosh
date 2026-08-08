<script setup lang="ts">
import { computed } from 'vue'
import type { CommunicationTone, MessageDirection } from './Communication.types'

const props = withDefaults(defineProps<{
	author?: string
	timestamp?: string
	meta?: string
	direction?: MessageDirection
	tone?: CommunicationTone
	selected?: boolean
	pending?: boolean
	class?: string
}>(), {
	direction: 'inbound',
	tone: 'neutral',
	selected: false,
	pending: false
})

const classes = computed(() => [
	'makosh-message-bubble',
	`makosh-message-bubble--${props.direction}`,
	`makosh-message-bubble--${props.tone}`,
	{
		'makosh-message-bubble--selected': props.selected,
		'makosh-message-bubble--pending': props.pending
	},
	props.class
])
</script>

<template>
	<article :class="classes">
		<header v-if="author || timestamp || meta" class="makosh-message-bubble__header">
			<strong v-if="author" class="makosh-message-bubble__author">{{ author }}</strong>
			<span v-if="timestamp" class="makosh-message-bubble__time">{{ timestamp }}</span>
			<span v-if="meta" class="makosh-message-bubble__meta">{{ meta }}</span>
		</header>
		<div class="makosh-message-bubble__body">
			<slot />
		</div>
		<footer v-if="$slots.footer || pending" class="makosh-message-bubble__footer">
			<slot name="footer" />
			<span v-if="pending" class="makosh-message-bubble__pending">Pending</span>
		</footer>
	</article>
</template>
