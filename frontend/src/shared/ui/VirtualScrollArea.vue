<script setup lang="ts">
import { computed } from 'vue'
import ScrollArea from './ScrollArea.vue'

const props = withDefaults(defineProps<{
	as?: string
	label?: string
	total?: number
	visibleStart?: number
	visibleCount?: number
	emptyText?: string
	class?: string
}>(), {
	as: 'section',
	total: 0,
	visibleStart: 0,
	visibleCount: 8,
	emptyText: 'No items'
})

const safeStart = computed(() => Math.max(0, Math.min(props.visibleStart, props.total)))
const safeEnd = computed(() => Math.min(props.total, safeStart.value + Math.max(0, props.visibleCount)))
const rangeLabel = computed(() => {
	if (props.total === 0) return props.emptyText
	return `${safeStart.value + 1}-${safeEnd.value} / ${props.total}`
})
const classes = computed(() => ['makosh-virtual-scroll-area', props.class])
</script>

<template>
	<component :is="as" :class="classes" :aria-label="label">
		<div class="makosh-virtual-scroll-area__meta">{{ rangeLabel }}</div>
		<ScrollArea class="makosh-virtual-scroll-area__viewport">
			<slot :visible-start="safeStart" :visible-end="safeEnd" />
		</ScrollArea>
	</component>
</template>
