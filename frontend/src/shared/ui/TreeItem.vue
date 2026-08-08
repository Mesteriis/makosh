<script setup lang="ts">
import { computed } from 'vue'
import Icon from './Icon.vue'
import type { TreeItemData } from './Navigation.types'

defineOptions({ name: 'TreeItem' })

const props = withDefaults(defineProps<{
	item: TreeItemData
	selectedId?: string
	expanded?: string[]
	level?: number
}>(), {
	selectedId: '',
	expanded: () => [],
	level: 1
})

const emit = defineEmits<{
	select: [item: TreeItemData]
	toggle: [item: TreeItemData]
}>()

const hasChildren = computed(() => Boolean(props.item.children?.length))
const isExpanded = computed(() => props.expanded.includes(props.item.id))
const isSelected = computed(() => props.selectedId === props.item.id)

function handleClick(): void {
	if (props.item.disabled) {
		return
	}
	if (hasChildren.value) {
		emit('toggle', props.item)
	}
	if (props.item.static) {
		return
	}
	emit('select', props.item)
}

function handleKeydown(event: KeyboardEvent): void {
	if (['Enter', ' '].includes(event.key)) {
		event.preventDefault()
		handleClick()
	}
	if (event.key === 'ArrowRight' && hasChildren.value && !isExpanded.value) {
		event.preventDefault()
		emit('toggle', props.item)
	}
	if (event.key === 'ArrowLeft' && hasChildren.value && isExpanded.value) {
		event.preventDefault()
		emit('toggle', props.item)
	}
}
</script>

<template>
	<li
		class="makosh-tree-item"
		role="treeitem"
		:aria-disabled="item.disabled"
		:aria-expanded="hasChildren ? isExpanded : undefined"
		:aria-level="level"
		:aria-selected="isSelected"
	>
		<component
			:is="item.static ? 'div' : 'button'"
			class="makosh-tree-item__button"
			:class="{ 'makosh-tree-item__button--static': item.static }"
			:type="item.static ? undefined : 'button'"
			:disabled="item.static ? undefined : item.disabled"
			:tabindex="item.static ? undefined : (isSelected ? 0 : -1)"
			@click="handleClick"
			@keydown="handleKeydown"
		>
			<Icon
				v-if="hasChildren"
				:icon="isExpanded ? 'tabler:chevron-down' : 'tabler:chevron-right'"
				size="0.875rem"
				class="makosh-tree-item__chevron"
				aria-hidden="true"
			/>
			<span v-else class="makosh-tree-item__spacer" aria-hidden="true"></span>
			<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-tree-item__icon" aria-hidden="true" />
			<span class="makosh-tree-item__body">
				<span>{{ item.label }}</span>
				<span v-if="item.detail" class="makosh-tree-item__detail">{{ item.detail }}</span>
			</span>
			<span
				v-if="item.status"
				class="makosh-tree-item__status"
				:class="`makosh-tree-item__status--${item.status}`"
			>
				{{ item.status }}
			</span>
		</component>
		<ul v-if="hasChildren && isExpanded" class="makosh-tree-item__children" role="group">
			<TreeItem
				v-for="child in item.children"
				:key="child.id"
				:item="child"
				:expanded="expanded"
				:level="level + 1"
				:selected-id="selectedId"
				@select="emit('select', $event)"
				@toggle="emit('toggle', $event)"
			/>
		</ul>
	</li>
</template>
