<script setup lang="ts">
import { computed } from 'vue'
import Image from './Image.vue'

const props = withDefaults(defineProps<{
	src?: string
	alt?: string
	title?: string
	description?: string
	meta?: string
	ratio?: 'auto' | 'square' | 'video' | 'wide'
	class?: string
}>(), {
	alt: '',
	ratio: 'video'
})

const classes = computed(() => ['makosh-image-preview', props.class])
</script>

<template>
	<article :class="classes">
		<Image :src="src" :alt="alt" :ratio="ratio" fit="cover" />
		<div v-if="title || description || meta || $slots.actions" class="makosh-image-preview__body">
			<div class="makosh-image-preview__copy">
				<h3 v-if="title" class="makosh-image-preview__title">{{ title }}</h3>
				<p v-if="description" class="makosh-image-preview__description">{{ description }}</p>
				<span v-if="meta" class="makosh-image-preview__meta">{{ meta }}</span>
			</div>
			<div v-if="$slots.actions" class="makosh-image-preview__actions">
				<slot name="actions" />
			</div>
		</div>
	</article>
</template>
