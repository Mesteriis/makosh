<script setup lang="ts">
import { ref } from 'vue'
import { useTelegramTgsSticker } from '../queries/useTelegramTgsSticker'

const props = defineProps<{
	url: string
	alt?: string
}>()

const container = ref<HTMLElement>()
const { failed } = useTelegramTgsSticker(() => props.url, container)
</script>

<template>
	<span
		ref="container"
		class="telegram-tgs-sticker"
		:role="alt ? 'img' : undefined"
		:aria-label="alt || undefined"
	/>
	<span v-if="failed" class="telegram-provider-media__unavailable">Sticker unavailable</span>
</template>
