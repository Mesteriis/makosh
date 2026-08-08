<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'
import Icon from './Icon.vue'
import type { TreeSelectOption } from './Selection.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

interface CascaderColumn {
	columnIndex: number
	items: CascaderItem[]
}

interface CascaderItem {
	key: string
	option: TreeSelectOption
	path: TreeSelectOption[]
	columnIndex: number
	optionIndex: number
	hasChildren: boolean
	isDisabled: boolean
	isDuplicateValue: boolean
	isInPath: boolean
	isSelectable: boolean
	isSelected: boolean
}

const props = withDefaults(defineProps<{
	modelValue?: string
	options?: TreeSelectOption[]
	placeholder?: string
	emptyLabel?: string
	ariaLabel?: string
	disabled?: boolean
	readonly?: boolean
	class?: string
}>(), {
	modelValue: '',
	options: () => [],
	placeholder: 'Select...',
	emptyLabel: 'No options',
	disabled: false,
	readonly: false
})

const emit = defineEmits<{
	'update:modelValue': [value: string]
	select: [option: TreeSelectOption]
	open: []
	close: []
}>()

const isOpen = ref(false)
const navigationPath = ref<TreeSelectOption[]>([])
const activeColumnIndex = ref(0)
const activeOptionIndex = ref(0)
const componentId = `makosh-cascader-${useId()}`
const rootRef = ref<HTMLElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeCascader, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, popoverRef.value]
})

const classes = computed(() => ['makosh-cascader', props.class])
const popoverId = computed(() => `${componentId}-popover`)
const canInteract = computed(() => !props.disabled && !props.readonly)
const accessibleLabel = computed(() => props.ariaLabel ?? props.placeholder)
const duplicateValues = computed(() => findDuplicateValues(props.options))
const selectedLeafPath = computed(() => findEnabledLeafPath(props.options, props.modelValue, duplicateValues.value))
const selectedOption = computed(() => selectedLeafPath.value?.at(-1))
const selectedValue = computed(() => selectedOption.value?.value)
const displayLabel = computed(() => selectedOption.value?.label ?? props.placeholder)
const columns = computed<CascaderColumn[]>(() => buildColumns())
const activeColumn = computed(() => columns.value[activeColumnIndex.value])
const activeItem = computed(() => activeColumn.value?.items[activeOptionIndex.value])
const activeItemId = computed(() => {
	if (!isOpen.value || !activeItem.value) {
		return undefined
	}
	return optionId(activeItem.value.columnIndex, activeItem.value.optionIndex)
})

watch([() => props.modelValue, () => props.options], () => {
	syncNavigationPathFromModel()
	if (isOpen.value) {
		setActivePositionToPathEnd()
		return
	}
	clampActivePosition()
}, { deep: true, immediate: true })

watch(columns, () => {
	clampActivePosition()
})

watch(canInteract, (nextCanInteract) => {
	if (!nextCanInteract) {
		closeCascader()
	}
})

function hasChildren(option: TreeSelectOption): boolean {
	return Boolean(option.children?.length)
}

function findDuplicateValues(options: TreeSelectOption[]): Set<string> {
	const counts = new Map<string, number>()
	const collect = (nodes: TreeSelectOption[]): void => {
		for (const option of nodes) {
			counts.set(option.value, (counts.get(option.value) ?? 0) + 1)
			collect(option.children ?? [])
		}
	}
	collect(options)
	return new Set([...counts.entries()].filter(([, count]) => count > 1).map(([value]) => value))
}

function findEnabledLeafPath(
	options: TreeSelectOption[],
	value: string,
	duplicateValueSet: Set<string>,
	ancestors: TreeSelectOption[] = [],
	ancestorDisabled = false
): TreeSelectOption[] | undefined {
	if (value === '' || duplicateValueSet.has(value)) {
		return undefined
	}
	for (const option of options) {
		const path = [...ancestors, option]
		const isDisabled = ancestorDisabled || Boolean(option.disabled)
		if (option.value === value) {
			if (isDisabled || hasChildren(option)) {
				return undefined
			}
			return path
		}
		const childPath = findEnabledLeafPath(option.children ?? [], value, duplicateValueSet, path, isDisabled)
		if (childPath) {
			return childPath
		}
	}
	return undefined
}

function buildColumns(): CascaderColumn[] {
	const result: CascaderColumn[] = []
	let currentOptions = props.options
	let pathPrefix: TreeSelectOption[] = []
	let ancestorDisabled = false
	let columnIndex = 0

	while (true) {
		result.push({
			columnIndex,
			items: currentOptions.map((option, optionIndex) =>
				toCascaderItem(option, optionIndex, columnIndex, pathPrefix, ancestorDisabled)
			)
		})

		const pathOption = navigationPath.value[columnIndex]
		const currentOption = currentOptions.find((option) => option === pathOption)
		if (!currentOption || !hasChildren(currentOption) || ancestorDisabled || currentOption.disabled) {
			break
		}

		pathPrefix = [...pathPrefix, currentOption]
		currentOptions = currentOption.children ?? []
		ancestorDisabled = ancestorDisabled || Boolean(currentOption.disabled)
		columnIndex += 1
	}

	return result
}

function toCascaderItem(
	option: TreeSelectOption,
	optionIndex: number,
	columnIndex: number,
	pathPrefix: TreeSelectOption[],
	ancestorDisabled: boolean
): CascaderItem {
	const optionHasChildren = hasChildren(option)
	const isDuplicateValue = duplicateValues.value.has(option.value)
	const isDisabled = ancestorDisabled || Boolean(option.disabled) || isDuplicateValue
	const isSelected = !optionHasChildren && !isDuplicateValue && selectedValue.value === option.value
	return {
		key: cascaderItemKey(pathPrefix, option, columnIndex, optionIndex),
		option,
		path: [...pathPrefix, option],
		columnIndex,
		optionIndex,
		hasChildren: optionHasChildren,
		isDisabled,
		isDuplicateValue,
		isInPath: navigationPath.value[columnIndex] === option,
		isSelectable: !isDisabled && !optionHasChildren,
		isSelected
	}
}

function syncNavigationPathFromModel(): void {
	navigationPath.value = selectedLeafPath.value ? [...selectedLeafPath.value] : []
}

function cascaderItemKey(
	pathPrefix: TreeSelectOption[],
	option: TreeSelectOption,
	columnIndex: number,
	optionIndex: number
): string {
	const pathValues = [...pathPrefix, option].map((pathOption) => pathOption.value).join('/')
	return `${columnIndex}:${optionIndex}:${pathValues}`
}

function columnId(columnIndex: number): string {
	return `${popoverId.value}-column-${columnIndex}`
}

function optionId(columnIndex: number, optionIndex: number): string {
	return `${columnId(columnIndex)}-option-${optionIndex}`
}

function columnLabel(columnIndex: number): string {
	return `${accessibleLabel.value} ${columnIndex + 1}`
}

function clampActivePosition(): void {
	if (columns.value.length === 0) {
		activeColumnIndex.value = 0
		activeOptionIndex.value = 0
		return
	}

	activeColumnIndex.value = Math.min(Math.max(activeColumnIndex.value, 0), columns.value.length - 1)
	const itemCount = columns.value[activeColumnIndex.value]?.items.length ?? 0
	if (itemCount === 0) {
		activeOptionIndex.value = 0
		return
	}
	activeOptionIndex.value = Math.min(Math.max(activeOptionIndex.value, 0), itemCount - 1)
	if (activeColumn.value?.items[activeOptionIndex.value]?.isDisabled) {
		activeOptionIndex.value = firstEnabledOptionIndex(activeColumnIndex.value)
	}
}

function scrollActiveItemIntoView(): void {
	void nextTick(() => {
		if (!activeItem.value) {
			return
		}
		document.getElementById(optionId(activeItem.value.columnIndex, activeItem.value.optionIndex))
			?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
	})
}

function setActivePosition(columnIndex: number, optionIndex: number): void {
	activeColumnIndex.value = columnIndex
	activeOptionIndex.value = optionIndex
	clampActivePosition()
	scrollActiveItemIntoView()
}

function setActivePositionToPathEnd(): void {
	if (navigationPath.value.length === 0) {
		setActivePosition(0, firstEnabledOptionIndex(0))
		return
	}

	const nextColumnIndex = Math.min(navigationPath.value.length - 1, columns.value.length - 1)
	const nextOption = navigationPath.value[nextColumnIndex]
	const nextOptionIndex = columns.value[nextColumnIndex]?.items.findIndex((item) => item.option === nextOption) ?? -1
	setActivePosition(nextColumnIndex, nextOptionIndex >= 0 ? nextOptionIndex : firstEnabledOptionIndex(nextColumnIndex))
}

function firstEnabledOptionIndex(columnIndex: number): number {
	const items = columns.value[columnIndex]?.items ?? []
	const enabledIndex = items.findIndex((item) => !item.isDisabled)
	return enabledIndex >= 0 ? enabledIndex : 0
}

function openCascader(): void {
	if (!canInteract.value) {
		return
	}
	cancelMouseLeaveDismiss()
	syncNavigationPathFromModel()
	setActivePositionToPathEnd()
	if (isOpen.value) {
		scrollActiveItemIntoView()
		return
	}
	isOpen.value = true
	emit('open')
	scrollActiveItemIntoView()
}

function closeCascader(): void {
	cancelMouseLeaveDismiss()
	if (!isOpen.value) {
		return
	}
	isOpen.value = false
	activeColumnIndex.value = 0
	activeOptionIndex.value = 0
	emit('close')
}

function toggleCascader(): void {
	if (isOpen.value) {
		closeCascader()
		return
	}
	openCascader()
}

function navigateToBranch(item: CascaderItem): void {
	if (!canInteract.value || item.isDisabled || !item.hasChildren) {
		return
	}
	navigationPath.value = item.path
	setActivePosition(item.columnIndex + 1, firstEnabledOptionIndex(item.columnIndex + 1))
}

function selectLeaf(item: CascaderItem): void {
	if (!canInteract.value || !item.isSelectable) {
		return
	}
	navigationPath.value = item.path
	emit('update:modelValue', item.option.value)
	emit('select', item.option)
	closeCascader()
}

function activateItem(item: CascaderItem | undefined): void {
	if (!item || item.isDisabled) {
		return
	}
	if (item.hasChildren) {
		navigateToBranch(item)
		return
	}
	selectLeaf(item)
}

function moveActiveOption(delta: number): void {
	const items = activeColumn.value?.items ?? []
	const itemCount = items.length
	if (itemCount === 0) {
		return
	}
	const direction = delta > 0 ? 1 : -1
	let nextOptionIndex = activeOptionIndex.value
	while (true) {
		const candidateIndex = nextOptionIndex + direction
		if (candidateIndex < 0 || candidateIndex >= itemCount) {
			return
		}
		nextOptionIndex = candidateIndex
		if (!items[nextOptionIndex]?.isDisabled) {
			activeOptionIndex.value = nextOptionIndex
			scrollActiveItemIntoView()
			return
		}
	}
}

function moveToPreviousColumn(): void {
	if (activeColumnIndex.value === 0) {
		return
	}
	const previousColumnIndex = activeColumnIndex.value - 1
	const parentOption = navigationPath.value[previousColumnIndex]
	const parentOptionIndex = columns.value[previousColumnIndex]?.items.findIndex((item) => item.option === parentOption) ?? -1
	setActivePosition(previousColumnIndex, parentOptionIndex >= 0 ? parentOptionIndex : firstEnabledOptionIndex(previousColumnIndex))
}

function moveToNextColumn(item: CascaderItem): void {
	if (!item.hasChildren || item.isDisabled) {
		return
	}
	const existingPathItem = navigationPath.value[item.columnIndex]
	if (existingPathItem !== item.option) {
		navigateToBranch(item)
		return
	}
	setActivePosition(item.columnIndex + 1, firstEnabledOptionIndex(item.columnIndex + 1))
}

function handleKeydown(event: KeyboardEvent): void {
	if (event.key === 'Escape') {
		closeCascader()
		return
	}
	if (!isOpen.value && ['ArrowDown', 'Enter', ' '].includes(event.key)) {
		event.preventDefault()
		openCascader()
		return
	}
	if (!isOpen.value) {
		return
	}
	if (event.key === 'ArrowDown') {
		event.preventDefault()
		moveActiveOption(1)
		return
	}
	if (event.key === 'ArrowUp') {
		event.preventDefault()
		moveActiveOption(-1)
		return
	}
	if (event.key === 'ArrowRight') {
		event.preventDefault()
		if (activeItem.value) {
			moveToNextColumn(activeItem.value)
		}
		return
	}
	if (event.key === 'ArrowLeft') {
		event.preventDefault()
		moveToPreviousColumn()
		return
	}
	if (event.key === 'Enter' || event.key === ' ') {
		event.preventDefault()
		activateItem(activeItem.value)
	}
}

function handleFocusout(event: FocusEvent): void {
	const currentTarget = event.currentTarget as HTMLElement
	const nextTarget = event.relatedTarget
	if (!(nextTarget instanceof Node) || !currentTarget.contains(nextTarget)) {
		closeCascader()
	}
}
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
			class="makosh-searchable-select__trigger makosh-cascader__trigger"
			:class="{ 'makosh-searchable-select__trigger--readonly': readonly, 'makosh-cascader__trigger--readonly': readonly }"
			type="button"
			role="combobox"
			:aria-activedescendant="activeItemId"
			:aria-controls="popoverId"
			:aria-expanded="isOpen"
			:aria-haspopup="'listbox'"
			:aria-label="accessibleLabel"
			:aria-readonly="readonly"
			:disabled="disabled"
			@click="toggleCascader"
		>
			<span
				class="makosh-searchable-select__value makosh-cascader__value"
				:class="{ 'makosh-searchable-select__value--placeholder': !selectedOption, 'makosh-cascader__value--placeholder': !selectedOption }"
			>
				{{ displayLabel }}
			</span>
			<Icon icon="tabler:chevron-down" size="1rem" class="makosh-searchable-select__chevron makosh-cascader__chevron" aria-hidden="true" />
		</button>
		<div v-if="isOpen" :id="popoverId" ref="popoverRef" class="makosh-cascader__popover">
			<ul
				v-for="column in columns"
				:id="columnId(column.columnIndex)"
				:key="column.columnIndex"
				class="makosh-cascader__column"
				role="listbox"
				:aria-label="columnLabel(column.columnIndex)"
			>
				<li v-if="column.items.length === 0" class="makosh-cascader__empty" role="option" aria-disabled="true">
					{{ emptyLabel }}
				</li>
				<li v-for="item in column.items" :key="item.key" class="makosh-cascader__item" role="none">
					<button
						:id="optionId(item.columnIndex, item.optionIndex)"
						class="makosh-cascader__option"
						:class="{
							'makosh-cascader__option--active': item.columnIndex === activeColumnIndex && item.optionIndex === activeOptionIndex,
							'makosh-cascader__option--path': item.isInPath,
							'makosh-cascader__option--selected': item.isSelected
						}"
						type="button"
						role="option"
						:aria-current="item.isInPath ? 'true' : undefined"
						:aria-disabled="!canInteract || item.isDisabled"
						:aria-selected="item.isSelected"
						:disabled="!canInteract || item.isDisabled"
						tabindex="-1"
						@mouseenter="!item.isDisabled && setActivePosition(item.columnIndex, item.optionIndex)"
						@mousedown.prevent
						@click="activateItem(item)"
					>
						<Icon v-if="item.option.icon" :icon="item.option.icon" size="1rem" class="makosh-cascader__icon" aria-hidden="true" />
						<span class="makosh-cascader__body">
							<span class="makosh-cascader__label">{{ item.option.label }}</span>
							<span v-if="item.option.description" class="makosh-cascader__description">{{ item.option.description }}</span>
						</span>
						<Icon
							v-if="item.isSelected"
							icon="tabler:check"
							size="0.875rem"
							class="makosh-cascader__check"
							aria-hidden="true"
						/>
						<Icon
							v-else-if="item.hasChildren"
							icon="tabler:chevron-right"
							size="0.875rem"
							class="makosh-cascader__disclosure"
							aria-hidden="true"
						/>
					</button>
				</li>
			</ul>
		</div>
	</div>
</template>
