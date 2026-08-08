<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Image from './Image.vue'
import type { MediaImageItem } from './Media.types'

const props = withDefaults(defineProps<{
	items: MediaImageItem[]
	selectedIndex?: number
	label?: string
	emptyLabel?: string
	class?: string
}>(), {
	selectedIndex: 0,
	label: 'Image gallery',
	emptyLabel: 'No images'
})

const emit = defineEmits<{
	'update:selectedIndex': [value: number]
	select: [item: MediaImageItem, index: number]
}>()

const activeIndex = ref(props.selectedIndex)

watch(() => props.selectedIndex, (value) => {
	activeIndex.value = value
})

const safeIndex = computed(() => {
	if (props.items.length === 0) {
		return 0
	}
	return Math.min(Math.max(activeIndex.value, 0), props.items.length - 1)
})
const selectedItem = computed(() => props.items[safeIndex.value])
const classes = computed(() => ['makosh-image-gallery', props.class])

function selectImage(index: number): void {
	const item = props.items[index]
	if (!item) {
		return
	}
	activeIndex.value = index
	emit('update:selectedIndex', index)
	emit('select', item, index)
}
</script>

<template>
	<section :class="classes" :aria-label="label">
		<div v-if="selectedItem" class="makosh-image-gallery__stage">
			<Image
				:src="selectedItem.src"
				:alt="selectedItem.alt"
				:caption="selectedItem.title"
				ratio="wide"
				fit="contain"
			/>
			<p v-if="selectedItem.description" class="makosh-image-gallery__description">
				{{ selectedItem.description }}
			</p>
		</div>
		<div v-else class="makosh-image-gallery__empty">{{ emptyLabel }}</div>

		<div v-if="items.length > 1" class="makosh-image-gallery__thumbs" role="list">
			<div
				v-for="(item, index) in items"
				:key="item.id"
				role="listitem"
			>
				<button
					class="makosh-image-gallery__thumb"
					:class="{ 'makosh-image-gallery__thumb--active': index === safeIndex }"
					type="button"
					:aria-label="item.title || item.alt"
					:aria-pressed="index === safeIndex"
					@click="selectImage(index)"
				>
					<Image :src="item.src" :alt="item.alt" ratio="square" />
				</button>
			</div>
		</div>
	</section>
</template>
