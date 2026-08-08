<script setup lang="ts">
import { computed } from 'vue'
import { sanitizeHtml } from './Media.rendering'

const props = withDefaults(defineProps<{
	content?: string
	format?: 'html' | 'text'
	sanitized?: boolean
	isolated?: boolean
	title?: string
	unsafeLabel?: string
	emptyLabel?: string
	class?: string
}>(), {
	content: '',
	format: 'text',
	sanitized: false,
	isolated: false,
	unsafeLabel: 'HTML preview requires sanitized content',
	emptyLabel: 'No preview content'
})

const classes = computed(() => [
	'makosh-html-preview',
	`makosh-html-preview--${props.format}`,
	{
		'makosh-html-preview--blocked': props.format === 'html' && !props.sanitized
	},
	props.class
])
const hasContent = computed(() => props.content.trim().length > 0)
const canRenderHtml = computed(() => props.format === 'html' && props.sanitized && hasContent.value)
const shouldIsolateHtml = computed(() => canRenderHtml.value && props.isolated)
const safeHtml = computed(() => {
	if (!canRenderHtml.value) {
		return ''
	}
	return sanitizeHtml(props.content)
})
</script>

<template>
	<article :class="classes">
		<h3 v-if="title" class="makosh-media-title">{{ title }}</h3>
		<iframe
			v-if="shouldIsolateHtml"
			class="makosh-html-preview__frame"
			sandbox="allow-same-origin"
			:srcdoc="safeHtml"
			:title="title || 'HTML preview'"
		/>
		<div v-else-if="canRenderHtml" class="makosh-html-preview__content" v-html="safeHtml" />
		<pre v-else-if="hasContent" class="makosh-html-preview__text">{{ content }}</pre>
		<p v-else class="makosh-media-empty">{{ emptyLabel }}</p>
		<p v-if="format === 'html' && hasContent && !sanitized" class="makosh-html-preview__safety">
			{{ unsafeLabel }}
		</p>
	</article>
</template>
