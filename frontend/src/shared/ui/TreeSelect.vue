<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'
import Icon from './Icon.vue'
import type { TreeSelectOption } from './Selection.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

interface VisibleTreeItem {
	option: TreeSelectOption
	level: number
	parentValue?: string
	hasChildren: boolean
	isExpanded: boolean
	isDisabled: boolean
	isSelectable: boolean
	posInSet: number
	setSize: number
}

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: TreeSelectOption[]
	placeholder?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	emptyLabel?: string
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	disabled: false,
	readonly: false,
	emptyLabel: 'No options'
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: TreeSelectOption]
	open: []
	close: []
}>()

const isOpen = ref(false)
const expandedIds = ref<Set<string>>(new Set())
const activeIndex = ref(0)
const rootRef = ref<HTMLElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)
const treeRef = ref<HTMLElement | null>(null)
const isPopoverPositioned = ref(false)
const componentId = `makosh-tree-select-${useId()}`
let isTrackingPopoverGeometry = false
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeTree, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, popoverRef.value]
})

const classes = computed(() => ['makosh-tree-select', props.class])
const treeId = computed(() => `${componentId}-tree`)
const canInteract = computed(() => !props.disabled && !props.readonly)
const accessibleLabel = computed(() => props.ariaLabel ?? props.placeholder)
const treeAriaLabel = computed(() => `${accessibleLabel.value} options`)
const selectedLeafPath = computed(() => findSelectableLeafPath(props.options, props.modelValue))
const selectedOption = computed(() => selectedLeafPath.value?.at(-1))
const visibleItems = computed(() => flattenVisibleTree(props.options))
const activeItem = computed(() => visibleItems.value[activeIndex.value])
const activeItemId = computed(() => {
	if (!isOpen.value || !activeItem.value) {
		return undefined
	}
	return treeItemId(activeIndex.value)
})
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)

watch(visibleItems, () => {
	clampActiveIndex()
})

watch(() => props.modelValue, () => {
	if (!isOpen.value) {
		return
	}
	expandSelectedAncestors()
	setActiveIndexToSelected()
	scrollActiveItemIntoView()
})

function hasChildren(option: TreeSelectOption): boolean {
	return Boolean(option.children?.length)
}

function flattenVisibleTree(
	options: TreeSelectOption[],
	level = 1,
	parentValue?: string,
	ancestorDisabled = false
): VisibleTreeItem[] {
	const items: VisibleTreeItem[] = []
	for (const [index, option] of options.entries()) {
		const optionHasChildren = hasChildren(option)
		const isDisabled = ancestorDisabled || Boolean(option.disabled)
		const isExpanded = !isDisabled && expandedIds.value.has(option.value)
		items.push({
			option,
			level,
			parentValue,
			hasChildren: optionHasChildren,
			isExpanded,
			isDisabled,
			isSelectable: !isDisabled && !optionHasChildren,
			posInSet: index + 1,
			setSize: options.length
		})
		if (optionHasChildren && isExpanded) {
			items.push(...flattenVisibleTree(option.children ?? [], level + 1, option.value, isDisabled))
		}
	}
	return items
}

function findSelectableLeafPath(
	options: TreeSelectOption[],
	value: string,
	ancestors: TreeSelectOption[] = []
): TreeSelectOption[] | undefined {
	for (const option of options) {
		const path = [...ancestors, option]
		if (option.value === value) {
			if (hasChildren(option) || path.some((pathOption) => pathOption.disabled)) {
				return undefined
			}
			return path
		}
		const childPath = findSelectableLeafPath(option.children ?? [], value, path)
		if (childPath) {
			return childPath
		}
	}
	return undefined
}

function treeItemId(index: number): string {
	return `${treeId.value}-item-${index}`
}

function expandSelectedAncestors(): void {
	const path = selectedLeafPath.value
	if (!path || path.length <= 1) {
		return
	}
	const nextExpandedIds = new Set(expandedIds.value)
	for (const option of path.slice(0, -1)) {
		if (option.disabled) {
			return
		}
		if (hasChildren(option)) {
			nextExpandedIds.add(option.value)
		}
	}
	expandedIds.value = nextExpandedIds
}

function setActiveIndexToSelected(): void {
	const selectedIndex = visibleItems.value.findIndex((item) => item.isSelectable && item.option.value === props.modelValue)
	activeIndex.value = selectedIndex >= 0 ? selectedIndex : 0
	clampActiveIndex()
}

function clampActiveIndex(): void {
	if (visibleItems.value.length === 0) {
		activeIndex.value = 0
		return
	}
	activeIndex.value = Math.min(Math.max(activeIndex.value, 0), visibleItems.value.length - 1)
}

function scrollActiveItemIntoView(): void {
	void nextTick(() => {
		if (!activeItem.value) {
			return
		}
		const activeElement = treeRef.value?.children.item(activeIndex.value)
		if (activeElement instanceof HTMLElement) {
			activeElement.scrollIntoView({ block: 'nearest' })
		}
	})
}

function getTriggerElement(): HTMLElement | null {
	return rootRef.value?.querySelector<HTMLElement>('.makosh-tree-select__trigger') ?? null
}

function updatePopoverGeometry(): void {
	const triggerElement = getTriggerElement()
	const popoverElement = popoverRef.value
	if (!triggerElement || !popoverElement || typeof window === 'undefined') {
		return
	}

	const rect = triggerElement.getBoundingClientRect()
	const viewportPadding = 12
	const gap = 4
	const width = Math.max(160, Math.min(rect.width, window.innerWidth - viewportPadding * 2))
	const left = Math.min(Math.max(rect.left, viewportPadding), window.innerWidth - width - viewportPadding)
	const preferredTop = rect.bottom + gap
	const availableBelow = window.innerHeight - preferredTop - viewportPadding
	const availableAbove = rect.top - gap - viewportPadding
	const shouldOpenAbove = availableBelow < 160 && availableAbove > availableBelow
	const availableHeight = shouldOpenAbove ? availableAbove : availableBelow
	const maxHeight = Math.max(160, Math.min(360, availableHeight))
	const top = shouldOpenAbove ? Math.max(viewportPadding, rect.top - gap - maxHeight) : preferredTop

	popoverElement.style.setProperty('--h-tree-select-popover-left', `${Math.round(left)}px`)
	popoverElement.style.setProperty('--h-tree-select-popover-top', `${Math.round(top)}px`)
	popoverElement.style.setProperty('--h-tree-select-popover-width', `${Math.round(width)}px`)
	popoverElement.style.setProperty('--h-tree-select-popover-max-height', `${Math.round(maxHeight)}px`)
	isPopoverPositioned.value = true
}

function schedulePopoverGeometryUpdate(): void {
	void nextTick(updatePopoverGeometry)
}

function addPopoverGeometryListeners(): void {
	if (isTrackingPopoverGeometry || typeof window === 'undefined') {
		return
	}

	window.addEventListener('resize', updatePopoverGeometry)
	window.addEventListener('scroll', updatePopoverGeometry, true)
	isTrackingPopoverGeometry = true
}

function removePopoverGeometryListeners(): void {
	if (!isTrackingPopoverGeometry || typeof window === 'undefined') {
		return
	}

	window.removeEventListener('resize', updatePopoverGeometry)
	window.removeEventListener('scroll', updatePopoverGeometry, true)
	isTrackingPopoverGeometry = false
}

function openTree(): void {
	if (!canInteract.value) {
		return
	}
	cancelMouseLeaveDismiss()
	expandSelectedAncestors()
	setActiveIndexToSelected()
	if (isOpen.value) {
		schedulePopoverGeometryUpdate()
		scrollActiveItemIntoView()
		return
	}
	isPopoverPositioned.value = false
	isOpen.value = true
	addPopoverGeometryListeners()
	emit('open')
	schedulePopoverGeometryUpdate()
	scrollActiveItemIntoView()
}

function closeTree(): void {
	cancelMouseLeaveDismiss()
	removePopoverGeometryListeners()
	if (!isOpen.value) {
		return
	}
	isOpen.value = false
	isPopoverPositioned.value = false
	activeIndex.value = 0
	emit('close')
}

function toggleTree(): void {
	if (isOpen.value) {
		closeTree()
		return
	}
	openTree()
}

function expandItem(item: VisibleTreeItem): void {
	if (!canInteract.value || item.isDisabled || !item.hasChildren) {
		return
	}
	const nextExpandedIds = new Set(expandedIds.value)
	nextExpandedIds.add(item.option.value)
	expandedIds.value = nextExpandedIds
	clampActiveIndex()
}

function collapseItem(item: VisibleTreeItem): void {
	if (!canInteract.value || item.isDisabled || !item.hasChildren) {
		return
	}
	const nextExpandedIds = new Set(expandedIds.value)
	nextExpandedIds.delete(item.option.value)
	expandedIds.value = nextExpandedIds
	clampActiveIndex()
}

function toggleItem(item: VisibleTreeItem): void {
	if (item.isExpanded) {
		collapseItem(item)
		return
	}
	expandItem(item)
}

function selectLeaf(item: VisibleTreeItem): void {
	if (!canInteract.value || !item.isSelectable) {
		return
	}
	emit('update:modelValue', item.option.value)
	emit('select', item.option)
	closeTree()
}

function activateItem(item: VisibleTreeItem | undefined): void {
	if (!item || item.isDisabled) {
		return
	}
	if (item.hasChildren) {
		toggleItem(item)
		return
	}
	selectLeaf(item)
}

function setActiveIndex(index: number): void {
	activeIndex.value = index
	scrollActiveItemIntoView()
}

function setActiveIndexFromPointer(index: number): void {
	activeIndex.value = index
}

function moveActiveIndex(delta: number): void {
	if (visibleItems.value.length === 0) {
		return
	}
	activeIndex.value = Math.min(Math.max(activeIndex.value + delta, 0), visibleItems.value.length - 1)
	scrollActiveItemIntoView()
}

function findVisibleParentIndex(item: VisibleTreeItem): number {
	if (!item.parentValue) {
		return -1
	}
	return visibleItems.value.findIndex((candidate) => candidate.option.value === item.parentValue)
}

function handleArrowRight(item: VisibleTreeItem): void {
	if (!item.hasChildren) {
		return
	}
	if (!item.isExpanded) {
		expandItem(item)
		scrollActiveItemIntoView()
		return
	}
	const nextItem = visibleItems.value[activeIndex.value + 1]
	if (nextItem && nextItem.level === item.level + 1) {
		setActiveIndex(activeIndex.value + 1)
	}
}

function handleArrowLeft(item: VisibleTreeItem): void {
	if (item.hasChildren && item.isExpanded) {
		collapseItem(item)
		scrollActiveItemIntoView()
		return
	}
	const parentIndex = findVisibleParentIndex(item)
	if (parentIndex >= 0) {
		setActiveIndex(parentIndex)
	}
}

function handleKeydown(event: KeyboardEvent): void {
	if (event.key === 'Escape') {
		closeTree()
		return
	}
	if (!isOpen.value && ['ArrowDown', 'Enter', ' '].includes(event.key)) {
		event.preventDefault()
		openTree()
		return
	}
	if (!isOpen.value || visibleItems.value.length === 0) {
		return
	}
	const item = activeItem.value
	if (!item) {
		return
	}
	if (event.key === 'ArrowDown') {
		event.preventDefault()
		moveActiveIndex(1)
	}
	if (event.key === 'ArrowUp') {
		event.preventDefault()
		moveActiveIndex(-1)
	}
	if (event.key === 'Home') {
		event.preventDefault()
		setActiveIndex(0)
	}
	if (event.key === 'End') {
		event.preventDefault()
		setActiveIndex(visibleItems.value.length - 1)
	}
	if (event.key === 'ArrowRight') {
		event.preventDefault()
		handleArrowRight(item)
	}
	if (event.key === 'ArrowLeft') {
		event.preventDefault()
		handleArrowLeft(item)
	}
	if (event.key === 'Enter' || event.key === ' ') {
		event.preventDefault()
		activateItem(item)
		scrollActiveItemIntoView()
	}
}

function handleItemClick(item: VisibleTreeItem, index: number): void {
	activeIndex.value = index
	activateItem(item)
}

function handleFocusout(event: FocusEvent): void {
	const currentTarget = event.currentTarget as HTMLElement
	const nextTarget = event.relatedTarget
	if (
		!(nextTarget instanceof Node)
		|| (!currentTarget.contains(nextTarget) && !popoverRef.value?.contains(nextTarget))
	) {
		closeTree()
	}
}

onBeforeUnmount(removePopoverGeometryListeners)
</script>

<template>
	<div
		ref="rootRef"
		:class="classes"
		@focusout="handleFocusout"
		@keydown="handleKeydown"
		@mouseenter="cancelMouseLeaveDismiss"
		@mouseleave="scheduleMouseLeaveDismiss"
	>
		<button
			class="makosh-tree-select__trigger"
			:class="{ 'makosh-tree-select__trigger--readonly': readonly }"
			type="button"
			:aria-activedescendant="activeItemId"
			:aria-controls="treeId"
			:aria-expanded="isOpen"
			:aria-haspopup="'tree'"
			:aria-label="accessibleLabel"
			:aria-readonly="readonly"
			:disabled="disabled"
			role="combobox"
			@click="toggleTree"
		>
			<span class="makosh-tree-select__value" :class="{ 'makosh-tree-select__value--placeholder': !selectedOption }">
				{{ displayLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" class="makosh-tree-select__chevron" aria-hidden="true" />
		</button>
		<Teleport to="body">
			<div
				v-if="isOpen"
				ref="popoverRef"
				class="makosh-tree-select__popover"
				:class="{ 'makosh-tree-select__popover--positioned': isPopoverPositioned }"
				@mouseenter="cancelMouseLeaveDismiss"
				@mouseleave="scheduleMouseLeaveDismiss"
			>
				<ul
					:id="treeId"
					ref="treeRef"
					class="makosh-tree-select__tree"
					role="tree"
					:aria-label="treeAriaLabel"
				>
					<li
						v-for="(item, index) in visibleItems"
						:key="item.option.value"
						class="makosh-tree-select__item"
						role="none"
					>
						<button
							:id="treeItemId(index)"
							class="makosh-tree-select__row"
							:class="{ 'makosh-tree-select__row--active': index === activeIndex }"
							type="button"
							role="treeitem"
							:aria-disabled="item.isDisabled"
							:aria-expanded="item.hasChildren ? item.isExpanded : undefined"
							:aria-level="item.level"
							:aria-posinset="item.posInSet"
							:aria-selected="item.isSelectable && item.option.value === modelValue"
							:aria-setsize="item.setSize"
							tabindex="-1"
							@mouseenter="setActiveIndexFromPointer(index)"
							@mousedown.prevent
							@click="handleItemClick(item, index)"
						>
							<span class="makosh-tree-select__spacer" aria-hidden="true">
								<span
									v-for="depth in Math.max(item.level - 1, 0)"
									:key="depth"
									class="makosh-tree-select__spacer-step"
								></span>
							</span>
							<Icon
								v-if="item.hasChildren"
								:icon="item.isExpanded ? 'tabler:chevron-down' : 'tabler:chevron-right'"
								size="0.875rem"
								class="makosh-tree-select__disclosure"
								aria-hidden="true"
							/>
							<span v-else class="makosh-tree-select__leaf-spacer" aria-hidden="true"></span>
							<Icon v-if="item.option.icon" :icon="item.option.icon" size="1rem" class="makosh-tree-select__icon" aria-hidden="true" />
							<span class="makosh-tree-select__body">
								<span class="makosh-tree-select__label">{{ item.option.label }}</span>
								<span v-if="item.option.description" class="makosh-tree-select__description">{{ item.option.description }}</span>
							</span>
							<Icon
								v-if="item.isSelectable && item.option.value === modelValue"
								icon="tabler:check"
								size="0.875rem"
								class="makosh-tree-select__check"
								aria-hidden="true"
							/>
						</button>
					</li>
					<li v-if="visibleItems.length === 0" class="makosh-tree-select__empty" role="presentation">{{ emptyLabel }}</li>
				</ul>
			</div>
		</Teleport>
	</div>
</template>
