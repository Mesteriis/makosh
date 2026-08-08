<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { FileIconKind } from './Utility.types'

const props = withDefaults(defineProps<{
	kind?: FileIconKind
	mimeType?: string
	label?: string
	size?: number | string
	class?: string
}>(), {
	kind: 'generic',
	size: '1.25rem'
})

const fileIcons: Record<FileIconKind, string> = {
	image: 'tabler:file-type-png',
	audio: 'tabler:file-music',
	video: 'tabler:file-film',
	pdf: 'tabler:file-type-pdf',
	code: 'tabler:file-code',
	archive: 'tabler:file-zip',
	spreadsheet: 'tabler:file-spreadsheet',
	text: 'tabler:file-text',
	generic: 'tabler:file'
}

const resolvedKind = computed<FileIconKind>(() => {
	if (!props.mimeType) {
		return props.kind
	}
	if (props.mimeType.startsWith('image/')) {
		return 'image'
	}
	if (props.mimeType.startsWith('audio/')) {
		return 'audio'
	}
	if (props.mimeType.startsWith('video/')) {
		return 'video'
	}
	if (props.mimeType.includes('pdf')) {
		return 'pdf'
	}
	if (props.mimeType.includes('zip') || props.mimeType.includes('archive')) {
		return 'archive'
	}
	if (props.mimeType.includes('sheet') || props.mimeType.includes('csv')) {
		return 'spreadsheet'
	}
	if (props.mimeType.includes('json') || props.mimeType.includes('javascript') || props.mimeType.includes('typescript')) {
		return 'code'
	}
	if (props.mimeType.startsWith('text/')) {
		return 'text'
	}
	return props.kind
})
const classes = computed(() => ['makosh-file-icon', `makosh-file-icon--${resolvedKind.value}`, props.class])
const accessibleLabel = computed(() => props.label ?? resolvedKind.value)
</script>

<template>
	<span :class="classes" role="img" :aria-label="accessibleLabel">
		<Icon :icon="fileIcons[resolvedKind]" :size="size" />
	</span>
</template>
