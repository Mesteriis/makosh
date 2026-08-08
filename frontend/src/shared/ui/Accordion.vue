<script setup lang="ts">
import { computed, useId } from 'vue'
import type { AccordionItem } from './Accordion.types'

const props = withDefaults(defineProps<{
	items: AccordionItem[]
	modelValue?: string[]
	multiple?: boolean
	collapsible?: boolean
	class?: string
}>(), {
	modelValue: () => [],
	multiple: false,
	collapsible: true
})

const emit = defineEmits<{
	'update:modelValue': [value: string[]]
}>()

const accordionId = useId()

const classes = computed(() => [
	'makosh-accordion',
	props.multiple ? 'makosh-accordion--multiple' : 'makosh-accordion--single',
	props.class
])

const itemIds = computed(() => props.items.map((item) => item.id))
const itemIdSet = computed(() => new Set(itemIds.value))
const fallbackOpenId = computed(() => props.items.find((item) => !item.disabled)?.id ?? props.items[0]?.id)
const normalizedOpenIds = computed(() => {
	const openIds = props.modelValue.filter((id) => itemIdSet.value.has(id))

	if (openIds.length > 0) {
		return props.multiple ? openIds : openIds.slice(0, 1)
	}

	return !props.collapsible && fallbackOpenId.value ? [fallbackOpenId.value] : []
})
const openIdSet = computed(() => new Set(normalizedOpenIds.value))

function itemPanelId(item: AccordionItem): string {
	return `makosh-accordion-${accordionId}-panel-${item.id}`
}

function itemTriggerId(item: AccordionItem): string {
	return `makosh-accordion-${accordionId}-trigger-${item.id}`
}

function isOpen(item: AccordionItem): boolean {
	return openIdSet.value.has(item.id)
}

function toggleItem(item: AccordionItem): void {
	if (item.disabled) {
		return
	}

	const itemIsOpen = isOpen(item)
	const currentOpenIds = normalizedOpenIds.value

	if (!props.multiple) {
		if (itemIsOpen) {
			if (!props.collapsible) {
				return
			}

			emit('update:modelValue', [])
			return
		}

		emit('update:modelValue', [item.id])
		return
	}

	if (itemIsOpen) {
		if (!props.collapsible && currentOpenIds.length <= 1) {
			return
		}

		emit('update:modelValue', currentOpenIds.filter((id) => id !== item.id))
		return
	}

	emit('update:modelValue', [...currentOpenIds, item.id])
}
</script>

<template>
	<div :class="classes">
		<section
			v-for="item in items"
			:key="item.id"
			:class="[
				'makosh-accordion-item',
				isOpen(item) && 'makosh-accordion-item--open',
				item.disabled && 'makosh-accordion-item--disabled'
			]"
		>
			<h3 class="makosh-accordion-heading">
				<button
					class="makosh-accordion-trigger"
					type="button"
					:aria-controls="itemPanelId(item)"
					:aria-expanded="isOpen(item)"
					:disabled="item.disabled"
					:id="itemTriggerId(item)"
					@click="toggleItem(item)"
				>
					<span class="makosh-accordion-title">{{ item.title }}</span>
					<span v-if="item.description" class="makosh-accordion-description">
						{{ item.description }}
					</span>
					<span class="makosh-accordion-indicator" aria-hidden="true">+</span>
				</button>
			</h3>
			<div
				v-if="isOpen(item)"
				:id="itemPanelId(item)"
				class="makosh-accordion-panel"
				role="region"
				:aria-labelledby="itemTriggerId(item)"
			>
				<slot name="item" :item="item" />
			</div>
		</section>
	</div>
</template>
