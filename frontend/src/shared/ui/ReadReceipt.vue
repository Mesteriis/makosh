<script setup lang="ts">
import { computed } from 'vue'
import Avatar from './Avatar.vue'
import type { ReadReceiptItem } from './Communication.types'

const props = withDefaults(defineProps<{
	items?: ReadReceiptItem[]
	label?: string
	visibleCount?: number
	class?: string
}>(), {
	items: () => [],
	label: 'Read receipts',
	visibleCount: 3
})

const visibleItems = computed(() => props.items.slice(0, props.visibleCount))
const hiddenCount = computed(() => Math.max(props.items.length - props.visibleCount, 0))
const classes = computed(() => ['makosh-read-receipt', props.class])
</script>

<template>
	<div :class="classes" role="group" :aria-label="label">
		<div class="makosh-read-receipt__avatars" aria-hidden="true">
			<Avatar
				v-for="item in visibleItems"
				:key="item.id"
				:alt="item.label"
				:fallback="item.initials || item.label.slice(0, 2)"
				:src="item.src"
				size="sm"
			/>
			<span v-if="hiddenCount > 0" class="makosh-read-receipt__more">+{{ hiddenCount }}</span>
		</div>
		<span class="makosh-read-receipt__label">
			<slot>{{ label }}</slot>
		</span>
	</div>
</template>
