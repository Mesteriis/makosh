<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
	src?: string
	alt?: string
	caption?: string
	fit?: 'cover' | 'contain'
	ratio?: 'auto' | 'square' | 'video' | 'wide'
	loading?: 'lazy' | 'eager'
	fallbackLabel?: string
	class?: string
}>(), {
	alt: '',
	fit: 'cover',
	ratio: 'auto',
	loading: 'lazy',
	fallbackLabel: 'Image unavailable'
})

const failed = ref(false)

watch(() => props.src, () => {
	failed.value = false
})

const canRenderImage = computed(() => Boolean(props.src) && !failed.value)
const classes = computed(() => [
	'makosh-image',
	`makosh-image--fit-${props.fit}`,
	`makosh-image--ratio-${props.ratio}`,
	props.class
])
const fallbackLabel = computed(() => props.alt || props.fallbackLabel)
</script>

<template>
	<figure :class="classes">
		<img
			v-if="canRenderImage"
			class="makosh-image__asset"
			:src="src"
			:alt="alt"
			:loading="loading"
			@error="failed = true"
		/>
		<div v-else class="makosh-image__fallback" role="img" :aria-label="fallbackLabel">
			<Icon icon="tabler:photo-off" size="1.5rem" aria-hidden="true" />
			<span>{{ fallbackLabel }}</span>
		</div>
		<figcaption v-if="caption" class="makosh-image__caption">{{ caption }}</figcaption>
	</figure>
</template>
