<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdownToSafeHtml } from './Media.rendering'

const props = withDefaults(defineProps<{
	source?: string
	title?: string
	emptyLabel?: string
	class?: string
}>(), {
	source: '',
	emptyLabel: 'No markdown content'
})

const classes = computed(() => ['makosh-markdown-viewer', props.class])
const renderedHtml = computed(() => renderMarkdownToSafeHtml(props.source))
</script>

<template>
	<article :class="classes">
		<h3 v-if="title" class="makosh-media-title">{{ title }}</h3>
		<div v-if="renderedHtml" class="makosh-markdown-viewer__content" v-html="renderedHtml" />
		<p v-else class="makosh-media-empty">{{ emptyLabel }}</p>
	</article>
</template>
