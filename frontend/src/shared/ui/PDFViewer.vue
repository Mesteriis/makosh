<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	src?: string
	title?: string
	description?: string
	fallbackLabel?: string
	class?: string
}>(), {
	fallbackLabel: 'PDF preview unavailable'
})

const classes = computed(() => ['makosh-pdf-viewer', props.class])
const viewerTitle = computed(() => props.title || props.fallbackLabel)
</script>

<template>
	<section :class="classes" :aria-label="viewerTitle">
		<header v-if="title || description" class="makosh-media-header">
			<h3 v-if="title" class="makosh-media-title">{{ title }}</h3>
			<p v-if="description" class="makosh-media-description">{{ description }}</p>
		</header>
		<object v-if="src" class="makosh-pdf-viewer__object" :data="src" type="application/pdf" :aria-label="viewerTitle">
			<div class="makosh-media-empty" role="status">
				<Icon icon="tabler:file-type-pdf" size="1.25rem" aria-hidden="true" />
				<span>{{ fallbackLabel }}</span>
			</div>
		</object>
		<div v-else class="makosh-media-empty" role="status">
			<Icon icon="tabler:file-type-pdf" size="1.25rem" aria-hidden="true" />
			<span>{{ fallbackLabel }}</span>
		</div>
	</section>
</template>
