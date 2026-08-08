<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import IconButton from './IconButton.vue'
import type { CommunicationTone } from './Communication.types'

const props = withDefaults(defineProps<{
	name: string
	meta?: string
	icon?: string
	tone?: CommunicationTone
	removable?: boolean
	removeLabel?: string
	class?: string
}>(), {
	icon: 'tabler:paperclip',
	tone: 'neutral',
	removable: false,
	removeLabel: 'Remove attachment'
})

const emit = defineEmits<{
	remove: []
}>()

const classes = computed(() => [
	'makosh-attachment-chip',
	`makosh-attachment-chip--${props.tone}`,
	props.class
])
</script>

<template>
	<span :class="classes">
		<Icon :icon="icon" size="1rem" />
		<span class="makosh-attachment-chip__copy">
			<span class="makosh-attachment-chip__name">{{ name }}</span>
			<span v-if="meta" class="makosh-attachment-chip__meta">{{ meta }}</span>
		</span>
		<IconButton
			v-if="removable"
			icon="tabler:x"
			:label="removeLabel"
			size="sm"
			variant="ghost"
			@click="emit('remove')"
		/>
	</span>
</template>
