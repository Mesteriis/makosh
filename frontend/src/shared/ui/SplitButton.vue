<script setup lang="ts">
import {
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuPortal,
	DropdownMenuRoot,
	DropdownMenuTrigger
} from 'reka-ui'
import { computed, ref } from 'vue'
import Button from './primitives/Button.vue'
import Icon from './Icon.vue'
import IconButton from './IconButton.vue'
import type { NavigationItem } from './Navigation.types'
import { useMouseLeaveDismiss } from './useMouseLeaveDismiss'

const props = withDefaults(defineProps<{
	label: string
	items?: NavigationItem[]
	modelValue?: string
	icon?: string
	variant?: 'default' | 'secondary' | 'outline' | 'ghost' | 'destructive'
	size?: 'sm' | 'md' | 'lg'
	disabled?: boolean
	menuLabel?: string
	class?: string
}>(), {
	items: () => [],
	modelValue: '',
	variant: 'default',
	size: 'md',
	disabled: false,
	menuLabel: 'Open action menu'
})

const emit = defineEmits<{
	click: [event: MouseEvent]
	'update:modelValue': [value: string]
	select: [item: NavigationItem]
}>()

const isOpen = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const menuRef = ref<HTMLElement | { $el?: Element | null } | null>(null)
const classes = computed(() => ['makosh-split-button', props.class])
const { cancelMouseLeaveDismiss, scheduleMouseLeaveDismiss } = useMouseLeaveDismiss(closeMenu, undefined, {
	isOpen,
	getBoundaryElements: () => [rootRef.value, menuRef.value]
})

function handlePrimaryClick(event: MouseEvent): void {
	if (props.disabled) {
		return
	}
	emit('click', event)
}

function setOpen(value: boolean): void {
	if (value && !props.disabled && props.items.length > 0) {
		cancelMouseLeaveDismiss()
	}

	isOpen.value = value && !props.disabled && props.items.length > 0
}

function closeMenu(): void {
	cancelMouseLeaveDismiss()
	setOpen(false)
}

function selectItem(item: NavigationItem): void {
	if (item.disabled) {
		return
	}
	emit('update:modelValue', item.id)
	emit('select', item)
	closeMenu()
}
</script>

<template>
	<DropdownMenuRoot :open="isOpen" @update:open="setOpen">
		<div
			ref="rootRef"
			:class="classes"
			@keydown.escape="closeMenu"
			@mouseenter="cancelMouseLeaveDismiss"
			@mouseleave="scheduleMouseLeaveDismiss"
		>
			<Button
				class="makosh-split-button__main"
				:disabled="disabled"
				:icon="icon"
				:size="size"
				:title="label"
				:variant="variant"
				@click="handlePrimaryClick"
			>
				{{ label }}
			</Button>
			<DropdownMenuTrigger as-child>
				<IconButton
					class="makosh-split-button__toggle"
					icon="tabler:chevron-down"
					:aria-expanded="isOpen"
					:disabled="disabled || items.length === 0"
					:label="menuLabel"
					:size="size"
					:variant="variant"
				/>
			</DropdownMenuTrigger>
		</div>
		<DropdownMenuPortal>
			<DropdownMenuContent
				ref="menuRef"
				class="makosh-split-button__menu"
				:aria-label="menuLabel"
				align="end"
				side="bottom"
				:side-offset="4"
				@mouseenter="cancelMouseLeaveDismiss"
				@mouseleave="scheduleMouseLeaveDismiss"
			>
				<DropdownMenuItem v-for="item in items" :key="item.id" as-child>
					<button
						class="makosh-split-button__item"
						role="menuitem"
						type="button"
						:aria-current="item.id === modelValue ? 'true' : undefined"
						:disabled="item.disabled"
						@click="selectItem(item)"
					>
						<Icon v-if="item.icon" :icon="item.icon" size="1rem" class="makosh-split-button__icon" aria-hidden="true" />
						<span>{{ item.label }}</span>
					</button>
				</DropdownMenuItem>
			</DropdownMenuContent>
		</DropdownMenuPortal>
	</DropdownMenuRoot>
</template>
