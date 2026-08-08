<script setup lang="ts">
import { computed } from 'vue'
import TreeItem from './TreeItem.vue'
import type { TreeItemData } from './Navigation.types'

const props = withDefaults(defineProps<{
	items?: TreeItemData[]
	modelValue?: string
	expanded?: string[]
	label?: string
	class?: string
}>(), {
	items: () => [],
	modelValue: '',
	expanded: () => [],
	label: 'Tree'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	'update:expanded': [value: string[]]
	select: [item: TreeItemData]
}>()

const classes = computed(() => ['makosh-tree', props.class])

function toggleItem(item: TreeItemData): void {
	const expanded = new Set(props.expanded)
	if (expanded.has(item.id)) {
		expanded.delete(item.id)
	} else {
		expanded.add(item.id)
	}
	emit('update:expanded', Array.from(expanded))
}

function selectItem(item: TreeItemData): void {
	if (item.disabled) {
		return
	}
	emit('update:modelValue', item.id)
	emit('select', item)
}
</script>

<template>
	<ul :class="classes" role="tree" :aria-label="label">
		<TreeItem
			v-for="item in items"
			:key="item.id"
			:item="item"
			:expanded="expanded"
			:selected-id="modelValue"
			@select="selectItem"
			@toggle="toggleItem"
		/>
	</ul>
</template>
